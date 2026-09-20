use crate::backend::models::*;
use crate::backend::process::{LogStreamEvent, ProcessRunner};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct ShellyClient {
    pub binary_path: String,
}

impl Default for ShellyClient {
    fn default() -> Self {
        Self::new(None)
    }
}

impl ShellyClient {
    /// Crée une nouvelle instance de client Shelly en localisant le binaire
    pub fn new(custom_path: Option<String>) -> Self {
        let binary_path = if let Some(path) = custom_path {
            path
        } else if let Ok(env_path) = std::env::var("SHELLY_BIN") {
            env_path
        } else {
            // Priorité : binaire compilé localement dans ../Shelly.Cli.Zig/zig-out/bin/shelly, puis système
            let local_build = PathBuf::from("../Shelly.Cli.Zig/zig-out/bin/shelly");
            if local_build.exists() {
                local_build.to_string_lossy().to_string()
            } else if Path::new("/usr/bin/shelly").exists() {
                "/usr/bin/shelly".to_string()
            } else {
                "shelly".to_string()
            }
        };

        log::info!("ShellyClient initialisé avec le binaire : {}", binary_path);
        Self { binary_path }
    }

    /// Recherche les paquets dans les dépôts ALPM (officiels)
    pub async fn search_standard(&self, query: &str) -> Result<Vec<AlpmPackage>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let raw = ProcessRunner::run_json_command(
            &self.binary_path,
            &["search", "standard", "-v", query, "-j"],
        )
        .await?;

        Self::parse_alpm_results(&raw)
    }

    /// Recherche simultanément et en parallèle dans les dépôts officiels, l'AUR et Flatpak
    pub async fn search_all(
        &self,
        query: &str,
    ) -> (
        Result<Vec<AlpmPackage>>,
        Result<Vec<AurPackage>>,
        Result<Vec<FlatpakHit>>,
    ) {
        tokio::join!(
            self.search_standard(query),
            self.search_aur(query),
            self.search_flatpak(query)
        )
    }

    /// Recherche les paquets installés localement
    pub async fn search_installed(&self, query: &str) -> Result<Vec<AlpmPackage>> {
        let args = if query.trim().is_empty() {
            vec!["list", "standard", "-j"]
        } else {
            vec!["search", "standard", "-i", query, "-j"]
        };

        let raw = ProcessRunner::run_json_command(&self.binary_path, &args).await?;
        Self::parse_alpm_results(&raw)
    }

    /// Recherche les paquets dans l'AUR
    pub async fn search_aur(&self, query: &str) -> Result<Vec<AurPackage>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let raw =
            ProcessRunner::run_json_command(&self.binary_path, &["search", "aur", query, "-j"])
                .await?;

        if raw.trim().starts_with('[') {
            serde_json::from_str::<Vec<AurPackage>>(&raw)
                .context("Désérialisation de la liste des paquets AUR")
        } else if raw.trim().starts_with('{') {
            let single: AurPackage = serde_json::from_str(&raw)?;
            Ok(vec![single])
        } else {
            Ok(Vec::new())
        }
    }

    /// Recherche les applications Flatpak
    pub async fn search_flatpak(&self, query: &str) -> Result<Vec<FlatpakHit>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let raw =
            ProcessRunner::run_json_command(&self.binary_path, &["search", "flatpak", query, "-j"])
                .await?;

        if let Ok(res) = serde_json::from_str::<FlatpakSearchResult>(&raw) {
            Ok(res.hits)
        } else {
            Ok(Vec::new())
        }
    }

    /// Liste des mises à jour disponibles pour tous les backends
    pub async fn list_updates(&self) -> Result<Vec<PackageUpdateItem>> {
        let raw =
            ProcessRunner::run_json_command(&self.binary_path, &["list-updates", "all", "-j"])
                .await?;

        if raw.trim().starts_with('[') {
            serde_json::from_str::<Vec<PackageUpdateItem>>(&raw)
                .context("Désérialisation de la liste des mises à jour")
        } else {
            Ok(Vec::new())
        }
    }

    /// Récupère les actualités Arch Linux
    pub async fn list_news(&self) -> Result<Vec<ArchNewsItem>> {
        let raw = ProcessRunner::run_json_command(&self.binary_path, &["news", "-j"]).await?;
        if raw.trim().starts_with('[') {
            serde_json::from_str::<Vec<ArchNewsItem>>(&raw)
                .context("Désérialisation des actualités Arch Linux")
        } else {
            Ok(Vec::new())
        }
    }

    /// Récupère la fiche détaillée d'un paquet ALPM
    pub async fn get_package_details(&self, name: &str) -> Result<Option<AlpmPackage>> {
        let raw = ProcessRunner::run_json_command(
            &self.binary_path,
            &["search", "standard", "-d", name, "-j"],
        )
        .await?;

        if raw.trim().starts_with('{') {
            let pkg: AlpmPackage = serde_json::from_str(&raw)?;
            Ok(Some(pkg))
        } else if raw.trim().starts_with('[') {
            let pkgs: Vec<AlpmPackage> = serde_json::from_str(&raw)?;
            Ok(pkgs.into_iter().next())
        } else {
            Ok(None)
        }
    }

    /// Lance l'installation d'un paquet en streamant les logs dans le canal tx
    pub fn install_package(
        &self,
        name: &str,
        is_aur: bool,
        is_flatpak: bool,
        tx: mpsc::UnboundedSender<LogStreamEvent>,
    ) {
        let mut args = vec!["install".to_string()];
        if is_flatpak {
            args.push("flatpak".to_string());
        } else if is_aur {
            args.push("aur".to_string());
        } else {
            args.push("standard".to_string());
        }
        args.push(name.to_string());
        args.push("--ui-mode".to_string());
        // --no-confirm évite que le CLI Zig bloque en attendant une entrée clavier
        args.push("--no-confirm".to_string());

        ProcessRunner::spawn_streaming_operation(self.binary_path.clone(), args, tx);
    }

    /// Lance la suppression d'un paquet en streamant les logs
    pub fn remove_package(
        &self,
        name: &str,
        is_flatpak: bool,
        tx: mpsc::UnboundedSender<LogStreamEvent>,
    ) {
        let mut args = vec!["remove".to_string()];
        if is_flatpak {
            args.push("flatpak".to_string());
        } else {
            args.push("standard".to_string());
        }
        args.push(name.to_string());
        args.push("--ui-mode".to_string());
        args.push("--no-confirm".to_string());

        ProcessRunner::spawn_streaming_operation(self.binary_path.clone(), args, tx);
    }

    /// Lance la mise à niveau globale du système (tous les backends)
    pub fn upgrade_system(&self, tx: mpsc::UnboundedSender<LogStreamEvent>) {
        let args = vec![
            "upgrade".to_string(),
            "all".to_string(), // sous-commande "all" obligatoire
            "--ui-mode".to_string(),
            "--no-confirm".to_string(),
        ];
        ProcessRunner::spawn_streaming_operation(self.binary_path.clone(), args, tx);
    }

    /// Helper privé pour parser les résultats ALPM qu'ils soient sous forme de tableau ou d'objet unique
    fn parse_alpm_results(raw: &str) -> Result<Vec<AlpmPackage>> {
        let trimmed = raw.trim();
        if trimmed.starts_with('[') {
            serde_json::from_str::<Vec<AlpmPackage>>(trimmed)
                .context("Désérialisation du tableau JSON ALPM")
        } else if trimmed.starts_with('{') {
            let pkg: AlpmPackage =
                serde_json::from_str(trimmed).context("Désérialisation de l'objet JSON ALPM")?;
            Ok(vec![pkg])
        } else {
            Ok(Vec::new())
        }
    }
}

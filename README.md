# ⚡ Shelly GPUI

<div align="center">

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Zig](https://img.shields.io/badge/Zig-0.13%20%7C%200.14%20%7C%200.16-f7a41d.svg)](https://ziglang.org/)
[![GPUI](https://img.shields.io/badge/UI-GPUI%200.2-blueviolet.svg)](https://github.com/zed-industries/zed/tree/main/crates/gpui)
[![Arch Linux](https://img.shields.io/badge/Arch%20Linux-ALPM%20%2B%20AUR-1793d1.svg)](https://archlinux.org/)

**Gestionnaire de paquets unifié haute performance pour Arch Linux, accéléré par GPU.**  
*Dépôts officiels (ALPM), AUR, Flatpaks et AppImages réunis dans une interface réactive inspirée de Zed.*

[Fonctionnalités](#-fonctionnalités) •
[Architecture](#-architecture) •
[Installation & Compilation](#-installation--compilation) •
[Raccourcis & Utilisation](#-raccourcis--utilisation) •
[Packaging AUR](#-packaging-aur) •
[Crédits](#-crédits)

</div>

---

## 🚀 Fonctionnalités

- ⚡ **Accélération matérielle native GPU** : Construit sur le framework **GPUI** (le moteur graphique de Zed), garantissant un rendu fluide à 120+ FPS via Vulkan/Wayland/Blade Graphics.
- 📦 **Support multi-backends unifié** :
  - **Dépôts officiels Arch Linux** (via libalpm / Shelly CLI)
  - **AUR** (clones git, builds sécurisés en chroot)
  - **Flatpaks** (gestion des catalogues Flathub et permissions)
  - **AppImages** (détection et indexation automatique)
- 🔍 **Recherche en direct interactive (`TextInput`)** : Saisie fluide avec debounce prédictif de 300 ms sans bloquer le thread graphique.
- 📋 **Volet des détails enrichi** : Dépendances, dépendances optionnelles, reverse dependencies ("Requis par"), paquet de base, motifs d'installation, licences et tailles réelles.
- 📟 **Terminal embarqué & Streaming de logs** : Tiroir inférieur repliable permettant de suivre chaque transaction `pacman`, build AUR ou installation Flatpak en temps réel avec élévation Polkit.
- 🔄 **Rafraîchissement automatique** : Synchronisation immédiate des compteurs et listes dès la complétion réussie d'une transaction.
- 🎨 **Thème moderne Catppuccin Mocha** : Typographie soignée, contrastes travaillés, badges contextuels par source.

---

## 🏗 Architecture

Shelly GPUI adopte une séparation stricte des responsabilités entre la vue et le moteur de transaction système :

```mermaid
graph TD
    subgraph Frontend [Shelly.Ui.Gpui (Rust)]
        GPUI[GPUI / Blade Renderer] --> Workspace[WorkspaceView & Panes]
        Workspace --> LiveSearch[Live Search / Debounce 300ms]
        Workspace --> LogDrawer[Terminal Log Drawer]
        LogDrawer --> Stream[Tokio Async Streaming Process]
    end

    subgraph Backend [Shelly.Cli.Zig (Zig)]
        Stream -->|shelly --ui-mode -j| CLI[Shelly CLI Engine]
        CLI --> ALPM[libalpm / Pacman Engine]
        CLI --> AUR[AUR RPC / Git Builder]
        CLI --> FP[Flatpak Engine]
        CLI --> AI[AppImage Manager]
        CLI --> Polkit[Polkit / pkexec Auth]
    end
```

- **Frontend (`Shelly.Ui.Gpui`)** : Application Rust compilée avec GPUI 0.2, gérant le rendu vectoriel direct sur GPU, la disposition flexible et l'état réactif de l'interface.
- **Backend (`Shelly.Cli.Zig`)** : Moteur natif Zig interagissant directement avec `libalpm.so` et les sources externes. Il émet des flux JSON structurés pour les requêtes de lecture et des flux encadrés `--ui-mode` pour les opérations avec logs en direct.

---

## 🛠 Installation & Compilation

### Prérequis

Sur Arch Linux ou dérivés :

```bash
sudo pacman -S --needed base-devel git rust cargo zig pacman libglvnd fontconfig freetype2 wayland mesa vulkan-icd-loader polkit
```

### 1. Compilation et lancement rapide avec `make`

Le dépôt inclut un `Makefile` universel qui compile automatiquement le backend Zig et lance le frontend :

```bash
git clone https://github.com/tension-atoi/shelly-gpui.git
cd shelly-gpui

# Lance directement l'application en mode développement
make run

# Ou compiler le binaire de release optimisé
make build
```

Les commandes du Makefile :
- `make run` : compile le backend Zig si nécessaire et lance `cargo run`
- `make build` : produit le binaire release optimisé (`shelly-gpui`)
- `make check` : vérifie la validité du code Rust sans étape de link
- `make install` : installe les binaires et fichiers de bureau dans `/usr/local`
- `make package` : teste la création du paquet Arch Linux via `makepkg`
- `make clean` : nettoie les caches et artefacts de compilation

---

## ⌨ Raccourcis & Utilisation

| Touche / Action | Contexte | Effet |
|---|---|---|
| **Clic sur recherche** | Volet gauche | Active la saisie avec curseur visuel `▌` |
| **Saisie texte** | Barre de recherche | Recherche live avec debounce automatique de 300 ms |
| **Entrée** | Barre de recherche | Force la recherche immédiatement (bypass debounce) |
| **Échap** | Barre de recherche | Efface la requête et désactive le focus clavier |
| **Clic sur la barre du bas** | Partout | Ouvre / ferme le tiroir de logs du terminal |
| **Onglets (Haut)** | Navigation | Bascule entre ALPM, AUR, Flatpaks, AppImages, Mises à jour, News, Paramètres |

---

## 📦 Packaging Arch Linux (AUR)

Un fichier [`PKGBUILD-gpui`](PKGBUILD-gpui) ainsi qu'un répertoire [`packaging/aur/`](packaging/aur/) sont fournis pour installer l'application proprement via pacman :

```bash
# Compilation du paquet localement
makepkg -si --noconfirm
```

Le paquet installe :
- `/usr/bin/shelly-gpui` : lanceur système avec détection automatique de l'environnement
- `/usr/lib/shelly/shelly` : moteur backend Zig
- `/usr/lib/shelly/shelly-gpui-bin` : binaire graphique GPUI Rust
- `/usr/share/applications/com.shellyorg.shelly-gpui.desktop` : intégration au menu d'applications
- `/usr/share/icons/hicolor/scalable/apps/shelly-gpui.svg` : icône vectorielle Catppuccin
- `/usr/share/polkit-1/actions/com.shellyorg.shelly-gpui.policy` : règles d'élévation Polkit pour l'installation/suppression sans mot de passe intempestif

---

## 🤝 Crédits & Remerciements

- [Seafoam-Labs/Shelly-ALPM](https://github.com/Seafoam-Labs/Shelly-ALPM) pour le moteur initial Shelly et son architecture Zig / ALPM.
- [Zed Industries](https://github.com/zed-industries/zed) pour le framework graphique [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui).
- La communauté [Catppuccin](https://github.com/catppuccin/catppuccin) pour la palette de couleurs Mocha.

---

## 📄 Licence

Ce projet est distribué sous licence **GPL-3.0**. Consultez le fichier [LICENSE](LICENSE) pour plus de détails.

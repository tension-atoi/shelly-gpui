use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use crate::components::status_pill::StatusPill;
use crate::models::InspectorTab;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;

use std::rc::Rc;

pub type MouseClickHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

pub struct PackageDetailsProps<'a> {
    pub package: Option<&'a UnifiedPackage>,
    pub alpm_details: Option<&'a AlpmPackage>,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub active_tab: InspectorTab,
    pub file_list: &'a [String],
    pub on_install: Option<MouseClickHandler>,
    pub on_remove: Option<MouseClickHandler>,
    pub on_change_tab: Option<Rc<dyn Fn(InspectorTab, &mut Window, &mut App) + 'static>>,
    pub on_navigate_dep: Option<Rc<dyn Fn(String, &mut Window, &mut App) + 'static>>,
}

pub struct PackageDetailsView;

impl PackageDetailsView {
    fn render_tab_button(
        tab: InspectorTab,
        active_tab: InspectorTab,
        theme: &Theme,
        on_change_tab: Option<Rc<dyn Fn(InspectorTab, &mut Window, &mut App) + 'static>>,
    ) -> impl IntoElement {
        let is_active = active_tab == tab;
        let base = div()
            .px_3()
            .py_1p5()
            .rounded_t_md()
            .text_xs()
            .cursor_pointer();

        let styled = if is_active {
            base.text_color(theme.accent)
                .font_weight(FontWeight::BOLD)
                .border_b_2()
                .border_color(theme.accent)
                .bg(theme.bg_surface)
        } else {
            let hover_bg = theme.bg_surface_hover;
            let hover_text = theme.text_primary;
            base.text_color(theme.text_muted)
                .hover(move |s| s.bg(hover_bg).text_color(hover_text))
        };

        styled.child(tab.label()).on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
            if let Some(ref cb) = on_change_tab {
                cb(tab, window, cx);
            }
        })
    }

    pub fn render(props: PackageDetailsProps) -> AnyElement {
        let theme = props.theme;

        let Some(pkg) = props.package else {
            return div()
                .id("empty_details")
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .bg(theme.bg_app)
                .text_color(theme.text_muted)
                .child("Sélectionnez un paquet dans la liste pour afficher ses détails.")
                .into_any_element();
        };

        let mut root = div()
            .id("details_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        // En-tête du paquet
        let header = div()
            .flex()
            .flex_col()
            .mb_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_2()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(pkg.name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(StatusPill::source_badge(&pkg.source_type, theme))
                            .child(StatusPill::installed_pill(pkg.is_installed, theme)),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.accent)
                    .font_weight(FontWeight::MEDIUM)
                    .child(format!("Version : {}", pkg.version)),
            );

        root = root.child(header);

        // Barre d'onglets Inspector (Aperçu, Dépendances, Fichiers)
        let tab_bar = div()
            .flex()
            .items_center()
            .gap_2()
            .mb_5()
            .border_b_1()
            .border_color(theme.border)
            .child(Self::render_tab_button(
                InspectorTab::Overview,
                props.active_tab,
                theme,
                props.on_change_tab.clone(),
            ))
            .child(Self::render_tab_button(
                InspectorTab::Dependencies,
                props.active_tab,
                theme,
                props.on_change_tab.clone(),
            ))
            .child(Self::render_tab_button(
                InspectorTab::Files,
                props.active_tab,
                theme,
                props.on_change_tab.clone(),
            ));

        root = root.child(tab_bar);

        // Barre d'actions (Installer / Désinstaller / MAJ) toujours visible en haut
        let mut action_bar = div()
            .flex()
            .items_center()
            .gap_3()
            .mb_5();

        if pkg.is_installed {
            let danger_hover = theme.danger_hover;
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.danger })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(danger_hover))
                })
                .child(if props.is_busy { "En cours..." } else { "Désinstaller" });

            let btn = if !props.is_busy {
                if let Some(on_remove) = props.on_remove.clone() {
                    btn.on_mouse_down(MouseButton::Left, move |e, w, cx| on_remove(e, w, cx))
                } else {
                    btn
                }
            } else {
                btn
            };
            action_bar = action_bar.child(btn);
        } else {
            let accent_hover = theme.accent_hover;
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.accent })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(accent_hover))
                })
                .child(if props.is_busy { "En cours..." } else { "Installer" });

            let btn = if !props.is_busy {
                if let Some(on_install) = props.on_install.clone() {
                    btn.on_mouse_down(MouseButton::Left, move |e, w, cx| on_install(e, w, cx))
                } else {
                    btn
                }
            } else {
                btn
            };
            action_bar = action_bar.child(btn);
        }

        if pkg.has_update {
            let warning_hover = theme.warning_hover;
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.warning })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(warning_hover))
                })
                .child(if props.is_busy { "En cours..." } else { "Mettre à jour" });

            let btn = if !props.is_busy {
                if let Some(on_install) = props.on_install.clone() {
                    btn.on_mouse_down(MouseButton::Left, move |e, w, cx| on_install(e, w, cx))
                } else {
                    btn
                }
            } else {
                btn
            };
            action_bar = action_bar.child(btn);
        }

        root = root.child(action_bar);

        // Contenu conditionnel selon l'onglet actif
        match props.active_tab {
            InspectorTab::Overview => {
                // Description
                let desc_section = div()
                    .flex()
                    .flex_col()
                    .mb_6()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_muted)
                            .mb_1()
                            .child("DESCRIPTION"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(if pkg.description.is_empty() {
                                "Aucune description fournie pour ce paquet.".to_string()
                            } else {
                                pkg.description.clone()
                            }),
                    );
                root = root.child(desc_section);

                // Métadonnées
                match &pkg.inner {
                    UnifiedPackageSource::Standard(alpm) => {
                        let effective_alpm = props.alpm_details.unwrap_or(alpm);
                        let mut meta_grid = div()
                            .flex()
                            .flex_col()
                            .gap_2p5()
                            .p_4()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.border)
                            .mb_6();

                        if let Some(ref repo) = effective_alpm.repository {
                            meta_grid = meta_grid.child(Self::meta_row("Dépôt", repo, theme));
                        }
                        if let Some(ref base) = effective_alpm.package_base {
                            if base != &pkg.name {
                                meta_grid = meta_grid.child(Self::meta_row("Paquet de base", base, theme));
                            }
                        }
                        if let Some(ref url) = effective_alpm.url {
                            meta_grid = meta_grid.child(Self::meta_row("Site amont", url, theme));
                        }
                        if !effective_alpm.licenses.is_empty() {
                            meta_grid = meta_grid.child(Self::meta_row("Licence", &effective_alpm.licenses.join(", "), theme));
                        }
                        if let Some(dl_size) = effective_alpm.download_size {
                            meta_grid = meta_grid.child(Self::meta_row("Taille téléchargement", &Self::format_size(dl_size), theme));
                        }
                        if let Some(size) = effective_alpm.installed_size.or(effective_alpm.size) {
                            meta_grid = meta_grid.child(Self::meta_row("Taille installée", &Self::format_size(size), theme));
                        }
                        if let Some(ref reason) = effective_alpm.install_reason {
                            meta_grid = meta_grid.child(Self::meta_row("Motif d'installation", reason, theme));
                        }
                        if let Some(ref date) = effective_alpm.build_date {
                            meta_grid = meta_grid.child(Self::meta_row("Date de compilation", date, theme));
                        }
                        if let Some(ref date) = effective_alpm.install_date {
                            meta_grid = meta_grid.child(Self::meta_row("Date d'installation", date, theme));
                        }
                        if !effective_alpm.groups.is_empty() {
                            meta_grid = meta_grid.child(Self::meta_row("Groupes", &effective_alpm.groups.join(", "), theme));
                        }
                        root = root.child(meta_grid);
                    }
                    UnifiedPackageSource::Aur(aur) => {
                        let mut meta_grid = div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.border)
                            .mb_6();

                        if let Some(ref url) = aur.url {
                            meta_grid = meta_grid.child(Self::meta_row("Site amont", url, theme));
                        }
                        if let Some(votes) = aur.num_votes {
                            meta_grid = meta_grid.child(Self::meta_row("Votes AUR", &votes.to_string(), theme));
                        }
                        if let Some(ref maintainer) = aur.maintainer {
                            meta_grid = meta_grid.child(Self::meta_row("Mainteneur", maintainer, theme));
                        }
                        root = root.child(meta_grid);
                    }
                    UnifiedPackageSource::Flatpak(fp) => {
                        let mut meta_grid = div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.border)
                            .mb_6();

                        if let Some(ref remote) = fp.remote {
                            meta_grid = meta_grid.child(Self::meta_row("Dépôt distant", remote, theme));
                        }
                        if let Some(ref app_id) = fp.app_id {
                            meta_grid = meta_grid.child(Self::meta_row("Identifiant App", app_id, theme));
                        }
                        root = root.child(meta_grid);
                    }
                }
            }
            InspectorTab::Dependencies => {
                match &pkg.inner {
                    UnifiedPackageSource::Standard(alpm) => {
                        let effective_alpm = props.alpm_details.unwrap_or(alpm);
                        let on_nav = props.on_navigate_dep.clone();

                        if !effective_alpm.provides.is_empty() {
                            root = root.child(Self::chip_section(
                                "FOURNIT (PROVIDES)",
                                &effective_alpm.provides,
                                theme,
                                theme.text_secondary,
                                on_nav.clone(),
                            ));
                        }
                        if !effective_alpm.conflicts.is_empty() {
                            root = root.child(Self::chip_section(
                                "CONFLITS",
                                &effective_alpm.conflicts,
                                theme,
                                theme.danger,
                                on_nav.clone(),
                            ));
                        }
                        if !effective_alpm.depends.is_empty() {
                            root = root.child(Self::chip_section(
                                &format!("DÉPENDANCES DIRECTES ({})", effective_alpm.depends.len()),
                                &effective_alpm.depends,
                                theme,
                                theme.text_secondary,
                                on_nav.clone(),
                            ));
                        }
                        if !effective_alpm.opt_depends.is_empty() {
                            root = root.child(Self::chip_section(
                                &format!("DÉPENDANCES OPTIONNELLES ({})", effective_alpm.opt_depends.len()),
                                &effective_alpm.opt_depends,
                                theme,
                                theme.warning,
                                on_nav.clone(),
                            ));
                        }
                        if !effective_alpm.required_by.is_empty() {
                            root = root.child(Self::chip_section(
                                &format!("REQUIS PAR ({})", effective_alpm.required_by.len()),
                                &effective_alpm.required_by,
                                theme,
                                theme.text_muted,
                                on_nav,
                            ));
                        }
                    }
                    UnifiedPackageSource::Aur(aur) => {
                        let on_nav = props.on_navigate_dep.clone();
                        if let Some(ref deps) = aur.depends {
                            if !deps.is_empty() {
                                root = root.child(Self::chip_section(
                                    &format!("DÉPENDANCES AUR ({})", deps.len()),
                                    deps,
                                    theme,
                                    theme.text_secondary,
                                    on_nav.clone(),
                                ));
                            }
                        }
                        if let Some(ref mdeps) = aur.make_depends {
                            if !mdeps.is_empty() {
                                root = root.child(Self::chip_section(
                                    &format!("DÉPENDANCES DE COMPILATION ({})", mdeps.len()),
                                    mdeps,
                                    theme,
                                    theme.warning,
                                    on_nav,
                                ));
                            }
                        }
                    }
                    _ => {
                        root = root.child(
                            div()
                                .p_4()
                                .rounded_md()
                                .bg(theme.bg_surface)
                                .text_color(theme.text_muted)
                                .text_xs()
                                .child("Aucune information de dépendance détaillée pour ce paquet."),
                        );
                    }
                }
            }
            InspectorTab::Files => {
                if !pkg.is_installed {
                    root = root.child(
                        div()
                            .p_6()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .text_color(theme.text_muted)
                            .text_center()
                            .text_sm()
                            .child("La liste des fichiers n'est disponible que pour les paquets installés."),
                    );
                } else if props.file_list.is_empty() {
                    root = root.child(
                        div()
                            .p_6()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .text_color(theme.text_muted)
                            .text_center()
                            .text_sm()
                            .child("Recherche de la liste des fichiers installés..."),
                    );
                } else {
                    let mut file_box = div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_3()
                        .rounded_md()
                        .bg(theme.bg_surface)
                        .border_1()
                        .border_color(theme.border);

                    for file in props.file_list.iter().take(500) {
                        let is_bin = file.starts_with("/usr/bin/");
                        let is_lib = file.starts_with("/usr/lib/");
                        let icon = if is_bin { "⚙️ " } else if is_lib { "📚 " } else { "📄 " };

                        file_box = file_box.child(
                            div()
                                .flex()
                                .items_center()
                                .text_xs()
                                .text_color(if is_bin { theme.accent } else { theme.text_secondary })
                                .font_weight(if is_bin { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .child(icon)
                                .child(file.clone()),
                        );
                    }
                    root = root.child(file_box);
                }
            }
        }

        root.into_any_element()
    }

    fn meta_row(label: &'static str, val: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_baseline()
            .justify_between()
            .gap_4()
            .text_xs()
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(theme.text_muted)
                    .child(label),
            )
            .child(
                div()
                    .text_color(theme.text_primary)
                    .font_weight(FontWeight::MEDIUM)
                    .child(val.to_string()),
            )
    }

    fn chip_section(
        title: &str,
        items: &[String],
        theme: &Theme,
        chip_text_color: Rgba,
        on_navigate: Option<Rc<dyn Fn(String, &mut Window, &mut App) + 'static>>,
    ) -> impl IntoElement {
        let section = div()
            .flex()
            .flex_col()
            .mb_6()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .mb_2()
                    .child(title.to_string()),
            );

        let hover_bg = theme.bg_surface_hover;
        let border_focus = theme.border_focus;

        let mut chips = div().flex().flex_wrap().gap_2();
        for item in items.iter().take(40) {
            let item_clone = item.clone();
            let nav_cb = on_navigate.clone();
            let mut chip = div()
                .px_2()
                .py_1()
                .rounded_sm()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .text_xs()
                .text_color(chip_text_color)
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(border_focus))
                .child(item.clone());

            if let Some(cb) = nav_cb {
                chip = chip.on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    // Extract package name by stripping version constraints like '>=1.0'
                    let clean_name = item_clone.split_whitespace().next().unwrap_or(&item_clone);
                    cb(clean_name.to_string(), window, cx);
                });
            }

            chips = chips.child(chip);
        }
        section.child(chips)
    }

    fn format_size(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} Go", (bytes as f64) / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} Mo", (bytes as f64) / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.2} Ko", (bytes as f64) / 1024.0)
        } else {
            format!("{} octets", bytes)
        }
    }
}

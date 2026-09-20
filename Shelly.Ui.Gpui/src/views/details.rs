use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use crate::components::status_pill::StatusPill;
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
    pub on_install: Option<MouseClickHandler>,
    pub on_remove: Option<MouseClickHandler>,
    pub on_navigate_dep: Option<Rc<dyn Fn(String, &mut Window, &mut App) + 'static>>,
}

pub struct PackageDetailsView;

impl PackageDetailsView {
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

        // En-tête du paquet (Nom, Version, Source, État)
        let header = div()
            .flex()
            .flex_col()
            .mb_5()
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
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted)
                            .child(pkg.version.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(StatusPill::source_badge(&pkg.source_type, theme))
                    .child(StatusPill::installed_pill(pkg.is_installed, theme)),
            );

        root = root.child(header);

        // Barre d'actions (Installer / Désinstaller / MAJ)
        let mut action_bar = div().flex().items_center().gap_3().mb_6();

        if pkg.is_installed {
            let danger_hover = theme.danger_hover;
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy {
                    theme.border
                } else {
                    theme.danger
                })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(danger_hover))
                })
                .child(if props.is_busy {
                    "En cours..."
                } else {
                    "Désinstaller"
                });

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
                .bg(if props.is_busy {
                    theme.border
                } else {
                    theme.accent
                })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(accent_hover))
                })
                .child(if props.is_busy {
                    "En cours..."
                } else {
                    "Installer"
                });

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
                .bg(if props.is_busy {
                    theme.border
                } else {
                    theme.warning
                })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                })
                .cursor_pointer()
                .when(!props.is_busy, move |el| {
                    el.hover(move |s| s.bg(warning_hover))
                })
                .child(if props.is_busy {
                    "En cours..."
                } else {
                    "Mettre à jour"
                });

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

        // Section Description
        let desc_section = div()
            .flex()
            .flex_col()
            .gap_1()
            .mb_6()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("DESCRIPTION"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_secondary)
                    .line_height(relative(1.4))
                    .child(if pkg.description.is_empty() {
                        "Aucune description disponible pour ce paquet.".to_string()
                    } else {
                        pkg.description.clone()
                    }),
            );
        root = root.child(desc_section);

        // Grille de métadonnées selon la source du paquet
        match &pkg.inner {
            UnifiedPackageSource::Standard(alpm) => {
                let effective_alpm = props.alpm_details.unwrap_or(alpm);
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

                if let Some(ref repo) = effective_alpm.repository {
                    meta_grid = meta_grid.child(Self::meta_row("Dépôt", repo, theme));
                }
                if let Some(ref url) = effective_alpm.url {
                    meta_grid = meta_grid.child(Self::meta_row("Site amont", url, theme));
                }
                if !effective_alpm.licenses.is_empty() {
                    meta_grid = meta_grid.child(Self::meta_row(
                        "Licence",
                        &effective_alpm.licenses.join(", "),
                        theme,
                    ));
                }
                if let Some(ref size) = effective_alpm.installed_size {
                    let formatted_size = format!("{:.1} Mo", *size as f64 / (1024.0 * 1024.0));
                    meta_grid =
                        meta_grid.child(Self::meta_row("Taille installée", &formatted_size, theme));
                }

                root = root.child(meta_grid);

                // Section Dépendances
                if !effective_alpm.depends.is_empty() {
                    root = root.child(Self::chip_section(
                        &format!("DÉPENDANCES ({})", effective_alpm.depends.len()),
                        &effective_alpm.depends,
                        theme,
                        theme.text_secondary,
                        props.on_navigate_dep.clone(),
                    ));
                }
                if !effective_alpm.opt_depends.is_empty() {
                    root = root.child(Self::chip_section(
                        &format!(
                            "DÉPENDANCES OPTIONNELLES ({})",
                            effective_alpm.opt_depends.len()
                        ),
                        &effective_alpm.opt_depends,
                        theme,
                        theme.warning,
                        props.on_navigate_dep.clone(),
                    ));
                }
                if !effective_alpm.required_by.is_empty() {
                    root = root.child(Self::chip_section(
                        &format!("REQUIS PAR ({})", effective_alpm.required_by.len()),
                        &effective_alpm.required_by,
                        theme,
                        theme.text_muted,
                        props.on_navigate_dep,
                    ));
                }
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

                meta_grid = meta_grid.child(Self::meta_row("Dépôt", "AUR", theme));
                if let Some(ref maintainer) = aur.maintainer {
                    meta_grid = meta_grid.child(Self::meta_row("Mainteneur", maintainer, theme));
                }
                if let Some(ref url) = aur.url {
                    meta_grid = meta_grid.child(Self::meta_row("Site amont", url, theme));
                }
                if let Some(pop) = aur.popularity {
                    meta_grid = meta_grid.child(Self::meta_row(
                        "Popularité",
                        &format!("{:.2}", pop),
                        theme,
                    ));
                }
                if let Some(votes) = aur.num_votes {
                    meta_grid = meta_grid.child(Self::meta_row("Votes", &votes.to_string(), theme));
                }

                root = root.child(meta_grid);

                if let Some(ref deps) = aur.depends {
                    if !deps.is_empty() {
                        root = root.child(Self::chip_section(
                            &format!("DÉPENDANCES AUR ({})", deps.len()),
                            deps,
                            theme,
                            theme.text_secondary,
                            props.on_navigate_dep.clone(),
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
                            props.on_navigate_dep,
                        ));
                    }
                }
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

                if let Some(ref app_id) = fp.app_id {
                    meta_grid =
                        meta_grid.child(Self::meta_row("Identifiant Application", app_id, theme));
                }
                if let Some(ref remote) = fp.remote {
                    meta_grid = meta_grid.child(Self::meta_row("Remote Flatpak", remote, theme));
                }
                if let Some(ref app_type) = fp.app_type {
                    meta_grid = meta_grid.child(Self::meta_row("Type", app_type, theme));
                }
                if let Some(ref size) = fp.installed_size {
                    let formatted_size = format!("{:.1} Mo", *size as f64 / (1024.0 * 1024.0));
                    meta_grid =
                        meta_grid.child(Self::meta_row("Taille installée", &formatted_size, theme));
                }

                root = root.child(meta_grid);
            }
            UnifiedPackageSource::AppImage(ai) => {
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

                if let Some(ref dname) = ai.desktop_name {
                    meta_grid = meta_grid.child(Self::meta_row("Desktop Name", dname, theme));
                }
                if let Some(ref path) = ai.path {
                    meta_grid = meta_grid.child(Self::meta_row("File Path", path, theme));
                }
                if let Some(ref size) = ai.size_on_disk {
                    let formatted_size = format!("{:.1} Mo", *size as f64 / (1024.0 * 1024.0));
                    meta_grid =
                        meta_grid.child(Self::meta_row("Size on Disk", &formatted_size, theme));
                }
                if let Some(ref url) = ai.update_url {
                    meta_grid = meta_grid.child(Self::meta_row("Update URL", url, theme));
                }

                root = root.child(meta_grid);
            }
        }

        root.into_any_element()
    }

    fn meta_row(label: &str, value: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .text_xs()
            .child(div().text_color(theme.text_muted).child(label.to_string()))
            .child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_primary)
                    .child(value.to_string()),
            )
    }

    fn chip_section(
        title: &str,
        items: &[String],
        theme: &Theme,
        chip_text_color: Rgba,
        on_navigate: Option<Rc<dyn Fn(String, &mut Window, &mut App) + 'static>>,
    ) -> impl IntoElement {
        let section = div().flex().flex_col().gap_2().mb_5().child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.text_muted)
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
                    let clean_name = item_clone.split_whitespace().next().unwrap_or(&item_clone);
                    cb(clean_name.to_string(), window, cx);
                });
            }

            chips = chips.child(chip);
        }
        section.child(chips)
    }
}

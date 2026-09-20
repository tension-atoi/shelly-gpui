use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::components::status_pill::StatusPill;
use crate::theme::Theme;
use gpui::*;

use std::rc::Rc;

pub type MouseClickHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

pub struct PackageDetailsProps<'a> {
    pub package: Option<&'a UnifiedPackage>,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub on_install: Option<MouseClickHandler>,
    pub on_remove: Option<MouseClickHandler>,
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

        // En-tête du paquet
        let header = div()
            .flex()
            .flex_col()
            .mb_6()
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

        // Barre d'actions (Installer / Désinstaller / MAJ)
        let mut action_bar = div()
            .flex()
            .items_center()
            .gap_3()
            .mb_6();

        if pkg.is_installed {
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.danger })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
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
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.accent })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
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
            let btn = div()
                .px_4()
                .py_2()
                .rounded_md()
                .bg(if props.is_busy { theme.border } else { theme.warning })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if props.is_busy { theme.text_muted } else { theme.bg_app })
                .cursor_pointer()
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

        // Métadonnées spécifiques selon la source
        match &pkg.inner {
            UnifiedPackageSource::Standard(alpm) => {
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

                if let Some(ref url) = alpm.url {
                    meta_grid = meta_grid.child(Self::meta_row("Site amont", url, theme));
                }
                if let Some(size) = alpm.installed_size {
                    let mb = (size as f64) / (1024.0 * 1024.0);
                    meta_grid = meta_grid.child(Self::meta_row("Taille installée", &format!("{:.2} Mo", mb), theme));
                }
                if !alpm.licenses.is_empty() {
                    meta_grid = meta_grid.child(Self::meta_row("Licence", &alpm.licenses.join(", "), theme));
                }
                if let Some(ref date) = alpm.build_date {
                    meta_grid = meta_grid.child(Self::meta_row("Date de compilation", date, theme));
                }

                root = root.child(meta_grid);

                // Dépendances
                if !alpm.depends.is_empty() {
                    let mut deps_div = div()
                        .flex()
                        .flex_col()
                        .mb_6()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_muted)
                                .mb_2()
                                .child(format!("DÉPENDANCES ({})", alpm.depends.len())),
                        );

                    let mut chips = div().flex().flex_wrap().gap_2();
                    for dep in alpm.depends.iter().take(24) {
                        chips = chips.child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .text_color(theme.text_secondary)
                                .child(dep.clone()),
                        );
                    }
                    deps_div = deps_div.child(chips);
                    root = root.child(deps_div);
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

                if let Some(ref path) = ai.path {
                    meta_grid = meta_grid.child(Self::meta_row("Chemin fichier", path, theme));
                }

                root = root.child(meta_grid);
            }
        }

        root.into_any_element()
    }

    fn meta_row(label: &'static str, val: &str, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .text_xs()
            .child(div().text_color(theme.text_muted).child(label))
            .child(div().text_color(theme.text_primary).font_weight(FontWeight::MEDIUM).child(val.to_string()))
    }
}

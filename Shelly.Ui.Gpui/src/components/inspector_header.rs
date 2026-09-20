use crate::backend::models::UnifiedPackage;
use crate::components::status_pill::StatusPill;
use crate::icons::AppIcon;
use crate::state::{canonical_install_command, InspectorTab, PackageCapabilities};
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type MouseClickHandler = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TabSelectHandler = Rc<dyn Fn(InspectorTab, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct InspectorHeaderProps<'a> {
    pub package: &'a UnifiedPackage,
    pub capabilities: &'a PackageCapabilities,
    pub active_tab: InspectorTab,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub copy_feedback: bool,
    pub on_select_tab: TabSelectHandler,
    pub on_install: Option<MouseClickHandler>,
    pub on_remove: Option<MouseClickHandler>,
    pub on_copy_install_cmd: Option<WindowActionHandler>,
}

pub struct InspectorHeader;

impl InspectorHeader {
    fn render_tab_item(
        tab: InspectorTab,
        active_tab: InspectorTab,
        theme: &Theme,
        on_select: TabSelectHandler,
    ) -> impl IntoElement {
        let is_active = tab == active_tab;
        let hover_text = theme.text_primary;

        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px_3()
            .py_2()
            .cursor_pointer()
            .text_sm()
            .font_weight(if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(if is_active {
                theme.accent
            } else {
                theme.text_secondary
            })
            .border_b_2()
            .border_color(if is_active {
                theme.accent
            } else {
                rgba(0x00000000)
            })
            .when(!is_active, move |el| {
                el.hover(move |s| s.text_color(hover_text))
            })
            .child(
                svg()
                    .path(tab.icon().path())
                    .size_3_5()
                    .text_color(if is_active {
                        theme.accent
                    } else {
                        theme.text_secondary
                    }),
            )
            .child(tab.label())
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(tab, window, cx);
            })
    }

    pub fn render(props: InspectorHeaderProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let caps = props.capabilities;
        let is_busy = props.is_busy;

        // 1. Hero title & badges
        let title_row = div().flex().items_center().justify_between().mb_2().child(
            div()
                .flex()
                .items_center()
                .gap_3()
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
        );

        let mut badges_row = div().flex().items_center().gap_2().mb_4();
        badges_row = badges_row.child(StatusPill::source_badge(&pkg.source_type, theme));
        badges_row = badges_row.child(StatusPill::installed_pill(pkg.is_installed, theme));

        if pkg.has_update {
            badges_row = badges_row.child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(theme.accent)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.bg_app)
                    .child(if let Some(ref nv) = pkg.new_version {
                        format!("Update to {}", nv)
                    } else {
                        "Update available".to_string()
                    }),
            );
        }

        if !pkg.repository_or_remote.is_empty() {
            badges_row = badges_row.child(
                div()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(pkg.repository_or_remote.clone()),
            );
        }

        // 2. Action buttons row
        let mut actions_row = div().flex().items_center().gap_3().mb_4();

        if caps.can_remove {
            let danger_hover = theme.danger_hover;
            let remove_btn = div()
                .px_4()
                .py_1p5()
                .rounded_md()
                .bg(if is_busy { theme.border } else { theme.danger })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                })
                .cursor_pointer()
                .when(!is_busy, move |el| el.hover(move |s| s.bg(danger_hover)))
                .child(if is_busy {
                    "In progress..."
                } else {
                    "Uninstall"
                });

            let remove_btn = if !is_busy {
                if let Some(on_remove) = props.on_remove.clone() {
                    remove_btn.on_mouse_down(MouseButton::Left, move |e, w, cx| on_remove(e, w, cx))
                } else {
                    remove_btn
                }
            } else {
                remove_btn
            };
            actions_row = actions_row.child(remove_btn);
        } else if caps.can_install {
            let accent_hover = theme.accent_hover;
            let install_btn = div()
                .px_4()
                .py_1p5()
                .rounded_md()
                .bg(if is_busy { theme.border } else { theme.accent })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                })
                .cursor_pointer()
                .when(!is_busy, move |el| el.hover(move |s| s.bg(accent_hover)))
                .child(if is_busy { "In progress..." } else { "Install" });

            let install_btn = if !is_busy {
                if let Some(on_install) = props.on_install.clone() {
                    install_btn
                        .on_mouse_down(MouseButton::Left, move |e, w, cx| on_install(e, w, cx))
                } else {
                    install_btn
                }
            } else {
                install_btn
            };
            actions_row = actions_row.child(install_btn);
        }

        // Copy install command button
        if canonical_install_command(pkg).is_some() {
            let on_copy = props.on_copy_install_cmd.clone();
            let copy_btn = div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(if props.copy_feedback {
                    theme.success
                } else {
                    theme.border
                })
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if props.copy_feedback {
                    theme.success
                } else {
                    theme.text_secondary
                })
                .cursor_pointer()
                .hover({
                    let h = theme.bg_surface_hover;
                    move |s| s.bg(h)
                })
                .child(
                    svg()
                        .path(if props.copy_feedback {
                            AppIcon::Check.path()
                        } else {
                            AppIcon::Copy.path()
                        })
                        .size_3_5()
                        .text_color(if props.copy_feedback {
                            theme.success
                        } else {
                            theme.text_secondary
                        }),
                )
                .child(if props.copy_feedback {
                    "Copied command!"
                } else {
                    "Copy install command"
                })
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    if let Some(ref cb) = on_copy {
                        cb(window, cx);
                    }
                });

            actions_row = actions_row.child(copy_btn);
        }

        // 3. Tab bar navigation
        let on_tab = props.on_select_tab.clone();
        let tab_bar = div()
            .flex()
            .items_center()
            .gap_4()
            .border_b_1()
            .border_color(theme.border)
            .child(Self::render_tab_item(
                InspectorTab::Overview,
                props.active_tab,
                theme,
                on_tab.clone(),
            ))
            .child(Self::render_tab_item(
                InspectorTab::Dependencies,
                props.active_tab,
                theme,
                on_tab.clone(),
            ))
            .child(Self::render_tab_item(
                InspectorTab::FilesBuild,
                props.active_tab,
                theme,
                on_tab,
            ));

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(title_row)
            .child(badges_row)
            .child(actions_row)
            .child(tab_bar)
    }
}

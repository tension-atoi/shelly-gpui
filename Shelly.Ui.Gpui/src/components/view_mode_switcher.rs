use crate::icons::AppIcon;
use crate::state::session::PackageViewMode;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;

pub struct ViewModeSwitcherProps<'a> {
    pub current_mode: PackageViewMode,
    pub theme: &'a Theme,
    pub on_select_mode: ViewModeHandler,
}

pub struct ViewModeSwitcher;

impl ViewModeSwitcher {
    pub fn render(props: ViewModeSwitcherProps) -> impl IntoElement {
        let theme = props.theme;
        let is_cards = props.current_mode == PackageViewMode::Cards;
        let is_table = props.current_mode == PackageViewMode::Table;
        let on_cards = props.on_select_mode.clone();
        let on_table = props.on_select_mode.clone();

        div()
            .flex()
            .items_center()
            .bg(theme.bg_app)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .p(px(2.0))
            .gap(px(2.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(if is_cards {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .bg(if is_cards {
                        theme.bg_surface_active
                    } else {
                        theme.bg_app
                    })
                    .text_color(if is_cards {
                        theme.accent
                    } else {
                        theme.text_muted
                    })
                    .hover(move |s| s.text_color(theme.text_primary))
                    .child(
                        svg()
                            .path(AppIcon::Cards.path())
                            .size_3()
                            .text_color(if is_cards {
                                theme.accent
                            } else {
                                theme.text_muted
                            }),
                    )
                    .child("Cards")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_cards(PackageViewMode::Cards, window, cx);
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(if is_table {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .bg(if is_table {
                        theme.bg_surface_active
                    } else {
                        theme.bg_app
                    })
                    .text_color(if is_table {
                        theme.accent
                    } else {
                        theme.text_muted
                    })
                    .hover(move |s| s.text_color(theme.text_primary))
                    .child(
                        svg()
                            .path(AppIcon::Table.path())
                            .size_3()
                            .text_color(if is_table {
                                theme.accent
                            } else {
                                theme.text_muted
                            }),
                    )
                    .child("Table")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_table(PackageViewMode::Table, window, cx);
                    }),
            )
    }
}

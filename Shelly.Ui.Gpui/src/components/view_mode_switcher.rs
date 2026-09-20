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
        let modes = [PackageViewMode::Cards, PackageViewMode::Table];

        let mut container = div()
            .flex()
            .items_center()
            .bg(theme.bg_app)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .p(px(2.0))
            .gap(px(2.0));

        for mode in modes {
            let is_active = props.current_mode == mode;
            let on_select = props.on_select_mode.clone();
            let on_select_key = props.on_select_mode.clone();
            let element_id = match mode {
                PackageViewMode::Cards => "view_mode_cards",
                PackageViewMode::Table => "view_mode_table",
            };
            let focus_border = theme.border_focus;
            container = container.child(
                div()
                    .id(element_id)
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_select_key(mode, window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(if is_active {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .bg(if is_active {
                        theme.bg_surface_active
                    } else {
                        theme.bg_app
                    })
                    .text_color(if is_active {
                        theme.accent
                    } else {
                        theme.text_muted
                    })
                    .hover(move |s| s.text_color(theme.text_primary))
                    .child(
                        svg()
                            .path(mode.icon().path())
                            .size_3()
                            .text_color(if is_active {
                                theme.accent
                            } else {
                                theme.text_muted
                            }),
                    )
                    .child(mode.label())
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_select(mode, window, cx);
                    }),
            );
        }

        container
    }
}

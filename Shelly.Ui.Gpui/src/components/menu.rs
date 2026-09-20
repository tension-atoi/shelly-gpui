use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type MenuActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Conteneur de surface de menu popup (ancré, différé, bloquant et fermable au clic extérieur)
pub struct MenuSurface;

impl MenuSurface {
    pub fn render(
        id: ElementId,
        theme: &Theme,
        min_width: Pixels,
        reduce_motion: bool,
        on_close: MenuActionHandler,
        children: Vec<AnyElement>,
    ) -> impl IntoElement {
        let on_close_out = on_close.clone();
        let on_close_key = on_close.clone();

        let base = div()
            .id(id)
            .occlude()
            .key_context("MenuSurface")
            .focusable()
            .tab_stop(true)
            .on_mouse_down_out(move |_ev, window, cx| {
                on_close_out(window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key.as_str() == "escape" {
                    on_close_key(window, cx);
                }
            })
            .flex()
            .flex_col()
            .min_w(min_width)
            .max_w(px(340.0))
            .p_1p5()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .shadow_lg()
            .children(children);

        if reduce_motion {
            base.into_any_element()
        } else {
            base.with_animation(
                ("menu_surface_anim", 0usize),
                Animation::new(crate::state::MotionDurations::FAST)
                    .with_easing(gpui::ease_out_quint()),
                |el, delta| el.opacity(delta),
            )
            .into_any_element()
        }
    }
}

/// Séparateur horizontal au sein d'un menu
pub struct MenuDivider;

impl MenuDivider {
    pub fn render(theme: &Theme) -> impl IntoElement {
        div().h(px(1.0)).my(px(4.0)).bg(theme.border)
    }
}

/// En-tête de section au sein d'un menu complexe
pub struct MenuSection;

impl MenuSection {
    pub fn render(title: &'static str, theme: &Theme) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.text_muted)
            .child(title)
    }
}

/// Élément de sélection exclusive type radio (ex: filtre d'état de paquet)
pub struct MenuRadioItem;

impl MenuRadioItem {
    pub fn render(
        id: ElementId,
        label: impl Into<SharedString>,
        is_selected: bool,
        theme: &Theme,
        on_select: MenuActionHandler,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        let border_focus = theme.border_focus;
        let on_key_select = on_select.clone();

        div()
            .id(id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.bg(hover_bg).border_color(border_focus))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_key_select(window, cx);
                }
            })
            .flex()
            .items_center()
            .gap_2()
            .px_2p5()
            .py_1p5()
            .rounded_sm()
            .cursor_pointer()
            .text_xs()
            .text_color(if is_selected {
                theme.text_primary
            } else {
                theme.text_secondary
            })
            .font_weight(if is_selected {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_3()
                    .rounded_full()
                    .border_1()
                    .border_color(if is_selected {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .child(if is_selected {
                        div().size(px(6.0)).rounded_full().bg(theme.accent)
                    } else {
                        div().size(px(6.0))
                    }),
            )
            .child(label.into())
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(window, cx);
            })
    }
}

/// Propriétés d'un élément de sélection multiple de menu
pub struct MenuCheckItemProps<'a> {
    pub id: ElementId,
    pub label: SharedString,
    pub is_checked: bool,
    pub health_label: Option<&'static str>,
    pub is_failed: bool,
    pub theme: &'a Theme,
    pub on_toggle: MenuActionHandler,
    pub on_retry: Option<MenuActionHandler>,
}

/// Élément de sélection multiple type case à cocher avec état de santé et action Retry optionnelle
pub struct MenuCheckItem;

impl MenuCheckItem {
    pub fn render(props: MenuCheckItemProps) -> impl IntoElement {
        let theme = props.theme;
        let hover_bg = theme.bg_surface_hover;
        let border_focus = theme.border_focus;
        let on_key_toggle = props.on_toggle.clone();

        let retry_btn = if let Some(ref retry_cb) = props.on_retry {
            let cb = retry_cb.clone();
            let retry_hover = theme.accent_hover;
            Some(
                div()
                    .id(ElementId::NamedInteger("menu_retry_btn".into(), 0))
                    .px_1p5()
                    .py(px(2.0))
                    .rounded_sm()
                    .bg(Rgba {
                        a: 0.15,
                        ..theme.accent
                    })
                    .border_1()
                    .border_color(theme.accent)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .cursor_pointer()
                    .hover(move |s| s.bg(retry_hover).text_color(theme.bg_app))
                    .child("Retry")
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        cb(window, cx);
                    }),
            )
        } else {
            None
        };

        div()
            .id(props.id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.bg(hover_bg).border_color(border_focus))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_key_toggle(window, cx);
                }
            })
            .flex()
            .items_center()
            .justify_between()
            .px_2p5()
            .py_1p5()
            .rounded_sm()
            .cursor_pointer()
            .text_xs()
            .text_color(theme.text_primary)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size_3p5()
                            .rounded_xs()
                            .border_1()
                            .border_color(if props.is_checked {
                                theme.accent
                            } else {
                                theme.border
                            })
                            .bg(if props.is_checked {
                                theme.accent
                            } else {
                                theme.bg_surface
                            })
                            .child(if props.is_checked {
                                svg()
                                    .path(AppIcon::Check.path())
                                    .size(px(10.0))
                                    .text_color(theme.bg_app)
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }),
                    )
                    .child(props.label),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .children(props.health_label.map(|lbl| {
                        div()
                            .text_xs()
                            .text_color(if props.is_failed {
                                theme.danger
                            } else {
                                theme.text_muted
                            })
                            .child(lbl)
                    }))
                    .children(retry_btn),
            )
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                (props.on_toggle)(window, cx);
            })
    }
}

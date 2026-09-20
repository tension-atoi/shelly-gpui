use crate::models::NavRoute;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub struct NavRailProps<'a> {
    pub active_route: NavRoute,
    pub is_collapsed: bool,
    pub update_count: usize,
    pub theme: &'a Theme,
    pub hud_active: bool,
    pub on_select_route: Rc<dyn Fn(NavRoute, &mut Window, &mut App) + 'static>,
    pub on_toggle_sidebar: Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>,
    pub on_toggle_hud: Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>,
}

pub struct NavRail;

impl NavRail {
    fn render_nav_item(
        route: NavRoute,
        badge_count: usize,
        active_route: NavRoute,
        is_collapsed: bool,
        theme: &Theme,
        on_select: Rc<dyn Fn(NavRoute, &mut Window, &mut App) + 'static>,
    ) -> impl IntoElement {
        let is_active = active_route == route;
        let base = div()
            .flex()
            .items_center()
            .cursor_pointer()
            .rounded_md()
            .h(px(38.0));

        let styled_item = if is_active {
            base.bg(theme.bg_surface_active)
                .text_color(theme.accent)
                .font_weight(FontWeight::SEMIBOLD)
        } else {
            let hover_bg = theme.bg_surface_hover;
            let active_text = theme.text_primary;
            base.text_color(theme.text_secondary)
                .hover(move |s| s.bg(hover_bg).text_color(active_text))
        };

        if is_collapsed {
            styled_item
                .justify_center()
                .w(px(40.0))
                .mx(px(8.0))
                .my(px(4.0))
                .child(
                    div()
                        .relative()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(div().text_size(px(16.0)).child(route.icon()))
                        .when(badge_count > 0, |parent| {
                            let warning_color = theme.warning;
                            let app_bg = theme.bg_app;
                            parent.child(
                                div()
                                    .absolute()
                                    .top(px(-4.0))
                                    .right(px(-6.0))
                                    .bg(warning_color)
                                    .text_color(app_bg)
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .rounded_full()
                                    .px(px(4.0))
                                    .py(px(1.0))
                                    .child(format!("{}", badge_count)),
                            )
                        }),
                )
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    on_select(route, window, cx);
                })
        } else {
            styled_item
                .px(px(12.0))
                .mx(px(8.0))
                .my(px(2.0))
                .gap(px(12.0))
                .child(div().text_size(px(16.0)).child(route.icon()))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.0))
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(route.label()),
                )
                .when(badge_count > 0, |parent| {
                    let warning_color = theme.warning;
                    let app_bg = theme.bg_app;
                    parent.child(
                        div()
                            .bg(warning_color)
                            .text_color(app_bg)
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .rounded_full()
                            .px(px(6.0))
                            .py(px(1.0))
                            .child(format!("{}", badge_count)),
                    )
                })
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    on_select(route, window, cx);
                })
        }
    }

    pub fn render(props: NavRailProps) -> impl IntoElement {
        let is_collapsed = props.is_collapsed;
        let rail_width = if is_collapsed { px(56.0) } else { px(200.0) };
        let theme = props.theme;

        let on_toggle_sidebar = props.on_toggle_sidebar.clone();
        let on_toggle_hud = props.on_toggle_hud.clone();
        let hud_active = props.hud_active;

        div()
            .flex()
            .flex_col()
            .justify_between()
            .w(rail_width)
            .h_full()
            .bg(theme.bg_sidebar)
            .border_r_1()
            .border_color(theme.border)
            // Header: Brand / Logo
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .h(px(52.0))
                            .flex()
                            .items_center()
                            .border_b_1()
                            .border_color(theme.border)
                            .when(is_collapsed, |d| {
                                d.justify_center().child(
                                    div()
                                        .text_size(px(22.0))
                                        .child("🐚"),
                                )
                            })
                            .when(!is_collapsed, |d| {
                                d.px(px(14.0)).gap(px(10.0)).child(
                                    div()
                                        .text_size(px(22.0))
                                        .child("🐚"),
                                ).child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(14.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.accent)
                                                .child("SHELLY"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(theme.text_muted)
                                                .child("GPUI Station"),
                                        ),
                                )
                            }),
                    )
                    // Nav Items Group
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .pt(px(12.0))
                            .child(Self::render_nav_item(
                                NavRoute::Search,
                                0,
                                props.active_route,
                                is_collapsed,
                                theme,
                                props.on_select_route.clone(),
                            ))
                            .child(Self::render_nav_item(
                                NavRoute::Updates,
                                props.update_count,
                                props.active_route,
                                is_collapsed,
                                theme,
                                props.on_select_route.clone(),
                            ))
                            .child(Self::render_nav_item(
                                NavRoute::Installed,
                                0,
                                props.active_route,
                                is_collapsed,
                                theme,
                                props.on_select_route.clone(),
                            ))
                            .child(Self::render_nav_item(
                                NavRoute::Settings,
                                0,
                                props.active_route,
                                is_collapsed,
                                theme,
                                props.on_select_route.clone(),
                            )),
                    ),
            )
            // Footer: Performance HUD Toggle & Expand/Collapse Rail
            .child(
                div()
                    .flex()
                    .flex_col()
                    .border_t_1()
                    .border_color(theme.border)
                    .py(px(8.0))
                    // Performance HUD Button
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .rounded_md()
                            .mx(px(8.0))
                            .my(px(2.0))
                            .h(px(32.0))
                            .when(hud_active, |d| {
                                d.bg(theme.bg_surface_active)
                                    .text_color(theme.accent)
                            })
                            .when(!hud_active, |d| {
                                let hover_bg = theme.bg_surface_hover;
                                let hover_text = theme.text_secondary;
                                d.text_color(theme.text_muted)
                                    .hover(move |s| s.bg(hover_bg).text_color(hover_text))
                            })
                            .when(is_collapsed, |d| {
                                d.justify_center().w(px(40.0)).child(
                                    div().text_size(px(14.0)).child("⚡"),
                                )
                            })
                            .when(!is_collapsed, |d| {
                                d.px(px(12.0)).gap(px(8.0)).child(
                                    div().text_size(px(14.0)).child("⚡"),
                                ).child(
                                    div().text_size(px(11.0)).child("HUD (Ctrl+Shift+D)"),
                                )
                            })
                            .on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                                on_toggle_hud(ev, window, cx);
                            }),
                    )
                    // Sidebar Expand / Collapse Affordance
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .rounded_md()
                            .mx(px(8.0))
                            .my(px(2.0))
                            .h(px(32.0))
                            .text_color(theme.text_muted)
                            .hover({
                                let hover_bg = theme.bg_surface_hover;
                                let hover_text = theme.text_primary;
                                move |s| s.bg(hover_bg).text_color(hover_text)
                            })
                            .when(is_collapsed, |d| {
                                d.justify_center().w(px(40.0)).child(
                                    div().text_size(px(14.0)).child("»"),
                                )
                            })
                            .when(!is_collapsed, |d| {
                                d.px(px(12.0)).gap(px(8.0)).child(
                                    div().text_size(px(14.0)).child("«"),
                                ).child(
                                    div().text_size(px(11.0)).child("Réduire le volet"),
                                )
                            })
                            .on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                                on_toggle_sidebar(ev, window, cx);
                            }),
                    ),
            )
    }
}

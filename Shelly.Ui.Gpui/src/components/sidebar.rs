use crate::components::status_pill::StatusPill;
use crate::state::NavDestination;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type NavDestinationHandler = Rc<dyn Fn(NavDestination, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct SidebarProps<'a> {
    pub active_destination: NavDestination,
    pub updates_count: usize,
    pub is_collapsed: bool,
    pub theme: &'a Theme,
    pub on_select_destination: NavDestinationHandler,
    pub on_toggle_collapse: WindowActionHandler,
}

pub struct Sidebar;

impl Sidebar {
    fn render_nav_item(
        dest: NavDestination,
        active_dest: NavDestination,
        badge_count: usize,
        is_collapsed: bool,
        theme: &Theme,
        on_select: NavDestinationHandler,
    ) -> impl IntoElement {
        let is_active = active_dest == dest;
        let icon = dest.icon();
        let label = dest.label();

        let base = div()
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(if is_collapsed { px(12.0) } else { px(14.0) })
            .py(px(8.0))
            .mx(px(6.0))
            .rounded_md()
            .cursor_pointer()
            .text_sm();

        let styled = if is_active {
            base.bg(theme.bg_surface_active)
                .text_color(theme.text_primary)
                .font_weight(FontWeight::SEMIBOLD)
                .border_l_2()
                .border_color(theme.accent)
        } else {
            let hover_bg = theme.bg_surface_hover;
            base.bg(theme.bg_sidebar)
                .text_color(theme.text_secondary)
                .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
        };

        let mut item = styled.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .text_base()
                .child(icon),
        );

        if !is_collapsed {
            item = item.child(div().flex_1().child(label));

            if badge_count > 0 {
                item = item.child(StatusPill::badge_count(badge_count, theme));
            }
        }

        item.on_mouse_down(MouseButton::Left, move |_e, window, cx| {
            on_select(dest, window, cx);
        })
    }

    pub fn render(props: SidebarProps) -> impl IntoElement {
        let theme = props.theme;
        let is_collapsed = props.is_collapsed;
        let width = if is_collapsed { px(56.0) } else { px(190.0) };

        let destinations = [
            NavDestination::Browse,
            NavDestination::Installed,
            NavDestination::Updates,
            NavDestination::News,
            NavDestination::Settings,
        ];

        let mut nav_items = div().flex().flex_col().gap(px(2.0)).py(px(8.0));

        for dest in destinations {
            let badge = if dest == NavDestination::Updates {
                props.updates_count
            } else {
                0
            };

            nav_items = nav_items.child(Self::render_nav_item(
                dest,
                props.active_destination,
                badge,
                is_collapsed,
                theme,
                props.on_select_destination.clone(),
            ));
        }

        let collapse_toggle = {
            let on_toggle = props.on_toggle_collapse.clone();
            let hover_bg = theme.bg_surface_hover;
            div()
                .flex()
                .items_center()
                .justify_center()
                .py(px(8.0))
                .mx(px(6.0))
                .rounded_md()
                .cursor_pointer()
                .text_xs()
                .text_color(theme.text_muted)
                .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
                .child(if is_collapsed { "▶" } else { "◀ Collapse" })
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    on_toggle(window, cx);
                })
        };

        div()
            .flex()
            .flex_col()
            .justify_between()
            .w(width)
            .h_full()
            .bg(theme.bg_sidebar)
            .border_r_1()
            .border_color(theme.border)
            .py(px(6.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    // Header de la barre latérale
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .px(if is_collapsed { px(14.0) } else { px(16.0) })
                            .py(px(10.0))
                            .border_b_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.accent)
                                    .child("⚡"),
                            )
                            .child(if !is_collapsed {
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .child("SHELLY GPUI")
                            } else {
                                div()
                            }),
                    )
                    .child(nav_items),
            )
            .child(collapse_toggle)
    }
}

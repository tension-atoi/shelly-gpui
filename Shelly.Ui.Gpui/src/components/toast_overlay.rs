use crate::state::{Toast, ToastAction, ToastKind, ToastLifecycle};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ToastDismissHandler = Rc<dyn Fn(u64, &mut Window, &mut App) + 'static>;
pub type ToastActionHandler = Rc<dyn Fn(ToastAction, &mut Window, &mut App) + 'static>;

pub struct ToastOverlayProps<'a> {
    pub toasts: &'a [Toast],
    pub theme: &'a Theme,
    pub on_dismiss: ToastDismissHandler,
    pub on_action: ToastActionHandler,
}

pub struct ToastOverlay;

impl ToastOverlay {
    pub fn render(props: ToastOverlayProps) -> impl IntoElement {
        let theme = props.theme;
        let mut container = div()
            .id("toast_overlay_container")
            .absolute()
            .bottom(px(16.0))
            .right(px(16.0))
            .flex()
            .flex_col()
            .gap(px(8.0));

        for toast in props.toasts {
            let toast_id = toast.id;
            let on_dismiss = props.on_dismiss.clone();
            let on_action = props.on_action.clone();

            let border_color = match toast.kind {
                ToastKind::Info => theme.accent,
                ToastKind::Success => theme.success,
                ToastKind::Warning => theme.warning,
                ToastKind::Error => theme.danger,
            };

            let opacity = match toast.lifecycle {
                ToastLifecycle::Entering => 0.85,
                ToastLifecycle::Visible => 1.0,
                ToastLifecycle::Exiting => 0.0,
            };

            let mut toast_el = div()
                .id(ElementId::NamedInteger("toast-item".into(), toast.id))
                .flex()
                .flex_col()
                .w(px(320.0))
                .p_3()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(border_color)
                .opacity(opacity);

            // En-tête du toast (Icône, Titre, Bouton fermeture)
            let header = div()
                .flex()
                .items_center()
                .justify_between()
                .mb_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(border_color)
                                .child(toast.kind.icon()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(toast.title.clone()),
                        ),
                )
                .child(
                    div()
                        .cursor_pointer()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .hover(move |s| s.text_color(theme.text_primary))
                        .child("✕")
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            on_dismiss(toast_id, window, cx);
                        }),
                );

            toast_el = toast_el.child(header);

            // Message du toast
            toast_el = toast_el.child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(toast.message.clone()),
            );

            // Action optionnelle (ex: Ouvrir les logs)
            if let Some(action) = &toast.action {
                let act = action.clone();
                let action_btn = div().flex().justify_end().mt_2().child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(theme.bg_surface_active)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.accent)
                        .cursor_pointer()
                        .hover(move |s| s.bg(theme.bg_surface_hover))
                        .child("Voir les journaux")
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            on_action(act.clone(), window, cx);
                        }),
                );
                toast_el = toast_el.child(action_btn);
            }

            container = container.child(toast_el);
        }

        container
    }
}

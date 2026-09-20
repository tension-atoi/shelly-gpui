use crate::theme::Theme;
use gpui::*;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Idle,
    Running(String),
    Success(String),
    Error(String),
}

pub struct LogDrawerProps<'a> {
    pub logs: &'a [String],
    pub status: &'a OperationStatus,
    pub is_open: bool,
    pub theme: &'a Theme,
}

pub struct LogDrawer;

impl LogDrawer {
    pub fn render(props: LogDrawerProps) -> impl IntoElement {
        let theme = props.theme;
        let is_open = props.is_open;

        let status_badge = match props.status {
            OperationStatus::Idle => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.border)
                .text_xs()
                .text_color(theme.text_muted)
                .child("Prêt"),
            OperationStatus::Running(op) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.warning)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("En cours : {}", op)),
            OperationStatus::Success(msg) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.success)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("Succès : {}", msg)),
            OperationStatus::Error(err) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.danger)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("Erreur : {}", err)),
        };

        let mut container = div()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.bg_sidebar);

        // Header du tiroir
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_1p5()
            .bg(theme.bg_surface)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("TERMINAL & FLUX D'EXÉCUTION SHELLY"),
                    )
                    .child(status_badge),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(if is_open { "Cliquer pour masquer" } else { "Cliquer pour afficher les logs" }),
            );

        container = container.child(header);

        if is_open {
            let mut log_content = div()
                .id("log_drawer_scroll")
                .flex()
                .flex_col()
                .h(px(200.0))
                .overflow_scroll()
                .p_3()
                .bg(theme.bg_sidebar)
                .text_xs()
                .text_color(theme.text_secondary);

            if props.logs.is_empty() {
                log_content = log_content.child(
                    div()
                        .text_color(theme.text_muted)
                        .child("Aucun log d'opération en cours. Les sorties stdout/stderr de Shelly s'afficheront ici."),
                );
            } else {
                for line in props.logs.iter() {
                    let is_err = line.starts_with("error:") || line.starts_with("Erreur") || line.contains("failed");
                    let color = if is_err { theme.danger } else { theme.text_primary };
                    log_content = log_content.child(
                        div()
                            .py_0p5()
                            .text_color(color)
                            .child(line.clone()),
                    );
                }
            }

            container = container.child(log_content);
        }

        container
    }
}

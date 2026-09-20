use crate::backend::models::ArchNewsItem;
use crate::theme::Theme;
use gpui::*;

pub struct NewsViewProps<'a> {
    pub news: &'a [ArchNewsItem],
    pub theme: &'a Theme,
    pub is_loading: bool,
}

pub struct NewsView;

impl NewsView {
    pub fn render(props: NewsViewProps) -> impl IntoElement {
        let theme = props.theme;

        let mut root = div()
            .id("news_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        root = root.child(
            div()
                .flex()
                .flex_col()
                .mb_6()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Official Arch Linux News"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Important announcements, package updates, and recommended manual interventions."),
                ),
        );

        if props.is_loading {
            return root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .text_color(theme.text_muted)
                    .child("Loading Arch Linux news..."),
            );
        }

        if props.news.is_empty() {
            return root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .text_color(theme.text_muted)
                    .child("No recent announcements to display."),
            );
        }

        for item in props.news.iter() {
            let card = div()
                .flex()
                .flex_col()
                .p_4()
                .mb_4()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .mb_2()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_base()
                                .text_color(theme.text_primary)
                                .child(item.title.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.accent)
                                .child(item.published_date.clone().unwrap_or_default()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .mb_2()
                        .child(if let Some(ref author) = item.author {
                            format!("Published by: {}", author)
                        } else {
                            "Arch Linux Team".to_string()
                        }),
                )
                .child(div().text_sm().text_color(theme.text_primary).child(
                    item.summary.clone().unwrap_or_else(|| {
                        "Click to read the full announcement on archlinux.org".to_string()
                    }),
                ));

            root = root.child(card);
        }

        root
    }
}

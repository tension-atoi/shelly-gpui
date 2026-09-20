use crate::backend::models::UnifiedPackage;
use crate::components::status_pill::StatusPill;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Version,
    Source,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct PackageTableProps<'a> {
    pub packages: &'a [UnifiedPackage],
    pub selected_package: Option<&'a UnifiedPackage>,
    pub theme: &'a Theme,
    pub scroll_handle: UniformListScrollHandle,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub on_select_package: Rc<dyn Fn(&UnifiedPackage, &mut Window, &mut App) + 'static>,
    pub on_change_sort: Rc<dyn Fn(SortColumn, &mut Window, &mut App) + 'static>,
}

pub struct PackageTable;

impl PackageTable {
    fn render_header_cell(
        column: SortColumn,
        label: &'static str,
        width_fraction: f32,
        active_sort: SortColumn,
        sort_dir: SortDirection,
        theme: &Theme,
        on_sort: Rc<dyn Fn(SortColumn, &mut Window, &mut App) + 'static>,
    ) -> impl IntoElement {
        let is_sorted = active_sort == column;
        let arrow = if is_sorted {
            match sort_dir {
                SortDirection::Ascending => " ▲",
                SortDirection::Descending => " ▼",
            }
        } else {
            ""
        };

        div()
            .flex()
            .items_center()
            .w(relative(width_fraction))
            .px_2()
            .py_1()
            .cursor_pointer()
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(if is_sorted { theme.accent } else { theme.text_muted })
            .hover({
                let hover_text = theme.text_primary;
                move |s| s.text_color(hover_text)
            })
            .child(format!("{}{}", label, arrow))
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_sort(column, window, cx);
            })
    }

    pub fn render(props: PackageTableProps) -> impl IntoElement {
        let theme = props.theme;
        let packages = props.packages.to_vec();
        let selected_name = props.selected_package.map(|p| p.name.clone());
        let on_select = props.on_select_package.clone();
        let on_sort = props.on_change_sort.clone();

        let header = div()
            .h(px(32.0))
            .w_full()
            .flex()
            .items_center()
            .bg(theme.bg_surface_hover)
            .border_b_1()
            .border_color(theme.border)
            .child(Self::render_header_cell(
                SortColumn::Name,
                "Nom",
                0.28,
                props.sort_column,
                props.sort_direction,
                theme,
                on_sort.clone(),
            ))
            .child(Self::render_header_cell(
                SortColumn::Version,
                "Version",
                0.18,
                props.sort_column,
                props.sort_direction,
                theme,
                on_sort.clone(),
            ))
            .child(Self::render_header_cell(
                SortColumn::Source,
                "Source",
                0.16,
                props.sort_column,
                props.sort_direction,
                theme,
                on_sort.clone(),
            ))
            .child(
                div()
                    .w(relative(0.24))
                    .px_2()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("Description"),
            )
            .child(Self::render_header_cell(
                SortColumn::Status,
                "État",
                0.14,
                props.sort_column,
                props.sort_direction,
                theme,
                on_sort,
            ));

        let package_count = packages.len();
        let list_theme = *theme;
        let scroll_handle = props.scroll_handle.clone();

        let body = uniform_list("package-table-rows", package_count, {
            let packages = packages.clone();
            let selected_name = selected_name.clone();
            let on_select = on_select.clone();

            move |range, _window, _cx| {
                range
                    .map(|idx| {
                        let pkg = &packages[idx];
                        let is_selected = selected_name
                            .as_ref()
                            .map(|n| n == &pkg.name)
                            .unwrap_or(false);

                        let row_bg = if is_selected {
                            list_theme.bg_surface_active
                        } else if idx % 2 == 1 {
                            list_theme.bg_surface_hover
                        } else {
                            list_theme.bg_surface
                        };

                        let pkg_clone = pkg.clone();
                        let on_click = on_select.clone();

                        div()
                            .h(px(36.0))
                            .w_full()
                            .flex()
                            .items_center()
                            .bg(row_bg)
                            .border_b_1()
                            .border_color(list_theme.border)
                            .cursor_pointer()
                            .when(is_selected, |d| {
                                d.border_l_4().border_color(list_theme.accent)
                            })
                            .when(!is_selected, |d| {
                                let hbg = list_theme.bg_surface_hover;
                                d.hover(move |s| s.bg(hbg))
                            })
                            // Col 1: Name
                            .child(
                                div()
                                    .w(relative(0.28))
                                    .px_2()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .w(px(6.0))
                                            .h(px(6.0))
                                            .rounded_full()
                                            .bg(if pkg.is_installed {
                                                list_theme.success
                                            } else {
                                                list_theme.text_muted
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(if is_selected {
                                                list_theme.accent
                                            } else {
                                                list_theme.text_primary
                                            })
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .child(pkg.name.clone()),
                                    ),
                            )
                            // Col 2: Version
                            .child(
                                div()
                                    .w(relative(0.18))
                                    .px_2()
                                    .text_xs()
                                    .text_color(list_theme.text_muted)
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(pkg.version.clone()),
                            )
                            // Col 3: Source Badge
                            .child(
                                div()
                                    .w(relative(0.16))
                                    .px_2()
                                    .flex()
                                    .items_center()
                                    .child(StatusPill::source_badge(&pkg.source_type, &list_theme)),
                            )
                            // Col 4: Description
                            .child(
                                div()
                                    .w(relative(0.24))
                                    .px_2()
                                    .text_xs()
                                    .text_color(list_theme.text_secondary)
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(if pkg.description.is_empty() {
                                        "—".to_string()
                                    } else {
                                        pkg.description.clone()
                                    }),
                            )
                            // Col 5: Status Pill
                            .child(
                                div()
                                    .w(relative(0.14))
                                    .px_2()
                                    .flex()
                                    .items_center()
                                    .child(StatusPill::installed_pill(pkg.is_installed, &list_theme)),
                            )
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                on_click(&pkg_clone, window, cx);
                            })
                    })
                    .collect()
            }
        })
        .track_scroll(scroll_handle);

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_hidden()
            .child(header)
            .child(div().flex_1().overflow_hidden().child(body))
    }
}

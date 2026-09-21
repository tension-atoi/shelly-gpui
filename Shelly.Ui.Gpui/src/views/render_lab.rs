use crate::render_lab::catalog::FixtureCatalog;
use crate::render_lab::fixture::{FixtureGroup, FixtureId};
use crate::render_lab::state::{ClockMode, RenderLabState};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type SelectFixtureHandler = Rc<dyn Fn(FixtureId, &mut Window, &mut App) + 'static>;

pub struct RenderLabViewProps<'a> {
    pub state: &'a RenderLabState,
    pub theme: &'a Theme,
    pub on_select_fixture: SelectFixtureHandler,
}

pub struct RenderLabView;

impl RenderLabView {
    pub fn render(props: RenderLabViewProps) -> impl IntoElement {
        let theme = *props.theme;
        let active_id = props.state.active_fixture.as_ref();
        let selected_def = active_id.and_then(FixtureCatalog::find);

        div()
            .id("render_lab_root")
            .size_full()
            .flex()
            .flex_row()
            .bg(theme.bg_app)
            .text_color(theme.text_primary)
            // ── Column 1: Catalog Column (260px) ───────────────────────────
            .child(Self::render_catalog(&props, &theme, active_id))
            // ── Column 2: Preview / Fixture Specification (flex-1) ─────────
            .child(Self::render_preview(&theme, selected_def))
            // ── Column 3: State Ledger & Diagnostics (280px) ───────────────
            .child(Self::render_inspector(props.state, &theme))
    }

    fn render_catalog(
        props: &RenderLabViewProps,
        theme: &Theme,
        active_id: Option<&FixtureId>,
    ) -> impl IntoElement {
        let mut groups_col = div()
            .id("render_lab_catalog")
            .w(px(260.0))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .overflow_scroll();

        // Header
        groups_col = groups_col.child(
            div()
                .p_4()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child("Fixture Catalog"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("{} canonical fixtures", FixtureCatalog::count())),
                ),
        );

        for group in FixtureGroup::ALL {
            let fixtures: Vec<_> = FixtureCatalog::for_group(*group).collect();

            let mut group_section = div().flex().flex_col().py_2();

            // Group header
            group_section = group_section.child(
                div()
                    .px_4()
                    .py_1()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_secondary)
                            .child(group.label()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_muted)
                            .child(format!("Board {}", group.board_code())),
                    ),
            );

            // Fixture rows
            for f in fixtures {
                let is_selected = active_id == Some(&f.id);
                let on_select = props.on_select_fixture.clone();
                let f_id = f.id.clone();

                let row_bg = if is_selected {
                    theme.bg_surface_active
                } else {
                    theme.bg_surface
                };

                let row_text_color = if is_selected {
                    theme.accent
                } else {
                    theme.text_primary
                };

                let display_title = f.provenance.source_label.unwrap_or_else(|| f.id.as_str());

                let row = div()
                    .id(SharedString::from(format!("cat_row_{}", f.id.as_str())))
                    .px_4()
                    .py_1p5()
                    .mx_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(row_bg)
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        on_select(f_id.clone(), window, cx);
                    })
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if is_selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(row_text_color)
                            .child(display_title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("#{} · seed {}", f.provenance.cell_index, f.seed)),
                    );

                group_section = group_section.child(row);
            }

            groups_col = groups_col.child(group_section);
        }

        groups_col
    }

    fn render_preview(
        theme: &Theme,
        selected_def: Option<&'static crate::render_lab::fixture::FixtureDef>,
    ) -> impl IntoElement {
        let mut preview = div()
            .id("render_lab_preview")
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_scroll()
            .p_6();

        match selected_def {
            Some(f) => {
                // Header
                preview = preview.child(
                    div()
                        .mb_6()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight::BOLD)
                                        .child(f.id.as_str()),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(theme.bg_surface_hover)
                                        .border_1()
                                        .border_color(theme.border)
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child(format!("Board {}", f.provenance.board)),
                                ),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .child(if !f.description.is_empty() {
                                    f.description
                                } else {
                                    "Unlabeled source microstructure swatch"
                                }),
                        ),
                );

                // Placeholder Preview Box
                preview = preview.child(
                    div()
                        .w_full()
                        .h(px(320.0))
                        .rounded_lg()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.bg_surface)
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .p_6()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_secondary)
                                .child("RENDER-00 Canonical Preview Slot"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .max_w(px(400.0))
                                .text_center()
                                .child("No visual cleverness before the catalog, determinism model, capability vocabulary, and control surface are canonical. Primitive confrontation begins in RENDER-01."),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .border_1()
                                .border_color(theme.border)
                                .bg(theme.bg_app)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.warning_text)
                                .child(format!("Capability: {}", f.capability.as_str())),
                        ),
                );

                // Provenance Card
                preview = preview.child(
                    div()
                        .mt_6()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.bg_surface)
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_secondary)
                                .child("PROVENANCE & INTEGRITY"),
                        )
                        .child(Self::meta_row(
                            theme,
                            "Source Board",
                            &format!(
                                "Board {} (Cell {})",
                                f.provenance.board, f.provenance.cell_index
                            ),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Source Label",
                            f.provenance.source_label.unwrap_or("None (unlabeled)"),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Artifact",
                            f.provenance.source_artifact,
                        ))
                        .child(Self::meta_row(theme, "SHA-256", f.provenance.source_sha256))
                        .child(Self::meta_row(theme, "Seed", &format!("{}", f.seed)))
                        .child(Self::meta_row(theme, "Determinism", "Deterministic")),
                );
            }
            None => {
                preview = preview.child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .child("Shelly Visual Render Lab"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("Select a fixture from the catalog to inspect its specification and provenance."),
                        ),
                );
            }
        }

        preview
    }

    fn render_inspector(state: &RenderLabState, theme: &Theme) -> impl IntoElement {
        let (clock_str, clock_detail) = match state.clock {
            ClockMode::Realtime => ("Realtime", "Running at wall clock".to_string()),
            ClockMode::Frozen(t) => ("Frozen", format!("t = {t:.3}s")),
        };

        div()
            .id("render_lab_inspector")
            .w(px(280.0))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .overflow_scroll()
            .p_4()
            .child(
                div()
                    .mb_4()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child("Render Lab Ledger"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child("Read-only control state (CLI-driven)"),
                    ),
            )
            // State Axes Section
            .child(
                div()
                    .p_3()
                    .rounded_md()
                    .bg(theme.bg_app)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_secondary)
                            .child("STATE AXES"),
                    )
                    .child(Self::meta_row(
                        theme,
                        "Topology",
                        state.active_topology.as_str(),
                    ))
                    .child(Self::meta_row(
                        theme,
                        "Motion",
                        state.active_motion.as_str(),
                    ))
                    .child(Self::meta_row(
                        theme,
                        "Quality",
                        state.active_quality.as_str(),
                    ))
                    .child(Self::meta_row(theme, "Clock Mode", clock_str))
                    .child(Self::meta_row(theme, "Clock Time", &clock_detail)),
            )
            // Diagnostics
            .child(
                div()
                    .mt_4()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_secondary)
                            .mb_2()
                            .child("DIAGNOSTICS"),
                    )
                    .child({
                        let mut diag_col = div().flex().flex_col().gap_1();
                        for d in &state.diagnostics {
                            diag_col = diag_col.child(
                                div()
                                    .p_2()
                                    .rounded_sm()
                                    .bg(theme.bg_app)
                                    .border_1()
                                    .border_color(theme.border)
                                    .text_xs()
                                    .child(d.message.clone()),
                            );
                        }
                        diag_col
                    }),
            )
    }

    fn meta_row(theme: &Theme, label: &'static str, val: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_start()
            .gap_2()
            .text_xs()
            .child(
                div()
                    .text_color(theme.text_muted)
                    .flex_shrink_0()
                    .child(label),
            )
            .child(
                div()
                    .text_color(theme.text_primary)
                    .font_weight(FontWeight::MEDIUM)
                    .text_right()
                    .child(val.to_string()),
            )
    }
}

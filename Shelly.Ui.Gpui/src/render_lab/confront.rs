use crate::render_lab::fixture::FixtureDef;
use crate::render_lab::recipe::{RecipeCatalog, RecipeDef};
use crate::render_lab::texture::{self, TextureKind};
use gpui::*;

/// Fixed preview-box height: the confrontation surface is always 340px
/// tall, full column width. ROI width is session-dependent and recorded
/// per capture; ROI height is structurally constant.
pub const CONFRONT_BOX_HEIGHT_PX: f32 = 340.0;

/// Fiducial marker color framing the confrontation box. The determinism
/// harness locates this exact color in window captures to derive the
/// fixed preview ROI; the marker itself is excluded from pixel hashing.
pub const FIDUCIAL_MARKER_RGB: u32 = 0xFF00FF;

/// Renders the stock confrontation for a fixture, or `None` when no
/// recipe exists yet (unevaluated slot). Content fills the parent box.
pub fn render_confrontation(def: &FixtureDef) -> Option<AnyElement> {
    let recipe = RecipeCatalog::find(def.id.as_str())?;
    Some(match recipe.id {
        // ── Group G: Fields ──────────────────────────────────────────
        "g01-linear-scalar-gradient/r1" => render_zero_positive_scalar().into_any_element(),
        "g02-bipolar-split/r1" => render_bipolar_split().into_any_element(),
        "g03-magnitude-contour-stops/r1" => render_current_magnitude().into_any_element(),
        "g04-streamline-vector-field/r1" => render_current_direction().into_any_element(),
        "g05-thermal-multistop-gradient/r1" => render_overload_heat().into_any_element(),
        "g06-probabilistic-confidence-band/r1" => render_diagnostic_confidence().into_any_element(),
        "g07-stress-concentration-contour/r1" => render_component_stress().into_any_element(),
        "g08-continuous-density-plane/r1" => render_selection_density().into_any_element(),

        // ── Group H: Depth ───────────────────────────────────────────
        "h01-single-box-shadow/r1" => render_contact_shadow().into_any_element(),
        "h02-diffused-elevation-shadow/r1" => render_component_lift_shadow().into_any_element(),
        "h03-cavity-occlusion-bevel/r1" => render_recessed_socket_shadow().into_any_element(),
        "h04-inset-panel-directional-bevel/r1" => {
            render_inset_panel_inner_shadow().into_any_element()
        }
        "h05-dual-tone-linear-bevel/r1" => {
            render_raised_instrument_subtle_bevel().into_any_element()
        }
        "h06-ambient-top-highlight-rim/r1" => render_soft_edge_top_highlight().into_any_element(),
        "h07-continuous-perimeter-rim/r1" => render_rim_highlight_outline().into_any_element(),
        "h08-prismatic-shallow-bevel/r1" => render_shallow_bevel_3d_edge().into_any_element(),

        // ── Group I: Optical ─────────────────────────────────────────
        "i01-concentric-falloff/r1" => render_concentric_falloff().into_any_element(),
        "i02-linear-directional-fade/r1" => render_directional_fade().into_any_element(),
        "i03-soft-rounded-box-glow/r1" => render_soft_rectangle_rounded().into_any_element(),
        "i04-radial-disc-glow-boundary/r1" => render_soft_circle_glow().into_any_element(),
        "i05-corner-darkening-vignette/r1" => render_edge_vignette().into_any_element(),
        "i06-central-luminance-boost/r1" => render_local_focus().into_any_element(),
        "i07-filament-core-bloom/r1" => render_energized_wire_glow().into_any_element(),
        "i08-thermal-emission-envelope/r1" => render_heat_region_glow().into_any_element(),

        // ── Group J: Materials ───────────────────────────────────────
        "j01-matte-anthracite-coating/r1" => render_matte_anthracite().into_any_element(),
        "j02-signal-orange-coating/r1" => render_signal_orange().into_any_element(),
        "j03-beret-green-coating/r1" => render_beret_green().into_any_element(),
        "j04-royal-blue-coating/r1" => render_royal_blue().into_any_element(),
        "j05-brushed-sphere-texture/r1" => {
            render_texture_fill(TextureKind::BrushedCell, 4005).into_any_element()
        }
        "j06-dark-anodized-satin/r1" => render_dark_anodized().into_any_element(),
        "j07-warm-paper-fibrous-texture/r1" => {
            render_texture_fill(TextureKind::WarmPaper, 4007).into_any_element()
        }
        "j08-smoked-polycarbonate-translucent/r1" => render_smoked_plastic().into_any_element(),

        // ── Group K: Microstructure ──────────────────────────────────
        "k01-uniform-noise-texture/r1" => {
            render_texture_fill(TextureKind::UniformNoise, 5001).into_any_element()
        }
        "k02-stratified-jitter-texture/r1" => {
            render_texture_fill(TextureKind::StratifiedJitter, 5002).into_any_element()
        }
        "k03-fine-grit-texture/r1" => {
            render_texture_fill(TextureKind::FineGrit, 5003).into_any_element()
        }
        "k04-brushed-micro-texture/r1" => {
            render_texture_fill(TextureKind::BrushedMicro, 5004).into_any_element()
        }
        "k05-anisotropic-grain-texture/r1" => {
            render_texture_fill(TextureKind::AnisotropicGrain, 5005).into_any_element()
        }
        "k06-cellular-pattern-texture/r1" => {
            render_texture_fill(TextureKind::Cellular, 5006).into_any_element()
        }
        "k07-stochastic-stipple-texture/r1" => {
            render_texture_fill(TextureKind::Stipple, 5007).into_any_element()
        }
        "k08-value-noise-lattice-texture/r1" => {
            render_texture_fill(TextureKind::ValueNoiseLattice, 5008).into_any_element()
        }
        "k09-halftone-mesh-texture/r1" => {
            render_texture_fill(TextureKind::HalftoneMesh, 5009).into_any_element()
        }
        "k10-woven-matrix-texture/r1" => {
            render_texture_fill(TextureKind::WovenMatrix, 5010).into_any_element()
        }
        "k11-crater-relief-texture/r1" => {
            render_texture_fill(TextureKind::CraterRelief, 5011).into_any_element()
        }
        "k12-etched-fiber-texture/r1" => {
            render_texture_fill(TextureKind::EtchedFiber, 5012).into_any_element()
        }
        "k13-fine-dither-texture/r1" => {
            render_texture_fill(TextureKind::FineDither, 5013).into_any_element()
        }
        "k14-coarse-grain-texture/r1" => {
            render_texture_fill(TextureKind::CoarseGrain, 5014).into_any_element()
        }

        _ => return None,
    })
}

pub fn recipe_for(def: &FixtureDef) -> Option<&'static RecipeDef> {
    RecipeCatalog::find(def.id.as_str())
}

// ── Helpers ─────────────────────────────────────────────────────────

fn render_texture_fill(kind: TextureKind, seed: u64) -> impl IntoElement {
    img(texture::texture_for(kind, seed, 256, 256))
        .w_full()
        .h_full()
        .object_fit(ObjectFit::Fill)
}

// ── Group G: Fields ──────────────────────────────────────────────────

/// G01: Zero-to-positive scalar gradient
fn render_zero_positive_scalar() -> impl IntoElement {
    div().size_full().bg(linear_gradient(
        90.0,
        linear_color_stop(rgb(0x0e1b24), 0.0),
        linear_color_stop(rgb(0x20a894), 1.0),
    ))
}

/// G02: Signed voltage bipolar domain
fn render_bipolar_split() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_row()
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x2e56c7), 0.0),
            linear_color_stop(rgb(0xa79e99), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0xa79e99), 0.0),
            linear_color_stop(rgb(0xfc7108), 1.0),
        )))
}

/// G03: Current magnitude: stepped contour bands
fn render_current_magnitude() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_row()
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x130e28), 0.0),
            linear_color_stop(rgb(0x281b54), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x362772), 0.0),
            linear_color_stop(rgb(0x5638a2), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x256cb0), 0.0),
            linear_color_stop(rgb(0x2cb3d4), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x5ae4d8), 0.0),
            linear_color_stop(rgb(0xe0ffff), 1.0),
        )))
}

/// G04: Current direction: streamline vector lines between two poles
fn render_current_direction() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x12171d))
        .flex()
        .items_center()
        .justify_center()
        .child(
            canvas(
                |_bounds, _window, _cx| {},
                |bounds, _, window, _cx| {
                    let w: f32 = bounds.size.width.into();
                    let h: f32 = bounds.size.height.into();
                    let ox: f32 = bounds.origin.x.into();
                    let oy: f32 = bounds.origin.y.into();
                    for line in 0..16 {
                        let mut u = -0.44f32;
                        let mut v = (line as f32 / 15.0 - 0.5) * 0.76;
                        let mut pts = Vec::with_capacity(120);
                        for _ in 0..120 {
                            pts.push((ox + w * (u * 0.44 + 0.5), oy + h * (v * 0.60 + 0.5)));
                            let r1 = ((u + 0.52).powi(2) + v * v + 0.015).powf(1.5);
                            let r2 = ((u - 0.52).powi(2) + v * v + 0.015).powf(1.5);
                            let ex = (u + 0.52) / r1 - (u - 0.52) / r2;
                            let ey = v / r1 - v / r2;
                            let len = (ex * ex + ey * ey).sqrt().max(0.01);
                            u += 0.018 * ex / len;
                            v += 0.018 * ey / len;
                            if u.abs() > 0.95 || v.abs() > 0.72 {
                                break;
                            }
                        }
                        let mut p = PathBuilder::stroke(px(1.2));
                        if let Some(&(x, y)) = pts.first() {
                            p.move_to(point(px(x), px(y)));
                        }
                        for &(x, y) in pts.iter().skip(1) {
                            p.line_to(point(px(x), px(y)));
                        }
                        if let Ok(built) = p.build() {
                            window.paint_path(built, hsla(196.0 / 360.0, 0.75, 0.65, 0.75));
                        }
                    }
                },
            )
            .size_full(),
        )
}

/// G05: Overload heat: thermal black-body distribution via 3 composed segments
fn render_overload_heat() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_row()
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x180004), 0.0),
            linear_color_stop(rgb(0x820c08), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x820c08), 0.0),
            linear_color_stop(rgb(0xf26419), 1.0),
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0xf26419), 0.0),
            linear_color_stop(rgb(0xffe169), 1.0),
        )))
}

/// G06: Diagnostic confidence band: amber uncertainty to emerald certainty
fn render_diagnostic_confidence() -> impl IntoElement {
    div().size_full().bg(linear_gradient(
        90.0,
        linear_color_stop(rgb(0x52442b), 0.0),
        linear_color_stop(rgb(0x2fe098), 1.0),
    ))
}

/// G07: Component stress: multi-layer stress concentration zones
fn render_component_stress() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x101b2b))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(240.0))
                .rounded_lg()
                .bg(rgb(0x18446b))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(px(180.0))
                        .rounded_lg()
                        .bg(rgb(0x28846c))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(120.0))
                                .rounded_lg()
                                .bg(rgb(0xd4a024))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(div().size(px(60.0)).rounded_lg().bg(rgb(0xd92b2b))),
                        ),
                ),
        )
}

/// G08: Selection density: cross-fading bilinear density plane
fn render_selection_density() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x11161d))
        .relative()
        .child(div().size_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgba(0x0a84ff00), 0.0),
            linear_color_stop(rgba(0x0a84ffcc), 1.0),
        )))
        .child(div().size_full().absolute().bg(linear_gradient(
            180.0,
            linear_color_stop(rgba(0x00000000), 0.0),
            linear_color_stop(rgba(0x000000bb), 1.0),
        )))
}

// ── Group H: Depth ───────────────────────────────────────────────────

/// H01: Contact shadow: tight dark occlusion
fn render_contact_shadow() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xf2f2f2))
        .child(
            div()
                .size(px(120.0))
                .rounded_full()
                .bg(rgb(0xbdbdbd))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.60),
                    offset: point(px(0.0), px(4.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// H02: Component lift shadow: elevated soft penumbra
fn render_component_lift_shadow() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xe8ecef))
        .child(
            div()
                .w(px(220.0))
                .h(px(140.0))
                .rounded_xl()
                .bg(rgb(0xffffff))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.22),
                    offset: point(px(0.0), px(14.0)),
                    blur_radius: px(28.0),
                    spread_radius: px(2.0),
                }]),
        )
}

/// H03: Recessed socket cavity shadow
fn render_recessed_socket_shadow() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xdfe3e6))
        .child(
            div()
                .size(px(150.0))
                .rounded_full()
                .border_2()
                .border_color(rgba(0x00000066))
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0x8a9299), 0.0),
                    linear_color_stop(rgb(0xadb5bd), 1.0),
                ))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .size(px(110.0))
                        .rounded_full()
                        .bg(rgb(0x2b3036))
                        .border_t_2()
                        .border_color(rgba(0x000000aa)),
                ),
        )
}

/// H04: Inset panel directional inner shadow
fn render_inset_panel_inner_shadow() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xd4d8dc))
        .child(
            div()
                .w(px(260.0))
                .h(px(160.0))
                .rounded_lg()
                .bg(rgb(0xbcc2c8))
                .border_t_2()
                .border_color(rgba(0x00000077))
                .border_b_1()
                .border_color(rgba(0xffffffcc))
                .child(div().w_full().h(px(24.0)).bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgba(0x00000055), 0.0),
                    linear_color_stop(rgba(0x00000000), 1.0),
                ))),
        )
}

/// H05: Raised instrument body subtle bevel
fn render_raised_instrument_subtle_bevel() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0x282c34))
        .child(
            div()
                .w(px(220.0))
                .h(px(130.0))
                .rounded_md()
                .bg(rgb(0x3a404a))
                .border_t_1()
                .border_color(rgba(0xffffffaa))
                .border_b_1()
                .border_color(rgba(0x00000099))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.45),
                    offset: point(px(0.0), px(6.0)),
                    blur_radius: px(10.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// H06: Soft edge top highlight
fn render_soft_edge_top_highlight() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0x181b20))
        .child(
            div()
                .w(px(230.0))
                .h(px(140.0))
                .rounded_lg()
                .bg(rgb(0x252a32))
                .border_t_1()
                .border_color(rgba(0xffffff88)),
        )
}

/// H07: Continuous rim highlight outline
fn render_rim_highlight_outline() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0x16181d))
        .child(
            div()
                .size(px(140.0))
                .rounded_full()
                .bg(rgb(0x232730))
                .border_1()
                .border_color(rgba(0xffffff77)),
        )
}

/// H08: Shallow bevel 3D edge
fn render_shallow_bevel_3d_edge() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0x1c2026))
        .child(
            div()
                .w(px(220.0))
                .h(px(130.0))
                .rounded_lg()
                .p(px(3.0))
                .bg(linear_gradient(
                    135.0,
                    linear_color_stop(rgb(0x8a929c), 0.0),
                    linear_color_stop(rgb(0x1b1d22), 1.0),
                ))
                .child(
                    div()
                        .size_full()
                        .rounded_md()
                        .bg(linear_gradient(
                            180.0,
                            linear_color_stop(rgb(0x353a42), 0.0),
                            linear_color_stop(rgb(0x262930), 1.0),
                        ))
                        .border_t_1()
                        .border_color(rgba(0xffffff55)),
                ),
        )
}

// ── Group I: Optical ─────────────────────────────────────────────────

/// I01: Concentric radial falloff
fn render_concentric_falloff() -> impl IntoElement {
    const RINGS: usize = 32;
    const OUTER: f32 = 300.0;
    const INNER: f32 = 24.0;
    let mut nested: Option<Div> = None;
    for index in (0..RINGS).rev() {
        let t = index as f32 / (RINGS - 1) as f32;
        let diameter = INNER + (OUTER - INNER) * t;
        let alpha = 0.16 + 0.70 * (1.0 - t);
        let ring = div()
            .size(px(diameter))
            .rounded_full()
            .bg(rgba(0xffffff00 | ((alpha * 255.0) as u32)))
            .flex()
            .items_center()
            .justify_center();
        nested = Some(match nested {
            Some(inner) => ring.child(inner),
            None => ring,
        });
    }
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(0xc9c9c9))
        .child(nested.expect("concentric falloff always builds rings"))
}

/// I02: Directional opacity fade
fn render_directional_fade() -> impl IntoElement {
    div().size_full().bg(linear_gradient(
        90.0,
        linear_color_stop(rgb(0x0099e6), 0.0),
        linear_color_stop(rgba(0x0099e600), 1.0),
    ))
}

/// I03: Soft rectangle rounded glow
fn render_soft_rectangle_rounded() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x0b131c))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(200.0))
                .h(px(120.0))
                .rounded_xl()
                .bg(rgb(0x20a0d0))
                .shadow(vec![BoxShadow {
                    color: hsla(196.0 / 360.0, 0.85, 0.60, 0.65),
                    offset: point(px(0.0), px(0.0)),
                    blur_radius: px(34.0),
                    spread_radius: px(10.0),
                }]),
        )
}

/// I04: Soft circle radial disc glow
fn render_soft_circle_glow() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x0c1219))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(100.0))
                .rounded_full()
                .bg(rgb(0x60d8ff))
                .shadow(vec![
                    BoxShadow {
                        color: hsla(195.0 / 360.0, 0.95, 0.55, 0.70),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(45.0),
                        spread_radius: px(25.0),
                    },
                    BoxShadow {
                        color: hsla(195.0 / 360.0, 0.90, 0.75, 0.80),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(15.0),
                        spread_radius: px(5.0),
                    },
                ]),
        )
}

/// I05: Edge vignette
fn render_edge_vignette() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0xc0c8d0))
        .flex()
        .flex_col()
        .justify_between()
        .child(div().w_full().h(px(60.0)).bg(linear_gradient(
            180.0,
            linear_color_stop(rgba(0x00000066), 0.0),
            linear_color_stop(rgba(0x00000000), 1.0),
        )))
        .child(div().w_full().h(px(60.0)).bg(linear_gradient(
            0.0,
            linear_color_stop(rgba(0x00000066), 0.0),
            linear_color_stop(rgba(0x00000000), 1.0),
        )))
}

/// I06: Local focus center bright
fn render_local_focus() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x0e141c))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(140.0))
                .rounded_full()
                .bg(rgb(0xffffff))
                .shadow(vec![BoxShadow {
                    color: hsla(210.0 / 360.0, 0.80, 0.70, 0.55),
                    offset: point(px(0.0), px(0.0)),
                    blur_radius: px(60.0),
                    spread_radius: px(30.0),
                }]),
        )
}

/// I07: Energized wire core bloom
fn render_energized_wire_glow() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x140f0a))
        .flex()
        .items_center()
        .justify_center()
        .child(div().w_full().h(px(3.0)).bg(rgb(0xfff8d0)).shadow(vec![
            BoxShadow {
                color: hsla(38.0 / 360.0, 1.0, 0.55, 0.90),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(12.0),
                spread_radius: px(4.0),
            },
            BoxShadow {
                color: hsla(24.0 / 360.0, 1.0, 0.45, 0.65),
                offset: point(px(0.0), px(0.0)),
                blur_radius: px(35.0),
                spread_radius: px(12.0),
            },
        ]))
}

/// I08: Heat region thermal glow
fn render_heat_region_glow() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x120808))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .size(px(120.0))
                .rounded_full()
                .bg(rgb(0xffaa20))
                .shadow(vec![
                    BoxShadow {
                        color: hsla(16.0 / 360.0, 1.0, 0.50, 0.85),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(40.0),
                        spread_radius: px(20.0),
                    },
                    BoxShadow {
                        color: hsla(0.0 / 360.0, 1.0, 0.40, 0.55),
                        offset: point(px(0.0), px(0.0)),
                        blur_radius: px(80.0),
                        spread_radius: px(40.0),
                    },
                ]),
        )
}

// ── Group J: Materials ───────────────────────────────────────────────

/// J01: Matte anthracite coating
fn render_matte_anthracite() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x16181b))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_lg()
                .bg(rgb(0x22252a))
                .border_1()
                .border_color(rgb(0x2e3238))
                .border_t_1()
                .border_color(rgb(0x454b54))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.50),
                    offset: point(px(0.0), px(8.0)),
                    blur_radius: px(16.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// J02: Signal orange painted metal
fn render_signal_orange() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x1c1e22))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_lg()
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0xff6e2b), 0.0),
                    linear_color_stop(rgb(0xdb4c0b), 1.0),
                ))
                .border_t_1()
                .border_color(rgba(0xffffff99))
                .border_1()
                .border_color(rgb(0xb03800))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.45),
                    offset: point(px(0.0), px(8.0)),
                    blur_radius: px(16.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// J03: Beret green painted metal
fn render_beret_green() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x181a1c))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_lg()
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0x455648), 0.0),
                    linear_color_stop(rgb(0x313d33), 1.0),
                ))
                .border_t_1()
                .border_color(rgba(0xffffff66))
                .border_1()
                .border_color(rgb(0x232c25))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.45),
                    offset: point(px(0.0), px(8.0)),
                    blur_radius: px(16.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// J04: Royal blue painted metal
fn render_royal_blue() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x15181d))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_lg()
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0x255bb5), 0.0),
                    linear_color_stop(rgb(0x143c82), 1.0),
                ))
                .border_t_1()
                .border_color(rgba(0xffffff88))
                .border_1()
                .border_color(rgb(0x0e2959))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.50),
                    offset: point(px(0.0), px(8.0)),
                    blur_radius: px(16.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// J06: Dark anodized aluminum satin
fn render_dark_anodized() -> impl IntoElement {
    div()
        .size_full()
        .bg(rgb(0x181a1d))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_lg()
                .bg(linear_gradient(
                    180.0,
                    linear_color_stop(rgb(0x353a40), 0.0),
                    linear_color_stop(rgb(0x272b30), 1.0),
                ))
                .border_1()
                .border_color(rgb(0x40454c))
                .border_t_1()
                .border_color(rgba(0xffffff55))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.55),
                    offset: point(px(0.0), px(6.0)),
                    blur_radius: px(14.0),
                    spread_radius: px(0.0),
                }]),
        )
}

/// J08: Smoked polycarbonate translucent
fn render_smoked_plastic() -> impl IntoElement {
    div()
        .size_full()
        .bg(linear_gradient(
            45.0,
            linear_color_stop(rgb(0x2c3e50), 0.0),
            linear_color_stop(rgb(0xbdc3c7), 1.0),
        ))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(240.0))
                .h(px(150.0))
                .rounded_xl()
                .bg(rgba(0x14181ecc))
                .border_1()
                .border_color(rgba(0xffffff44))
                .border_t_1()
                .border_color(rgba(0xffffffaa))
                .shadow(vec![BoxShadow {
                    color: hsla(0.0, 0.0, 0.0, 0.40),
                    offset: point(px(0.0), px(12.0)),
                    blur_radius: px(24.0),
                    spread_radius: px(0.0),
                }]),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_confront_box_geometry_constants() {
        assert_eq!(CONFRONT_BOX_HEIGHT_PX, 340.0);
        assert_eq!(FIDUCIAL_MARKER_RGB, 0xFF00FF);
    }

    #[test]
    fn test_all_46_fixtures_have_recipes_and_confrontations() {
        let catalog = crate::render_lab::catalog::FixtureCatalog::all();
        assert_eq!(catalog.len(), 46);
        for def in catalog {
            assert!(
                recipe_for(def).is_some(),
                "missing recipe for fixture {}",
                def.id.as_str()
            );
            assert!(
                render_confrontation(def).is_some(),
                "missing confrontation renderer for fixture {}",
                def.id.as_str()
            );
        }
    }
}

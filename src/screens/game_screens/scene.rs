//! Shared visual furniture for the in-shift screens: the cab scene strip,
//! metric tiles, and the warning pulse.

use macroquad::prelude::*;

use crate::ui::draw_ui_text;
use crate::ui::{colors, draw_glass_panel, draw_small_caps, fonts, UiRect};
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01;

/// Get a pulsing color for warning text (slow, gentle pulse)
pub(super) fn pulsing_warning_color() -> Color {
    // 3 second full cycle (0% -> 100% -> 0%)
    let alpha = pulse01(2.0 * std::f32::consts::PI / 3.0);
    with_alpha(colors::ACCENT_WARNING, alpha)
}

pub(super) fn draw_bottom_taxi_scene(rect: UiRect) {
    // The painted cockpit now carries the environment. This restrained glass
    // strip keeps decision text separated from the dashboard without laying
    // a second, exterior taxi on top of the first-person view.
    draw_rectangle(rect.x, rect.y, rect.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.006, 0.008, 0.009, 0.16),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);
}

pub(super) fn draw_metric_tile(rect: UiRect, label: &str, value: &str, color: Color) {
    draw_glass_panel(rect, colors::BORDER_DIM);
    let inner = rect.inset(12.0);
    draw_small_caps(
        label,
        inner.x,
        inner.y + 12.0,
        fonts::SIZE_XS,
        colors::TEXT_MUTED,
    );
    draw_ui_text(value, inner.x, inner.y + 42.0, fonts::SIZE_LG, color);
}

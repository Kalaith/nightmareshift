//! Shared visual furniture for the in-shift screens: the cab scene strip,
//! metric tiles, and the warning pulse.

use macroquad::prelude::*;

use crate::ui::{colors, draw_glass_panel, draw_small_caps, fonts, UiRect};
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::draw_ui_text;

/// Get a pulsing color for warning text (slow, gentle pulse)
pub(super) fn pulsing_warning_color() -> Color {
    // 3 second full cycle (0% -> 100% -> 0%)
    let alpha = pulse01(2.0 * std::f32::consts::PI / 3.0);
    with_alpha(colors::ACCENT_WARNING, alpha)
}

pub(super) fn draw_bottom_taxi_scene(rect: UiRect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.006, 0.008, 0.009, 0.92),
    );
    draw_rectangle(rect.x, rect.y, rect.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);

    let road_y = rect.y + rect.h * 0.70;
    draw_rectangle(
        rect.x,
        road_y,
        rect.w,
        rect.h * 0.30,
        Color::new(0.020, 0.023, 0.022, 1.0),
    );
    for i in 0..9 {
        let x = rect.x + rect.w * (i as f32 / 9.0);
        draw_rectangle(
            x + rect.w * 0.035,
            road_y + rect.h * 0.16,
            rect.w * 0.045,
            3.0,
            Color::new(0.95, 0.58, 0.08, 0.30),
        );
    }

    for i in 0..5 {
        let x = rect.x + rect.w * (0.18 + i as f32 * 0.16);
        let lamp_y = rect.y + rect.h * (0.20 + (i % 2) as f32 * 0.06);
        draw_line(
            x,
            lamp_y + 24.0,
            x,
            road_y + 28.0,
            2.0,
            Color::new(0.25, 0.22, 0.18, 0.70),
        );
        draw_circle(x, lamp_y, 5.0, colors::CAB_YELLOW);
        draw_circle(x, lamp_y, 24.0, Color::new(0.95, 0.58, 0.08, 0.08));
    }

    let scale = (rect.w / 1100.0).clamp(0.85, 1.35);
    let body_w = 350.0 * scale;
    let body_h = 74.0 * scale;
    let body_x = rect.center_x() - body_w / 2.0;
    let body_y = rect.y + rect.h * 0.48;
    draw_rectangle(
        body_x,
        body_y,
        body_w,
        body_h,
        Color::new(0.74, 0.45, 0.06, 0.95),
    );
    draw_rectangle(
        body_x + body_w * 0.15,
        body_y - body_h * 0.44,
        body_w * 0.46,
        body_h * 0.44,
        Color::new(0.55, 0.34, 0.06, 0.92),
    );
    draw_rectangle(
        body_x + body_w * 0.21,
        body_y - body_h * 0.31,
        body_w * 0.16,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.88),
    );
    draw_rectangle(
        body_x + body_w * 0.40,
        body_y - body_h * 0.31,
        body_w * 0.17,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.88),
    );
    draw_rectangle(
        body_x + body_w * 0.27,
        body_y - body_h * 0.66,
        body_w * 0.22,
        body_h * 0.18,
        colors::CAB_YELLOW,
    );
    draw_ui_text(
        "TAXI",
        body_x + body_w * 0.32,
        body_y - body_h * 0.51,
        fonts::SIZE_SM * scale,
        colors::BLACK,
    );
    draw_rectangle(
        body_x + body_w * 0.03,
        body_y + body_h * 0.46,
        body_w * 0.14,
        body_h * 0.14,
        Color::new(0.95, 0.10, 0.04, 0.60),
    );
    draw_rectangle(
        body_x + body_w * 0.84,
        body_y + body_h * 0.45,
        body_w * 0.11,
        body_h * 0.12,
        Color::new(1.0, 0.86, 0.44, 0.70),
    );
    draw_circle(
        body_x + body_w * 0.17,
        body_y + body_h * 0.78,
        body_h * 0.28,
        colors::BLACK,
    );
    draw_circle(
        body_x + body_w * 0.82,
        body_y + body_h * 0.78,
        body_h * 0.28,
        colors::BLACK,
    );
    draw_circle(
        body_x + body_w * 0.17,
        body_y + body_h * 0.78,
        body_h * 0.13,
        Color::new(0.12, 0.13, 0.12, 1.0),
    );
    draw_circle(
        body_x + body_w * 0.82,
        body_y + body_h * 0.78,
        body_h * 0.13,
        Color::new(0.12, 0.13, 0.12, 1.0),
    );
    draw_rectangle(
        body_x - body_w * 0.10,
        body_y + body_h * 0.88,
        body_w * 1.20,
        6.0,
        Color::new(0.95, 0.58, 0.08, 0.18),
    );
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

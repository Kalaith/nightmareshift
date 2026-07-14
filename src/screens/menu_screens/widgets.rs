//! Main-menu chrome: the vector icon set and the icon+label command button.

use macroquad::prelude::*;

use crate::ui::{colors, draw_glass_button, draw_small_caps, fonts, UiRect};
use macroquad_toolkit::colors::with_alpha;

pub(super) fn draw_menu_icon(kind: &str, cx: f32, cy: f32, scale: f32, color: Color) {
    let s = scale;
    let stroke = (2.0 * s).max(1.0);
    match kind {
        "wheel" => {
            draw_circle_lines(cx, cy, 14.0 * s, stroke, color);
            draw_circle(cx, cy, 3.0 * s, color);
            draw_line(cx, cy, cx, cy - 12.0 * s, stroke, color);
            draw_line(cx, cy, cx - 11.0 * s, cy + 7.0 * s, stroke, color);
            draw_line(cx, cy, cx + 11.0 * s, cy + 7.0 * s, stroke, color);
        }
        "tree" => {
            draw_rectangle(cx - 3.0 * s, cy + 8.0 * s, 6.0 * s, 11.0 * s, color);
            draw_triangle(
                Vec2::new(cx, cy - 18.0 * s),
                Vec2::new(cx - 16.0 * s, cy + 4.0 * s),
                Vec2::new(cx + 16.0 * s, cy + 4.0 * s),
                with_alpha(color, 0.58),
            );
            draw_triangle(
                Vec2::new(cx, cy - 7.0 * s),
                Vec2::new(cx - 18.0 * s, cy + 14.0 * s),
                Vec2::new(cx + 18.0 * s, cy + 14.0 * s),
                with_alpha(color, 0.68),
            );
        }
        "book" => {
            draw_rectangle_lines(
                cx - 18.0 * s,
                cy - 14.0 * s,
                16.0 * s,
                28.0 * s,
                stroke,
                color,
            );
            draw_rectangle_lines(
                cx + 2.0 * s,
                cy - 14.0 * s,
                16.0 * s,
                28.0 * s,
                stroke,
                color,
            );
            draw_line(cx, cy - 13.0 * s, cx, cy + 15.0 * s, stroke, color);
        }
        "trophy" => {
            draw_rectangle_lines(
                cx - 10.0 * s,
                cy - 15.0 * s,
                20.0 * s,
                18.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx - 16.0 * s,
                cy - 11.0 * s,
                cx - 10.0 * s,
                cy - 6.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx + 16.0 * s,
                cy - 11.0 * s,
                cx + 10.0 * s,
                cy - 6.0 * s,
                stroke,
                color,
            );
            draw_line(cx, cy + 3.0 * s, cx, cy + 14.0 * s, stroke, color);
            draw_rectangle_lines(
                cx - 12.0 * s,
                cy + 14.0 * s,
                24.0 * s,
                6.0 * s,
                stroke,
                color,
            );
        }
        "delete" => {
            draw_rectangle_lines(
                cx - 12.0 * s,
                cy - 10.0 * s,
                24.0 * s,
                26.0 * s,
                stroke,
                color,
            );
            draw_rectangle(cx - 15.0 * s, cy - 16.0 * s, 30.0 * s, 4.0 * s, color);
            draw_line(
                cx - 5.0 * s,
                cy - 6.0 * s,
                cx - 5.0 * s,
                cy + 11.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx + 5.0 * s,
                cy - 6.0 * s,
                cx + 5.0 * s,
                cy + 11.0 * s,
                stroke,
                color,
            );
        }
        _ => {
            draw_circle_lines(cx, cy, 13.0 * s, stroke, color);
        }
    }
}
pub(super) fn draw_menu_command(
    rect: UiRect,
    icon: &str,
    label: &str,
    detail: &str,
    accent: Color,
    scale: f32,
) -> bool {
    let clicked = draw_glass_button(rect, "", accent, true);
    let icon_col_w = (60.0 * scale).clamp(42.0, 60.0);
    let label_size = (fonts::SIZE_LG * scale).clamp(13.0, fonts::SIZE_LG);
    let detail_size = (fonts::SIZE_XS * scale).clamp(8.0, fonts::SIZE_XS);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        icon_col_w,
        rect.h - 2.0,
        Color::new(0.0, 0.0, 0.0, 0.16),
    );
    draw_menu_icon(
        icon,
        rect.x + icon_col_w / 2.0,
        rect.y + rect.h / 2.0,
        scale,
        colors::TEXT_MUTED,
    );
    let text_x = rect.x + icon_col_w + 16.0 * scale;
    draw_small_caps(
        label,
        text_x,
        rect.y + rect.h * 0.46,
        label_size,
        colors::TEXT_PRIMARY,
    );
    draw_small_caps(
        detail,
        text_x,
        rect.y + rect.h * 0.78,
        detail_size,
        colors::TEXT_MUTED,
    );
    clicked
}

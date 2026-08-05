//! Small shared visual primitives and their deterministic review states.

use super::*;
use macroquad::prelude::*;

/// Shared modal scrim. All modal surfaces use the same opacity so the layer
/// relationship stays predictable and underlying decisions cannot compete.
pub fn draw_modal_scrim() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 200),
    );
}

/// Deterministic visual states used by the component verification scene.
/// Runtime buttons still derive hover and focus from real input.
#[derive(Debug, Clone, Copy)]
pub enum ButtonPreviewState {
    Default,
    Hovered,
    Focused,
    Disabled,
    Selected,
    Urgent,
}

pub fn draw_glass_button_preview(rect: UiRect, label: &str, state: ButtonPreviewState) {
    let (enabled, background, border, focused) = match state {
        ButtonPreviewState::Default => (true, colors::GLASS, colors::TEXT_SECONDARY, false),
        ButtonPreviewState::Hovered => (true, colors::GLASS_LIGHT, colors::ACCENT_SKY, false),
        ButtonPreviewState::Focused => (true, colors::GLASS, colors::ACCENT_SKY, true),
        ButtonPreviewState::Disabled => (false, colors::GLASS, colors::BORDER_DIM, false),
        ButtonPreviewState::Selected => (true, colors::GLASS_LIGHT, colors::FUEL_GOOD, false),
        ButtonPreviewState::Urgent => (true, colors::GLASS_LIGHT, colors::ACCENT_DANGER, false),
    };
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(background)
        .with_left_accent(4.0, border)
        .with_top_highlight(1.0, Color::new(1.0, 1.0, 1.0, 0.10))
        .with_border(1.0, border);
    macroquad_toolkit::ui::draw_surface(rect.rect(), &surface);
    if focused {
        draw_rectangle_lines(
            rect.x - 2.0,
            rect.y - 2.0,
            rect.w + 4.0,
            rect.h + 4.0,
            3.0,
            colors::TEXT_PRIMARY,
        );
    }
    let color = if enabled {
        colors::TEXT_PRIMARY
    } else {
        colors::TEXT_MUTED
    };
    let dims = measure_ui_text(label, None, fonts::SIZE_SM as u16, 1.0);
    draw_ui_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0 - 2.0,
        fonts::SIZE_SM,
        color,
    );
}

/// Compact semantic label. The text always accompanies its color.
pub fn draw_ui_badge(rect: UiRect, label: &str, accent: Color) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(colors::GLASS_LIGHT)
        .with_left_accent(3.0, accent)
        .with_border(1.0, accent);
    macroquad_toolkit::ui::draw_surface(rect.rect(), &surface);
    let dims = measure_ui_text(label, None, fonts::SIZE_XS as u16, 1.0);
    draw_small_caps(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0 - 2.0,
        fonts::SIZE_XS,
        accent,
    );
}

/// Labelled progress meter whose meaning never depends on color alone.
pub fn draw_ui_meter(rect: UiRect, fraction: f32, fill: Color, label: &str) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, colors::INK);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w * fraction.clamp(0.0, 1.0),
        rect.h,
        fill,
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER);
    let dims = measure_ui_text(label, None, fonts::SIZE_XS as u16, 1.0);
    draw_ui_text(
        label,
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0 - 2.0,
        fonts::SIZE_XS,
        colors::TEXT_PRIMARY,
    );
}

/// Shared tooltip card for short explanations attached to a decision.
pub fn draw_ui_tooltip(rect: UiRect, title: &str, body: &str) {
    draw_glass_panel(rect, colors::ACCENT_SKY);
    let inner = rect.inset(12.0);
    draw_small_caps(
        title,
        inner.x,
        inner.y + 12.0,
        fonts::SIZE_XS,
        colors::ACCENT_SKY,
    );
    draw_wrapped_text(
        body,
        inner.x,
        inner.y + 32.0,
        inner.w,
        fonts::SIZE_XS,
        15.0,
        colors::TEXT_SECONDARY,
        2,
    );
}

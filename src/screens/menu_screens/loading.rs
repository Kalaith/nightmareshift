//! The loading screen shown while game data is being read.

use macroquad::prelude::*;

use crate::data::GameData;
use crate::ui::{colors, draw_noir_city_background, UiAction};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Draw the loading screen
pub fn draw_loading(game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let text = if let Some(data) = game_data {
        &data.localization.ui.common.loading
    } else {
        "Loading..."
    };

    let font_size = 32.0;
    let text_width = measure_ui_text(text, None, font_size as u16, 1.0).width;
    draw_ui_text(
        text,
        screen_width() / 2.0 - text_width / 2.0,
        screen_height() / 2.0,
        font_size,
        colors::TEXT_PRIMARY,
    );
    if let Some(data) = game_data {
        let system_width = measure_ui_text(&data.localization.system.loading, None, 14, 1.0).width;
        draw_ui_text(
            &data.localization.system.loading,
            screen_width() / 2.0 - system_width / 2.0,
            screen_height() / 2.0 + 28.0,
            14.0,
            colors::TEXT_SECONDARY,
        );
        let meta_text = format!(
            "{} {} v{}",
            data.localization.meta.language,
            data.localization.meta.code,
            data.localization.meta.version
        );
        let meta_width = measure_ui_text(&meta_text, None, 14, 1.0).width;
        draw_ui_text(
            &meta_text,
            screen_width() / 2.0 - meta_width / 2.0,
            screen_height() / 2.0 + 48.0,
            14.0,
            colors::TEXT_MUTED,
        );
    }
    UiAction::None
}

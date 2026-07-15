//! End-of-night screens: the game over report and the success payout.

use macroquad::prelude::*;

use crate::data::GameData;
use crate::state::GameState;
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_wrapped_text,
    fonts, spacing, UiAction, UiRect,
};
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Draw the game over screen
pub fn draw_game_over(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        let panel = UiRect::centered_x(screen_width(), 112.0, screen_width().min(540.0), 330.0);
        draw_glass_panel(panel, colors::ACCENT_DANGER);
        let inner = panel.inset(spacing::PADDING_LG);

        let title = &data.localization.ui.game_over.title;
        let title_size = 46.0;
        let title_width = measure_ui_text(title, None, title_size as u16, 1.0).width;
        draw_ui_text(
            title,
            center_x - title_width / 2.0,
            inner.y + 50.0,
            title_size,
            colors::FUEL_CRITICAL,
        );

        if let Some(ref reason) = game_state.game_over_reason {
            draw_wrapped_text(
                reason,
                inner.x,
                inner.y + 92.0,
                inner.w,
                fonts::SIZE_MD,
                22.0,
                colors::TEXT_SECONDARY,
                3,
            );
        }

        let score = game_state.calculate_score(&data.constants);
        let stats = data
            .localization
            .ui
            .game_over
            .stats
            .replacen("{}", &game_state.earnings.to_string(), 1)
            .replacen("{}", &game_state.rides_completed.to_string(), 1)
            .replacen("{}", &score.to_string(), 1);

        draw_ui_text(
            &stats,
            inner.x,
            inner.y + 186.0,
            fonts::SIZE_MD,
            colors::TEXT_PRIMARY,
        );

        if draw_glass_button(
            UiRect::new(center_x - 150.0, panel.bottom() - 70.0, 300.0, 46.0),
            &data.localization.ui.common.try_again,
            colors::ACCENT_DANGER,
            true,
        ) {
            return UiAction::TryAgain;
        }
    }

    UiAction::None
}

/// Draw the success screen
pub fn draw_success(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        let panel = UiRect::centered_x(screen_width(), 104.0, screen_width().min(540.0), 350.0);
        draw_glass_panel(panel, colors::ACCENT_GOLD);
        let inner = panel.inset(spacing::PADDING_LG);

        // Interim nights and the final run-victory show different framing.
        let interim = !game_state.run_complete;
        let nights_left = data
            .constants
            .game_constants
            .nights_per_run
            .saturating_sub(game_state.night);
        let title = if interim {
            data.localization
                .ui
                .success
                .night_title
                .replace("{}", &game_state.night.to_string())
        } else {
            data.localization.ui.success.run_title.clone()
        };
        let title_size = 48.0;
        let title_width = measure_ui_text(&title, None, title_size as u16, 1.0).width;
        draw_ui_text(
            &title,
            center_x - title_width / 2.0,
            inner.y + 50.0,
            title_size,
            colors::ACCENT_GOLD,
        );

        let subtitle = if interim {
            data.localization
                .ui
                .success
                .night_subtitle
                .replace("{}", &nights_left.to_string())
        } else {
            data.localization.ui.success.run_subtitle.clone()
        };
        let sub_width = measure_ui_text(&subtitle, None, 24, 1.0).width;
        draw_ui_text(
            &subtitle,
            center_x - sub_width / 2.0,
            inner.y + 92.0,
            24.0,
            colors::ACCENT_PRIMARY,
        );

        let y = inner.y + 142.0;

        let earnings_text = data
            .localization
            .ui
            .success
            .total_earnings
            .replace("{}", &game_state.earnings.to_string());
        draw_ui_text(&earnings_text, inner.x, y, 20.0, colors::ACCENT_GOLD);

        let rides_text = data
            .localization
            .ui
            .success
            .rides_completed
            .replace("{}", &game_state.rides_completed.to_string());
        draw_ui_text(&rides_text, inner.x, y + 30.0, 20.0, WHITE);

        let bonus_text = data.localization.ui.success.survival_bonus.replace(
            "{}",
            &data.constants.game_constants.survival_bonus.to_string(),
        );
        draw_ui_text(&bonus_text, inner.x, y + 60.0, 20.0, colors::FUEL_GOOD);

        let score_text = data.localization.ui.success.final_score.replace(
            "{}",
            &game_state.calculate_score(&data.constants).to_string(),
        );
        draw_ui_text(&score_text, inner.x, y + 100.0, 24.0, colors::ACCENT_DANGER);

        if interim {
            // One button: press on into the next night.
            if draw_glass_button(
                UiRect::new(center_x - 110.0, panel.bottom() - 62.0, 220.0, 42.0),
                &data.localization.ui.success.next_night,
                colors::ACCENT_GOLD,
                true,
            ) {
                return UiAction::NextNight;
            }
        } else {
            // Run complete: start a fresh run or bank progress at the menu.
            if draw_glass_button(
                UiRect::new(center_x - 210.0, panel.bottom() - 62.0, 200.0, 42.0),
                &data.localization.ui.common.try_again,
                colors::ACCENT_GOLD,
                true,
            ) {
                return UiAction::TryAgain;
            }
            if draw_glass_button(
                UiRect::new(center_x + 10.0, panel.bottom() - 62.0, 200.0, 42.0),
                &data.localization.ui.common.back_button,
                colors::ACCENT_PRIMARY,
                true,
            ) {
                return UiAction::ReturnToMenu;
            }
        }
    }

    UiAction::None
}

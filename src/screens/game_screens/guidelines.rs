//! The guideline decision screen: timer, detected tells, and follow/break.

use macroquad::prelude::*;

use crate::data::{self, GameData};
use crate::state::GameState;
use crate::ui::{
    colors, draw_cockpit_background, draw_glass_button, draw_glass_panel, fonts, layout, spacing,
    UiAction, UiRect,
};
use macroquad_toolkit::ui::draw_ui_text;

use super::scene::draw_bottom_taxi_scene;

/// Draw the guideline decision screen
pub fn draw_guideline_decision(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.25).clamp(200.0, 280.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(ref guideline) = game_state.active_guideline {
        if let Some(data) = game_data {
            let center_x = screen_width() / 2.0;
            let rect_w = (screen_width() - 140.0).min(860.0);
            let rect_h = (scene_rect.y - layout::STATUS_BAR_HEIGHT - 76.0).clamp(440.0, 520.0);
            let rect = UiRect::centered_x(
                screen_width(),
                layout::STATUS_BAR_HEIGHT + 34.0,
                rect_w,
                rect_h,
            );
            draw_glass_panel(rect, colors::ACCENT_WARNING);

            let inner = rect.inset(spacing::PADDING_LG);
            let mut y = inner.y;

            // Title
            draw_ui_text(
                &data.localization.ui.game.guidelines.title,
                inner.x,
                y + 28.0,
                fonts::SIZE_XL,
                colors::ACCENT_WARNING,
            );
            y += 50.0;

            // Timer with color coding
            let time_left = game_state.guideline_time_remaining;
            let timer_color = if time_left <= 10.0 {
                colors::FUEL_CRITICAL
            } else if time_left <= 20.0 {
                colors::ACCENT_WARNING
            } else {
                colors::FUEL_GOOD
            };

            // "⏱️ Time: {:.1}s"
            let timer_text = data
                .localization
                .ui
                .game
                .guidelines
                .timer
                .replace("{:.1}", &format!("{:.1}", time_left));

            draw_ui_text(&timer_text, inner.x, y + 20.0, fonts::SIZE_LG, timer_color);
            y += 50.0;

            // Guideline info
            draw_ui_text(
                &data.localization.ui.game.guidelines.label,
                inner.x,
                y + 18.0,
                fonts::SIZE_MD,
                colors::TEXT_MUTED,
            );
            y += 25.0;
            draw_ui_text(
                &guideline.title,
                inner.x,
                y + 18.0,
                fonts::SIZE_LG,
                colors::ACCENT_SKY,
            );
            y += 35.0;

            // Description (truncated)
            let desc_preview = if guideline.description.len() > 60 {
                format!("{}...", &guideline.description[..60])
            } else {
                guideline.description.clone()
            };
            draw_ui_text(
                &desc_preview,
                inner.x,
                y + 16.0,
                fonts::SIZE_SM,
                colors::TEXT_PRIMARY,
            );
            y += 50.0;

            // Detected tells
            draw_ui_text(
                &data.localization.ui.game.guidelines.tells_label,
                inner.x,
                y + 18.0,
                fonts::SIZE_MD,
                colors::TEXT_MUTED,
            );
            y += 30.0;

            let relevant_tells: Vec<_> = game_state
                .detected_tells
                .iter()
                .filter(|t| t.related_guideline == Some(guideline.id))
                .collect();

            if relevant_tells.is_empty() {
                draw_ui_text(
                    &data.localization.ui.game.guidelines.no_tells,
                    inner.x + 20.0,
                    y + 16.0,
                    fonts::SIZE_SM,
                    colors::TEXT_MUTED,
                );
                y += 25.0;
            } else {
                for tell in relevant_tells.iter().take(3) {
                    let (intensity_text, intensity_color) = match tell.tell.intensity {
                        data::TellIntensity::Subtle => (
                            &data.localization.ui.game.guidelines.intensity.subtle,
                            colors::TEXT_MUTED,
                        ),
                        data::TellIntensity::Moderate => (
                            &data.localization.ui.game.guidelines.intensity.moderate,
                            colors::ACCENT_WARNING,
                        ),
                        data::TellIntensity::Obvious => (
                            &data.localization.ui.game.guidelines.intensity.obvious,
                            colors::FUEL_CRITICAL,
                        ),
                    };

                    let noticed_text = if tell.player_noticed {
                        "noticed"
                    } else {
                        "uncertain"
                    };
                    let age = (get_time() - tell.detection_time).max(0.0);
                    let tell_text = format!(
                        "• [{}] {} ({}, {:.0}s)",
                        intensity_text, tell.tell.description, noticed_text, age
                    );
                    draw_ui_text(
                        &tell_text,
                        inner.x + 20.0,
                        y + 16.0,
                        fonts::SIZE_SM,
                        intensity_color,
                    );
                    y += 25.0;
                }
            }

            y += 30.0;

            // Decision buttons
            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_spacing = 20.0;

            if let Some(last_decision) = game_state.decision_history.last() {
                let last_action = match last_decision.action {
                    crate::state::GuidelineAction::Follow => "followed",
                    crate::state::GuidelineAction::Break => "broke",
                };
                let history_text = format!(
                    "Last decision: #{} passenger {} {} with {} tells ({:.0}s ago)",
                    last_decision.guideline_id,
                    last_decision.passenger_id,
                    last_action,
                    last_decision.tells_present.len(),
                    (get_time() - last_decision.timestamp).max(0.0)
                );
                draw_ui_text(
                    &history_text,
                    inner.x,
                    y - 8.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
            }

            // Follow guideline button (left)
            if draw_glass_button(
                UiRect::new(center_x - btn_w - btn_spacing / 2.0, y, btn_w, btn_h),
                &data.localization.ui.game.guidelines.follow,
                colors::FUEL_GOOD,
                true,
            ) {
                return UiAction::FollowGuideline;
            }

            if draw_glass_button(
                UiRect::new(center_x + btn_spacing / 2.0, y, btn_w, btn_h),
                &data.localization.ui.game.guidelines.break_guideline,
                colors::ACCENT_DANGER,
                true,
            ) {
                return UiAction::BreakGuideline;
            }

            // Auto-decide if time runs out
            if time_left <= 0.0 {
                return UiAction::FollowGuideline;
            }
        }
    }

    UiAction::None
}

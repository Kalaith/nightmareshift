//! The mid-ride event screen: event prompt and choice buttons.

use macroquad::prelude::*;

use crate::data::GameData;
use crate::state::GameState;
use crate::ui::{
    colors, draw_cockpit_background, draw_glass_button, draw_glass_panel, draw_passenger_portrait,
    draw_small_caps, draw_wrapped_text, fonts, layout, spacing, UiAction, UiRect,
};
use macroquad_toolkit::ui::draw_ui_text;

use super::scene::draw_bottom_taxi_scene;

/// Draw the interaction screen (Mid-Ride Event)
pub fn draw_interaction(
    game_state: &GameState,
    _game_data: Option<&GameData>,
    player_stats: &crate::state::PlayerStats,
) -> UiAction {
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.27).clamp(210.0, 300.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(event) = &game_state.current_event {
        let rect_w = (screen_width() - 140.0).min(980.0);
        let rect_h = (scene_rect.y - layout::STATUS_BAR_HEIGHT - 76.0).clamp(430.0, 520.0);
        let rect = UiRect::centered_x(
            screen_width(),
            layout::STATUS_BAR_HEIGHT + 34.0,
            rect_w,
            rect_h,
        );
        draw_glass_panel(rect, colors::BORDER);

        let inner = rect.inset(spacing::PADDING_MD);
        let left_w = inner.w * 0.46;
        let right_x = inner.x + left_w + 28.0;
        let right_w = inner.w - left_w - 28.0;
        let mut y = inner.y + 16.0;

        draw_small_caps(
            "Mid-Ride Event",
            inner.x,
            y,
            fonts::SIZE_SM,
            colors::CAB_YELLOW,
        );
        y += 34.0;
        draw_ui_text(
            &event.title,
            inner.x,
            y,
            fonts::SIZE_XXL,
            colors::TEXT_PRIMARY,
        );
        y += 44.0;

        draw_wrapped_text(
            &event.description,
            inner.x,
            y,
            left_w,
            fonts::SIZE_MD,
            23.0,
            colors::TEXT_SECONDARY,
            8,
        );

        if let Some(passenger) = &game_state.current_passenger {
            let portrait_size = left_w.min(inner.h - 210.0).clamp(180.0, 260.0);
            let portrait_rect = UiRect::new(
                inner.x,
                inner.y + inner.h - portrait_size - 14.0,
                portrait_size,
                portrait_size,
            );
            draw_passenger_portrait(portrait_rect, passenger.id);
            draw_ui_text(
                &passenger.name,
                portrait_rect.x + portrait_rect.w + 18.0,
                portrait_rect.y + 42.0,
                fonts::SIZE_LG,
                colors::CAB_YELLOW,
            );
            draw_wrapped_text(
                &format!("{} -> {}", passenger.pickup, passenger.destination),
                portrait_rect.x + portrait_rect.w + 18.0,
                portrait_rect.y + 72.0,
                left_w - portrait_size - 18.0,
                fonts::SIZE_SM,
                18.0,
                colors::TEXT_MUTED,
                3,
            );
        }

        let mut choice_y = inner.y + 28.0;
        for (i, choice) in event.choices.iter().enumerate() {
            let btn_h = 78.0;
            let btn_rect = UiRect::new(right_x, choice_y, right_w, btn_h);
            let mut hint_text = None;
            if let Some(req_trait) = &choice.required_trait {
                if let Some(passenger) = &game_state.current_passenger {
                    if passenger.traits.contains(req_trait)
                        && player_stats.is_backstory_unlocked(passenger.id)
                    {
                        hint_text = Some(format!("{}'s {} helps!", passenger.name, req_trait));
                    }
                }
            }

            if draw_glass_button(btn_rect, "", colors::ACCENT_WARNING, true) {
                return UiAction::SelectEventChoice(i);
            }
            draw_small_caps(
                &format!("[{}]", i + 1),
                right_x + 16.0,
                choice_y + 28.0,
                fonts::SIZE_SM,
                colors::CAB_YELLOW,
            );
            draw_wrapped_text(
                &choice.description,
                right_x + 58.0,
                choice_y + 23.0,
                right_w - 78.0,
                fonts::SIZE_SM,
                18.0,
                colors::TEXT_PRIMARY,
                2,
            );
            if let Some(hint) = hint_text {
                draw_ui_text(
                    &hint,
                    right_x + 58.0,
                    choice_y + btn_h - 12.0,
                    fonts::SIZE_XS,
                    colors::FUEL_GOOD,
                );
            }

            choice_y += btn_h + 14.0;
            if choice_y > rect.bottom() - 70.0 {
                break;
            }
        }
    } else if let Some(ref passenger) = game_state.current_passenger {
        // Fallback or legacy interaction (START of ride dialogue if any?)
        // Currently `Interaction` phase is reused for start pickup too?
        // No, Pickup phase transitions to Interaction.
        // My implementation in RideService sets current_event when transitioning to Interaction.
        // So `current_event` SHOULD be present.
        // But if I want to support legacy behavior just in case:
        let rect = UiRect::centered_x(screen_width(), 150.0, 500.0, 150.0);
        draw_glass_panel(rect, colors::BORDER);

        let inner = rect.inset(spacing::PADDING_MD);

        // Name
        draw_ui_text(
            &passenger.name,
            inner.x,
            inner.y + 24.0,
            fonts::SIZE_LG,
            colors::ACCENT_WARNING,
        );

        // Dialogue
        if let Some(ref dialogue) = game_state.current_passenger_dialogue {
            let preview = if dialogue.len() > 70 {
                format!("\"{}...\"", &dialogue[..70])
            } else {
                format!("\"{}\"", dialogue)
            };
            draw_ui_text(
                &preview,
                inner.x,
                inner.y + 60.0,
                fonts::SIZE_MD,
                colors::TEXT_PRIMARY,
            );
        }

        // Continue Button
        if draw_glass_button(
            UiRect::new(
                screen_width() / 2.0 - 100.0,
                rect.bottom() + 40.0,
                200.0,
                50.0,
            ),
            "Continue",
            colors::CAB_YELLOW,
            true,
        ) {
            return UiAction::Continue;
        }
    }
    UiAction::None
}

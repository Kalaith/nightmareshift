//! Reusable UI components.

use super::*;
use crate::data::*;
use crate::state::*;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

fn ascii_trimmed(text: String) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii())
        .collect::<String>()
        .trim()
        .to_string()
}

/// Status bar component at top of screen
pub struct StatusBar;

impl StatusBar {
    pub fn draw(
        state: &GameState,
        constants: &ConstantsData,
        game_data: Option<&GameData>,
    ) -> UiAction {
        if let Some(data) = game_data {
            let bar_rect = UiRect::new(0.0, 0.0, screen_width(), layout::STATUS_BAR_HEIGHT);
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                layout::STATUS_BAR_HEIGHT,
                colors::BLACK,
            );
            draw_glass_panel(bar_rect, colors::BORDER_DIM);

            let padding = 18.0;
            let y = 18.0;
            let btn_h = 38.0;
            let inv_btn_w = 78.0;
            let inv_btn_x = screen_width() - padding - inv_btn_w;
            let rules_btn_x = inv_btn_x - 96.0;
            let btn_y = 17.0;
            let stats_right = rules_btn_x - padding;
            let stat_slot_w = ((stats_right - padding) / 5.0).max(74.0);
            let stat_x = |idx: usize| padding + stat_slot_w * idx as f32;
            let divider_x = |idx: usize| padding + stat_slot_w * idx as f32 - 10.0;

            let fuel_color = get_fuel_color(state.fuel);
            let fuel_value = ascii_trimmed(
                data.localization
                    .ui
                    .game
                    .status_bar
                    .fuel
                    .replace("{}", &(state.fuel as u32).to_string()),
            );
            draw_stat_block("F", &fuel_value, "Fuel", stat_x(0), y, fuel_color);
            draw_divider(divider_x(1), 18.0, 38.0);

            let earnings_value = ascii_trimmed(
                data.localization
                    .ui
                    .game
                    .status_bar
                    .earnings
                    .replace(
                        "${}",
                        &format!("{}{}", data.localization.ui.common.currency, state.earnings),
                    )
                    .replace("{}", &state.earnings.to_string()),
            );
            draw_stat_block(
                "$",
                &earnings_value,
                "Earnings",
                stat_x(1),
                y,
                colors::ACCENT_GOLD,
            );
            draw_divider(divider_x(2), 18.0, 38.0);

            let time_color = if state.is_time_critical(constants) {
                colors::FUEL_CRITICAL
            } else {
                colors::TEXT_PRIMARY
            };
            let hours = state.time_remaining / 60;
            let mins = state.time_remaining % 60;
            let formatted_time = data
                .localization
                .ui
                .common
                .time_format
                .replacen("{}", &hours.to_string(), 1)
                .replacen("{:02}", &format!("{:02}", mins), 1);
            let time_value = ascii_trimmed(
                data.localization
                    .ui
                    .game
                    .status_bar
                    .time
                    .replacen("{}", &hours.to_string(), 1)
                    .replacen("{:02}", &format!("{:02}", mins), 1)
                    .replace(&format!("{}:{:02}", hours, mins), &formatted_time),
            );
            draw_stat_block("T", &time_value, "Time", stat_x(2), y, time_color);
            draw_divider(divider_x(3), 18.0, 38.0);

            let rides_value = state.rides_completed.to_string();
            let rides_label =
                ascii_trimmed(data.localization.ui.game.status_bar.rides.replace("{}", ""));
            let rides_label = if rides_label.is_empty() {
                "Rides"
            } else {
                rides_label.as_str()
            };
            draw_stat_block(
                "#",
                &rides_value,
                rides_label,
                stat_x(3),
                y,
                colors::TEXT_PRIMARY,
            );
            draw_divider(divider_x(4), 18.0, 38.0);

            let weather_name = state.current_weather.weather_type.name();
            let weather_label = if weather_name.len() > 8 {
                format!("{}...", &weather_name[..8])
            } else {
                weather_name.to_string()
            };
            draw_stat_block(
                "W",
                &weather_label,
                "Weather",
                stat_x(4),
                y,
                colors::ACCENT_SKY,
            );

            if draw_glass_button(
                UiRect::new(rules_btn_x, btn_y, 86.0, btn_h),
                "R Rules",
                colors::TEXT_SECONDARY,
                true,
            ) {
                return UiAction::ToggleRules;
            }
            if draw_glass_button(
                UiRect::new(inv_btn_x, btn_y, inv_btn_w, btn_h),
                "I Inv",
                colors::TEXT_SECONDARY,
                true,
            ) {
                return UiAction::ToggleInventory;
            }
        }
        UiAction::None
    }
}

/// Passenger card component
pub struct PassengerCard;

impl PassengerCard {
    pub fn draw(
        passenger: &Passenger,
        rect: UiRect,
        show_controls: bool,
        dialogue: Option<&String>,
        game_data: Option<&GameData>,
    ) -> UiAction {
        draw_glass_panel(rect, colors::BORDER);
        let inner = rect.inset(spacing::PADDING_MD);
        let portrait_h = (rect.h * 0.38).clamp(150.0, 220.0);
        let portrait_rect = UiRect::new(inner.x, inner.y, inner.w, portrait_h);
        draw_passenger_portrait(portrait_rect, passenger.id);

        let mut y = portrait_rect.bottom() + 28.0;
        draw_ui_text(
            &passenger.name,
            inner.x,
            y,
            fonts::SIZE_XL,
            colors::CAB_YELLOW,
        );
        y += 22.0;

        let rarity_text = format!("{:?}", passenger.rarity);
        let rarity_color = match passenger.rarity {
            Rarity::Common => colors::TEXT_MUTED,
            Rarity::Uncommon => colors::ACCENT_PRIMARY,
            Rarity::Rare => colors::ACCENT_SKY,
            Rarity::Legendary => colors::ACCENT_GOLD,
        };
        draw_small_caps(&rarity_text, inner.x, y, fonts::SIZE_XS, rarity_color);
        y += 28.0;

        y = draw_wrapped_text(
            &passenger.description,
            inner.x,
            y,
            inner.w,
            fonts::SIZE_SM,
            18.0,
            colors::TEXT_SECONDARY,
            2,
        ) + 8.0;

        let route = format!("{} → {}", passenger.pickup, passenger.destination);
        y = draw_wrapped_text(
            &route,
            inner.x,
            y,
            inner.w,
            fonts::SIZE_SM,
            18.0,
            colors::TEXT_PRIMARY,
            2,
        ) + 10.0;

        let fare = format!("| ${}", passenger.fare);
        draw_ui_text(&fare, inner.x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
        y += 30.0;

        if let Some(dialogue_text) = dialogue {
            let preview = if dialogue_text.len() > 80 {
                format!("\"{}...\"", &dialogue_text[..80])
            } else {
                format!("\"{}\"", dialogue_text)
            };
            draw_wrapped_text(
                &preview,
                inner.x,
                y,
                inner.w,
                fonts::SIZE_XS,
                16.0,
                colors::TEXT_MUTED,
                2,
            );
        }

        if show_controls {
            let accept_text = if let Some(d) = game_data {
                d.localization.ui.common.accept_space.clone()
            } else {
                "Accept (SPACE)".to_string()
            };

            let decline_text = if let Some(d) = game_data {
                d.localization.ui.common.decline_esc.clone()
            } else {
                "Decline (ESC)".to_string()
            };

            let btn_h = 40.0;
            let gap = 12.0;
            let btn_w = (inner.w - gap) / 2.0;
            let btn_y = rect.bottom() - btn_h - spacing::PADDING_MD;
            if draw_glass_button(
                UiRect::new(inner.x, btn_y, btn_w, btn_h),
                &accept_text,
                colors::ACCENT_PRIMARY,
                true,
            ) {
                return UiAction::AcceptRide;
            }
            if draw_glass_button(
                UiRect::new(inner.x + btn_w + gap, btn_y, btn_w, btn_h),
                &decline_text,
                colors::ACCENT_DANGER,
                true,
            ) {
                return UiAction::DeclineRide;
            }
        }
        UiAction::None
    }
}
/// Completion summary component
pub struct CompletionSummary;

impl CompletionSummary {
    pub fn draw(
        completion: &RideCompletion,
        rect: UiRect,
        game_data: Option<&GameData>,
    ) -> UiAction {
        draw_glass_panel(rect, colors::FUEL_GOOD);

        let inner = rect.inset(spacing::PADDING_MD);
        let left_w = inner.w * 0.42;
        let right_x = inner.x + left_w + 26.0;
        let right_w = inner.w - left_w - 26.0;
        let mut y = inner.y + 14.0;

        if let Some(data) = game_data {
            let portrait_size = left_w.min(inner.h - 64.0).clamp(240.0, 390.0);
            let portrait_rect = UiRect::new(inner.x, inner.y + 16.0, portrait_size, portrait_size);
            draw_passenger_portrait(portrait_rect, completion.passenger.id);

            draw_ui_text(
                &data.localization.ui.game.completion.title,
                right_x,
                y + 30.0,
                fonts::SIZE_XL,
                colors::FUEL_GOOD,
            );
            y += 60.0;

            draw_ui_text(
                &completion.passenger.name,
                right_x,
                y,
                fonts::SIZE_LG,
                colors::ACCENT_WARNING,
            );
            y += 35.0;

            if !completion.passenger.dialogue.is_empty() {
                let feedback = &completion.passenger.dialogue[0];
                let formatted_feedback = format!("\"{}\"", feedback);
                y = draw_wrapped_text(
                    &formatted_feedback,
                    right_x,
                    y,
                    right_w,
                    fonts::SIZE_SM,
                    20.0,
                    colors::TEXT_MUTED,
                    3,
                ) + 22.0;
            }

            let fare_text = data
                .localization
                .ui
                .game
                .completion
                .fare
                .replace("{}", &completion.fare_earned.to_string());
            draw_ui_text(&fare_text, right_x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
            y += 35.0;

            if !completion.items_received.is_empty() {
                draw_ui_text(
                    &data.localization.ui.game.completion.items,
                    right_x,
                    y,
                    fonts::SIZE_MD,
                    colors::TEXT_PRIMARY,
                );
                y += 25.0;

                for item in &completion.items_received {
                    let item_text = format!("  {}", item.name);
                    let item_color = match item.rarity {
                        crate::data::Rarity::Common => colors::TEXT_SECONDARY,
                        crate::data::Rarity::Uncommon => colors::ACCENT_PRIMARY,
                        crate::data::Rarity::Rare => colors::ACCENT_SKY,
                        crate::data::Rarity::Legendary => colors::ACCENT_GOLD,
                    };
                    draw_ui_text(&item_text, right_x, y, fonts::SIZE_SM, item_color);
                    y += 20.0;
                }
                y += 10.0;
            }

            if let Some((name, backstory)) = &completion.backstory_unlocked {
                let unlock_text = data
                    .localization
                    .ui
                    .game
                    .completion
                    .backstory
                    .replace("{}", name);

                draw_ui_text(
                    &unlock_text,
                    right_x,
                    y,
                    fonts::SIZE_MD,
                    colors::ACCENT_DANGER,
                );
                y += 25.0;

                draw_wrapped_text(
                    backstory,
                    right_x,
                    y,
                    right_w,
                    fonts::SIZE_SM,
                    19.0,
                    colors::TEXT_MUTED,
                    4,
                );
            }

            if draw_glass_button(
                UiRect::new(right_x, rect.bottom() - 58.0, right_w, 44.0),
                &data.localization.ui.common.continue_space,
                colors::FUEL_GOOD,
                true,
            ) {
                return UiAction::Continue;
            }
        }

        UiAction::None
    }
}

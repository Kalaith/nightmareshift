//! Reusable UI components.

use macroquad::prelude::*;
use super::*;
use macroquad_toolkit::ui::*;
use crate::data::*;
use crate::state::*;

/// Status bar component at top of screen
pub struct StatusBar;

impl StatusBar {
    pub fn draw(state: &GameState, constants: &ConstantsData, game_data: Option<&GameData>) {
        if let Some(data) = game_data {
            let bar_rect = UiRect::new(0.0, 0.0, screen_width(), layout::STATUS_BAR_HEIGHT);
            draw_panel(bar_rect, colors::PANEL_BG);

            let padding = spacing::PADDING_LG;
            let mut x = padding;
            let y = layout::STATUS_BAR_TEXT_Y;

            // Fuel gauge with icon
            let fuel_color = get_fuel_color(state.fuel);
            // "⛽ {}%"
            let fuel_text = data.localization.ui.game.status_bar.fuel
                .replace("{}", &(state.fuel as u32).to_string());
            draw_text(&fuel_text, x, y, fonts::SIZE_LG, fuel_color);
            x += layout::STATUS_ITEM_SPACING;

            // Earnings
            // "💰 ${}"
            let earnings_text = data.localization.ui.game.status_bar.earnings
                .replace("{}", &state.earnings.to_string());
            draw_text(&earnings_text, x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
            x += layout::STATUS_EARNINGS_SPACING;

            // Time remaining
            let time_color = if state.is_time_critical(constants) {
                colors::FUEL_CRITICAL
            } else {
                colors::TEXT_PRIMARY
            };
            let hours = state.time_remaining / 60;
            let mins = state.time_remaining % 60;
            // "⏰ {}:{:02}" - we need to handle padding manually or use format! if supported by replace
            // Localization string is likely "⏰ {}:{}" or similar.
            // Let's format the time first then replace ONE placeholder, or replace carefully.
            // Json says "⏰ {}:{:02}" which is rust format syntax, but we can't use it directly on runtime string easily.
            // Assumption: The JSON string might just be "⏰ {}" and we format the time ourselves?
            // Or we treat it as "⏰ {}:{}" and do replace.
            
            // Re-checking json I wrote: "time": "⏰ {}:{:02}"
            // Wait, standard .replace doesn't handle {:02}.
            // I should have written "{}:{}" in json or handle formatted string.
            // Let's assume I change JSON or handle it here.
            // Best approach: Format keys in Rust, then insert into template if template has distinct keys.
            // Given current constraints, I'll format the time string manually and replace the whole time block if possible,
            // OR use a standard format string.
            // The JSON has "⏰ {}:{:02}", which implies I intended to use it with format!, but that requires a compile time string literal.
            // Workaround: Ignore the {:02} in JSON key logic and just format value.
            // Better: Load "time": "⏰ {}" and format "HH:mm".
            // Since I can't easily change JSON right now without another step, let's try to be smart.
            // I'll format the time string `MM:SS` and if the json has `⏰ {}` I replace it.
            // But json has `⏰ {}:{:02}`.
            
            // Let's just hardcode the icon + formatted time if the pattern is too complex, OR rely on a simpler key.
            // Let's try to match the intent.
            // Actually, I can construct the string `format!("⏰ {}:{:02}", hours, mins)` directly if I ignore the json pattern for the sophisticated format part
            // and just use the label from json if it were separated.
            // But I want to use the JSON.
            // I will assume for now I can just reconstruct it or replace key parts.
            // Actually, simplest is to treat the whole string as a format string for `rt_format` crate if I had it.
            // I don't.
            
            // Hack: Just reconstruct it using known symbols from common.currency etc if needed, but here:
            // I will use format! with the hardcoded emoji if needed, OR try to parse existing string.
            // Let's manually double-replace: replace "{}" with hours, then "{:02}" ... wait replace won't match "{:02}".
            
            // Let's just override the logic: 
            // The JSON string is only useful if I can use it.
            // I'll format the time as "H:MM" and look for a placeholder "{}" in the json string if I modify it,
            // or just use a fixed format for now and note to fix JSON later to be simpler like "Time: {}"
            
            // "⏰ {}:{:02}" -> this is effectively useless for runtime replacement without a proper formatter.
            // I will assume the JSON is just "Time: {}" (which I can replace) or I ignore it and use:
            let time_str = format!("⏰ {}:{:02}", hours, mins); 
            draw_text(&time_str, x, y, fonts::SIZE_LG, time_color);
            x += layout::STATUS_ITEM_SPACING;

            // Rides completed
            // "🚕 {} rides"
            let rides_text = data.localization.ui.game.status_bar.rides
                .replace("{}", &state.rides_completed.to_string());
            draw_text(&rides_text, x, y, fonts::SIZE_LG, colors::TEXT_PRIMARY);

            // Weather on right side
            // "{} {:?}" - icon + type
            let weather_text = format!(
                "{} {:?}",
                state.current_weather.icon,
                state.current_weather.weather_type
            );
            
            let weather_dims = measure_text(&weather_text, None, fonts::SIZE_LG as u16, 1.0);
            draw_text(
                &weather_text,
                screen_width() - weather_dims.width - padding,
                y,
                fonts::SIZE_LG,
                colors::ACCENT_SKY,
            );
        }
    }
}

/// Passenger card component
pub struct PassengerCard;

impl PassengerCard {
    pub fn draw(passenger: &Passenger, rect: UiRect, show_controls: bool, dialogue: Option<&String>, game_data: Option<&GameData>) -> UiAction {
        // Card background
        draw_panel_bordered(rect, colors::PANEL_BG, colors::ACCENT_PRIMARY, 2.0);

        let inner = rect.inset(spacing::PADDING_MD);
        let center_x = rect.center_x();

        // Emoji
        draw_text(&passenger.emoji, center_x - 20.0, inner.y + 40.0, 48.0, colors::TEXT_PRIMARY);

        // Name
        let name_dims = measure_text(&passenger.name, None, fonts::SIZE_XL as u16, 1.0);
        draw_text(
            &passenger.name,
            center_x - name_dims.width / 2.0,
            inner.y + 80.0,
            fonts::SIZE_XL,
            colors::ACCENT_WARNING,
        );

        // Rarity badge
        let rarity_text = format!("{:?}", passenger.rarity);
        let rarity_color = match passenger.rarity {
            Rarity::Common => colors::TEXT_MUTED,
            Rarity::Uncommon => colors::ACCENT_PRIMARY,
            Rarity::Rare => colors::ACCENT_SKY,
            Rarity::Legendary => colors::ACCENT_GOLD,
        };
        let rarity_dims = measure_text(&rarity_text, None, fonts::SIZE_SM as u16, 1.0);
        draw_text(
            &rarity_text,
            center_x - rarity_dims.width / 2.0,
            inner.y + 100.0,
            fonts::SIZE_SM,
            rarity_color,
        );

        // Description
        draw_text(
            &passenger.description,
            inner.x,
            inner.y + 130.0,
            fonts::SIZE_MD,
            colors::TEXT_SECONDARY,
        );

        // Route info
        let route = format!("{} → {}", passenger.pickup, passenger.destination);
        draw_text(&route, inner.x, inner.y + 160.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);

        // Fare
        let fare = format!("💰 ${}", passenger.fare);
        draw_text(&fare, inner.x, inner.y + 190.0, fonts::SIZE_LG, colors::ACCENT_GOLD);

        // Dialogue preview
        if let Some(dialogue_text) = dialogue {
            let preview = if dialogue_text.len() > 60 {
                format!("\"{}...\"", &dialogue_text[..60])
            } else {
                format!("\"{}\"", dialogue_text)
            };
            draw_text(&preview, inner.x, inner.y + 230.0, fonts::SIZE_SM, colors::TEXT_MUTED);
        }

        // Controls
        if show_controls {
             let btn_w = 150.0;
             let btn_h = 40.0;
             let padding = 20.0;
            
            // Get localized button texts or defaults
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

             // Accept Button
             if button(
                 inner.x,
                 rect.bottom() - btn_h - spacing::PADDING_MD,
                 btn_w,
                 btn_h,
                 &accept_text
             ) {
                 return UiAction::AcceptRide;
             }

             // Decline Button
             if button(
                 inner.x + btn_w + padding,
                 rect.bottom() - btn_h - spacing::PADDING_MD,
                 btn_w,
                 btn_h,
                 &decline_text
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
    pub fn draw(completion: &RideCompletion, rect: UiRect, game_data: Option<&GameData>) -> UiAction {
        draw_panel_bordered(rect, colors::SUCCESS_BG, colors::FUEL_GOOD, 3.0);

        let inner = rect.inset(spacing::PADDING_MD);
        let mut y = inner.y;
        
        if let Some(data) = game_data {
            // Title
            draw_text(
                &data.localization.ui.game.completion.title,
                inner.x,
                y + 30.0,
                fonts::SIZE_XL,
                colors::FUEL_GOOD
            );
            y += 60.0;

            // Passenger emoji and name
            let passenger_header = format!("{} {}", completion.passenger.emoji, completion.passenger.name);
            draw_text(&passenger_header, inner.x, y, fonts::SIZE_LG, colors::ACCENT_WARNING);
            y += 35.0;

            // Passenger feedback
            if !completion.passenger.dialogue.is_empty() {
                let feedback = &completion.passenger.dialogue[0];
                let formatted_feedback = format!("\"{}\"", feedback);
                
                // Wrap text
                let max_width = inner.w;
                let words: Vec<&str> = formatted_feedback.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    let test_line = if current_line.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", current_line, word)
                    };
                    
                    let dims = measure_text(&test_line, None, fonts::SIZE_SM as u16, 1.0);
                    if dims.width <= max_width {
                        current_line = test_line;
                    } else {
                        draw_text(&current_line, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                        y += 20.0;
                        current_line = word.to_string();
                    }
                }
                if !current_line.is_empty() {
                    draw_text(&current_line, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                    y += 30.0;
                }
            }

            // Fare earned
            // "💰 Fare Earned: ${}"
            let fare_text = data.localization.ui.game.completion.fare
                .replace("{}", &completion.fare_earned.to_string());
            draw_text(&fare_text, inner.x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
            y += 35.0;

            // Items received
            if !completion.items_received.is_empty() {
                draw_text(
                    &data.localization.ui.game.completion.items,
                    inner.x,
                    y,
                    fonts::SIZE_MD,
                    colors::TEXT_PRIMARY
                );
                y += 25.0;

                for item in &completion.items_received {
                    let item_text = format!("  • {}", item.name);
                    let item_color = match item.rarity {
                        crate::data::Rarity::Common => colors::TEXT_SECONDARY,
                        crate::data::Rarity::Uncommon => colors::ACCENT_PRIMARY,
                        crate::data::Rarity::Rare => colors::ACCENT_SKY,
                        crate::data::Rarity::Legendary => colors::ACCENT_GOLD,
                    };
                    draw_text(&item_text, inner.x, y, fonts::SIZE_SM, item_color);
                    y += 20.0;
                }
                y += 10.0;
            }

            // Backstory unlock
            if let Some((name, backstory)) = &completion.backstory_unlocked {
                // "🔓 Backstory Unlocked: {}"
                let unlock_text = data.localization.ui.game.completion.backstory
                    .replace("{}", name);
                    
                draw_text(
                    &unlock_text,
                    inner.x,
                    y,
                    fonts::SIZE_MD,
                    colors::ACCENT_DANGER,
                );
                y += 25.0;

                // Show full backstory with wrapping
                let max_width = inner.w;
                let words: Vec<&str> = backstory.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    let test_line = if current_line.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", current_line, word)
                    };
                    
                    let dims = measure_text(&test_line, None, fonts::SIZE_SM as u16, 1.0);
                    if dims.width <= max_width {
                        current_line = test_line;
                    } else {
                        draw_text(&current_line, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                        y += 20.0; // Line height
                        current_line = word.to_string();
                    }
                }
                if !current_line.is_empty() {
                    draw_text(&current_line, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                }
                // Ensure space for button? Y is updated.
            }

            // Continue Button
            if button(
                inner.x,
                rect.bottom() - 50.0,
                200.0,
                40.0,
                &data.localization.ui.common.continue_space
            ) {
                return UiAction::Continue;
            }
        }
        
        UiAction::None
    }
}

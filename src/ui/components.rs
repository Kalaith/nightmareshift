//! Reusable UI components.

use macroquad::prelude::*;
use super::*;
use macroquad_toolkit::ui::*;
use crate::data::*;
use crate::state::*;

/// Status bar component at top of screen
pub struct StatusBar;

impl StatusBar {
    pub fn draw(state: &GameState, constants: &ConstantsData) {
        let bar_rect = UiRect::new(0.0, 0.0, screen_width(), 55.0);
        draw_panel(bar_rect, colors::PANEL_BG);

        let padding = spacing::PADDING_LG;
        let mut x = padding;
        let y = 35.0;

        // Fuel gauge with icon
        let fuel_color = get_fuel_color(state.fuel);
        let fuel_text = format!("⛽ {}%", state.fuel as u32);
        draw_text(&fuel_text, x, y, fonts::SIZE_LG, fuel_color);
        x += 100.0;

        // Earnings
        let earnings_text = format!("💰 ${}", state.earnings);
        draw_text(&earnings_text, x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
        x += 120.0;

        // Time remaining
        let time_color = if state.is_time_critical(constants) {
            colors::FUEL_CRITICAL
        } else {
            colors::TEXT_PRIMARY
        };
        let hours = state.time_remaining / 60;
        let mins = state.time_remaining % 60;
        let time_text = format!("⏰ {}:{:02}", hours, mins);
        draw_text(&time_text, x, y, fonts::SIZE_LG, time_color);
        x += 100.0;

        // Rides completed
        let rides_text = format!("🚕 {} rides", state.rides_completed);
        draw_text(&rides_text, x, y, fonts::SIZE_LG, colors::TEXT_PRIMARY);

        // Weather on right side
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

/// Passenger card component
pub struct PassengerCard;

impl PassengerCard {
    pub fn draw(passenger: &Passenger, rect: UiRect, show_controls: bool, dialogue: Option<&String>) -> UiAction {
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

             // Accept Button
             if button(
                 inner.x,
                 rect.bottom() - btn_h - spacing::PADDING_MD,
                 btn_w,
                 btn_h,
                 "Accept (SPACE)"
             ) {
                 return UiAction::AcceptRide;
             }

             // Decline Button
             if button(
                 inner.x + btn_w + padding,
                 rect.bottom() - btn_h - spacing::PADDING_MD,
                 btn_w,
                 btn_h,
                 "Decline (ESC)"
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
    pub fn draw(completion: &RideCompletion, rect: UiRect) -> UiAction {
        draw_panel_bordered(rect, colors::SUCCESS_BG, colors::FUEL_GOOD, 3.0);

        let inner = rect.inset(spacing::PADDING_MD);
        let mut y = inner.y;

        // Title
        draw_text("✓ Ride Complete!", inner.x, y + 30.0, fonts::SIZE_XL, colors::FUEL_GOOD);
        y += 60.0;

        // Passenger emoji and name
        let passenger_header = format!("{} {}", completion.passenger.emoji, completion.passenger.name);
        draw_text(&passenger_header, inner.x, y, fonts::SIZE_LG, colors::ACCENT_WARNING);
        y += 35.0;

        // Passenger feedback (use first dialogue line as feedback)
        if !completion.passenger.dialogue.is_empty() {
            let feedback = &completion.passenger.dialogue[0];
            let feedback_preview = if feedback.len() > 50 {
                format!("\"{}...\"", &feedback[..50])
            } else {
                format!("\"{}\"", feedback)
            };
            draw_text(&feedback_preview, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
            y += 30.0;
        }

        // Fare earned
        let fare_text = format!("💰 Fare Earned: ${}", completion.fare_earned);
        draw_text(&fare_text, inner.x, y, fonts::SIZE_LG, colors::ACCENT_GOLD);
        y += 35.0;

        // Items received
        if !completion.items_received.is_empty() {
            draw_text("Items Received:", inner.x, y, fonts::SIZE_MD, colors::TEXT_PRIMARY);
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
            draw_text(
                &format!("🔓 Backstory Unlocked: {}", name),
                inner.x,
                y,
                fonts::SIZE_MD,
                colors::ACCENT_DANGER,
            );
            y += 25.0;

            // Show snippet of backstory
            let backstory_snippet = if backstory.len() > 60 {
                format!("   {}...", &backstory[..60])
            } else {
                format!("   {}", backstory)
            };
            draw_text(&backstory_snippet, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        }

        // Continue Button
        if button(
            inner.x,
            rect.bottom() - 50.0,
            200.0,
            40.0,
            "Continue (SPACE)"
        ) {
            return UiAction::Continue;
        }

        UiAction::None
    }
}

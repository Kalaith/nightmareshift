//! Game phase screens: Waiting, Driving, Interaction, DropOff, Guidelines.
//!
//! These screens are shown during active gameplay.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::{self, GameData, RouteType, Rarity, TimePhase, PreferenceLevel};
use crate::state::{GameState, GamePhase, DrivingPhase};
use crate::ui::{
    UiAction, UiRect, colors, fonts, spacing, 
    draw_panel, draw_panel_bordered, get_fuel_color,
    CompletionSummary,
};

/// Get a pulsing color for warning text (slow, gentle pulse)
fn pulsing_warning_color() -> Color {
    // Slow pulse: 1.5 second cycle (lower frequency = slower)
    let pulse = (get_time() * 2.0 * std::f64::consts::PI / 1.5).sin() as f32;
    // Pulse between 0.6 and 1.0 alpha (subtle, not jarring)  
    let alpha = 0.6 + pulse * 0.2 + 0.2;
    Color::new(colors::ACCENT_WARNING.r, colors::ACCENT_WARNING.g, colors::ACCENT_WARNING.b, alpha)
}


// Update signatures to accept player_stats
pub fn draw_game(game_state: &GameState, game_data: Option<&GameData>, player_stats: &crate::state::PlayerStats) -> UiAction {
    match game_state.game_phase {
        GamePhase::Waiting => draw_waiting(game_state, game_data),
        GamePhase::RideRequest => draw_ride_request(game_state, game_data),
        GamePhase::Driving => draw_driving(game_state, game_data, player_stats),
        GamePhase::Interaction => draw_interaction(game_state, game_data, player_stats),
        GamePhase::DropOff => draw_dropoff(game_state, game_data),
        GamePhase::GuidelineDecision => draw_guideline_decision(game_state, game_data),
        _ => UiAction::None,
    }
}

/// Draw the waiting for passenger screen
pub fn draw_waiting(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    if let Some(data) = game_data {
        let center_x = screen_width() / 2.0;
        let mut y = 120.0;

        // Title
        let text = &data.localization.ui.game.waiting.looking;
        let font_size = 24.0;
        let text_width = measure_text(text, None, font_size as u16, 1.0).width;
        draw_text(
            text,
            center_x - text_width / 2.0,
            y,
            font_size,
            colors::TEXT_SECONDARY,
        );
        y += 60.0;

        // Fuel status
        let fuel_pct = game_state.fuel;
        let fuel_color = get_fuel_color(fuel_pct);
        let fuel_status = if fuel_pct <= 10.0 {
            &data.localization.ui.game.waiting.fuel_status.critical
        } else if fuel_pct <= 20.0 {
            &data.localization.ui.game.waiting.fuel_status.low
        } else if fuel_pct <= 40.0 {
            &data.localization.ui.game.waiting.fuel_status.medium
        } else {
            &data.localization.ui.game.waiting.fuel_status.good
        };

        // "⛽ Fuel: {:.0}% - {}"
        let fuel_text = data.localization.ui.game.waiting.fuel_label
            .replace("{:.0}", &format!("{:.0}", fuel_pct))
            .replace("{}", fuel_status);
            
        let fuel_width = measure_text(&fuel_text, None, fonts::SIZE_LG as u16, 1.0).width;
        draw_text(&fuel_text, center_x - fuel_width / 2.0, y, fonts::SIZE_LG, fuel_color);
        y += 50.0;

        // Inventory hint
        // "Press I for Inventory ({}  items)"
        let inv_hint = data.localization.ui.game.waiting.inventory_hint
            .replace("{}", &game_state.inventory.len().to_string());
            
        let inv_width = measure_text(&inv_hint, None, fonts::SIZE_SM as u16, 1.0).width;
        draw_text(&inv_hint, center_x - inv_width / 2.0, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 35.0;

        // Refuel options
        if fuel_pct < 100.0 {
            let refuel_text = &data.localization.ui.game.waiting.refuel_title;
            let refuel_width = measure_text(refuel_text, None, fonts::SIZE_MD as u16, 1.0).width;
            draw_text(refuel_text, center_x - refuel_width / 2.0, y, fonts::SIZE_MD, colors::TEXT_PRIMARY);
            y += 40.0;

            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_spacing = 20.0;

            // Full refuel button
            let fuel_needed = 100.0 - fuel_pct;
            let full_cost = (fuel_needed * data.constants.fuel.cost_per_percent) as u32;
            
            // "Full Tank (${})"
            let full_label = data.localization.ui.game.waiting.full_tank
                .replace("{}", &full_cost.to_string());

            let can_afford_full = game_state.earnings >= full_cost;

            if can_afford_full && button(center_x - btn_w - btn_spacing / 2.0, y, btn_w, btn_h, &full_label) {
                return UiAction::RefuelFull;
            } else if !can_afford_full {
                // Show disabled button
                draw_rectangle(center_x - btn_w - btn_spacing / 2.0, y, btn_w, btn_h, Color::from_rgba(60, 60, 80, 255));
                let label_width = measure_text(&full_label, None, 18, 1.0).width;
                draw_text(&full_label, center_x - btn_w - btn_spacing / 2.0 + (btn_w - label_width) / 2.0, y + 30.0, 18.0, colors::TEXT_MUTED);
            }

            // Partial refuel button
            let partial_amount = 25.0_f32.min(fuel_needed);
            let partial_cost = (partial_amount * data.constants.fuel.cost_per_percent) as u32;
            
            // "+25% (${})"
            let partial_label = data.localization.ui.game.waiting.partial
                .replace("{}", &partial_cost.to_string());

            let can_afford_partial = game_state.earnings >= partial_cost;

            if can_afford_partial && button(center_x + btn_spacing / 2.0, y, btn_w, btn_h, &partial_label) {
                return UiAction::RefuelPartial;
            } else if !can_afford_partial {
                // Show disabled button
                draw_rectangle(center_x + btn_spacing / 2.0, y, btn_w, btn_h, Color::from_rgba(60, 60, 80, 255));
                let label_width = measure_text(&partial_label, None, 18, 1.0).width;
                draw_text(&partial_label, center_x + btn_spacing / 2.0 + (btn_w - label_width) / 2.0, y + 30.0, 18.0, colors::TEXT_MUTED);
            }

            y += 70.0;
        }

        // Find passenger button
        y += 20.0;
        let find_label = &data.localization.ui.game.waiting.find_passenger;
        let find_width = 300.0;
        let find_height = 60.0;
        if button(center_x - find_width / 2.0, y, find_width, find_height, find_label) {
            return UiAction::Continue;
        }

        UiAction::None
    } else {
        UiAction::None
    }
}

/// Draw the ride request screen (uses PassengerCard component)
pub fn draw_ride_request(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    use crate::ui::PassengerCard;
    
    if let Some(ref passenger) = game_state.current_passenger {
        let rect = UiRect::centered_x(100.0, 400.0, 350.0);
        return PassengerCard::draw(passenger, rect, true, game_state.current_passenger_dialogue.as_ref(), game_data);
    }
    UiAction::None
}

/// Draw the driving/route selection screen
pub fn draw_driving(game_state: &GameState, game_data: Option<&GameData>, _player_stats: &crate::state::PlayerStats) -> UiAction {
    let center_x = screen_width() / 2.0;
    let y = 80.0;
    
    if let Some(data) = game_data {
        // Phase indicator
        // Localized strings for pickup/dest
        // "Driving to pickup...", "Driving to destination..."
        let phase_text = match game_state.driving_phase {
            Some(DrivingPhase::Pickup) => &data.localization.ui.game.driving.pickup,
            Some(DrivingPhase::Destination) => &data.localization.ui.game.driving.destination,
            None => "Driving...",
        };
        
        let phase_width = measure_text(phase_text, None, 24, 1.0).width;
        draw_text(
            phase_text,
            center_x - phase_width / 2.0,
            y,
            24.0,
            WHITE,
        );

        // Route options
        // Use localized route names and descriptions
        let r = &data.localization.ui.game.driving.routes;
        let routes = [
            (RouteType::Normal, &r.normal, &r.normal_desc, "[1]"),
            (RouteType::Shortcut, &r.shortcut, &r.shortcut_desc, "[2]"),
            (RouteType::Scenic, &r.scenic, &r.scenic_desc, "[3]"),
            (RouteType::Police, &r.police, &r.police_desc, "[4]"),
        ];

        // Get passenger for preferences
        let passenger_opt = game_state.current_passenger.as_ref();

        let mut route_y = y + 60.0;
        for (i, (route_type, name, desc, key)) in routes.iter().enumerate() {
            // Check if route is blocked by environmental hazards
            let is_blocked = game_state.environmental_hazards.iter()
                .any(|h| h.blocks_route(*route_type));

            // Check weather warnings
            let mut weather_warning = String::new();
            if matches!(game_state.current_weather.intensity, data::WeatherIntensity::Heavy)
                && *route_type == RouteType::Shortcut {
                // "⚠️ {} weather!"
                let w_type = game_state.current_weather.weather_type.name();
                weather_warning = data.localization.ui.game.driving.weather_warning
                    .replace("{}", w_type);
            }
            if matches!(game_state.time_of_day.phase, TimePhase::Night | TimePhase::Latenight)
                && *route_type == RouteType::Shortcut {
                if !weather_warning.is_empty() {
                    // Combine weather and night warnings
                    weather_warning.push_str(" + 🌙 Night");
                } else {
                    weather_warning = data.localization.ui.game.driving.night_warning.clone();
                }
            }

            // Button logic for route selection (disabled if blocked)
            if !is_blocked && button(center_x - 200.0, route_y, 400.0, 100.0, "") {
                return UiAction::SelectRoute(i);
            }

            // Get passenger preference for this route
            let preference = passenger_opt.and_then(|p| p.get_route_preference(*route_type));

            // Background color based on preference or blocked status
            let bg_color = if is_blocked {
                Color::from_rgba(60, 30, 30, 255) // Dark red for blocked
            } else if let Some(pref) = preference {
                match pref.preference {
                    PreferenceLevel::Loves => Color::from_rgba(50, 100, 50, 255), // Dark green
                    PreferenceLevel::Likes => Color::from_rgba(50, 80, 60, 255),  // Light green tint
                    PreferenceLevel::Neutral => if i % 2 == 0 {
                        Color::from_hex(0x2d2d44)
                    } else {
                        Color::from_hex(0x252538)
                    },
                    PreferenceLevel::Dislikes => Color::from_rgba(80, 60, 50, 255), // Orange tint
                    PreferenceLevel::Fears => Color::from_rgba(100, 50, 50, 255),   // Dark red
                }
            } else {
                if i % 2 == 0 {
                    Color::from_hex(0x2d2d44)
                } else {
                    Color::from_hex(0x252538)
                }
            };

            draw_rectangle(
                center_x - 200.0,
                route_y,
                400.0,
                100.0,
                bg_color,
            );

            // Show blocked overlay if route is blocked
            if is_blocked {
                draw_rectangle(
                    center_x - 200.0,
                    route_y,
                    400.0,
                    100.0,
                    Color::from_rgba(0, 0, 0, 180),
                );
                // "🚫 BLOCKED"
                draw_text(&data.localization.ui.game.driving.blocked, center_x - 180.0, route_y + 25.0, 18.0, colors::FUEL_CRITICAL);
                if let Some(hazard) = game_state.environmental_hazards.iter()
                    .find(|h| h.blocks_route(*route_type)) {
                    draw_text(&hazard.description, center_x - 180.0, route_y + 50.0, 14.0, colors::ACCENT_WARNING);
                }
            } else {
                draw_text(key, center_x - 180.0, route_y + 25.0, 16.0, Color::from_hex(0x4ecdc4));
                draw_text(name, center_x - 140.0, route_y + 25.0, 18.0, WHITE);
                draw_text(desc, center_x - 140.0, route_y + 45.0, 14.0, Color::from_hex(0x888888));

                // Show passenger preference (Header for right side)
                if let Some(pref) = preference {
                    let pref_text = pref.preference.display_text();
                    if !pref_text.is_empty() {
                        let pref_color = match pref.preference {
                            PreferenceLevel::Loves => colors::FUEL_GOOD,
                            PreferenceLevel::Likes => Color::from_hex(0x88ff88),
                            PreferenceLevel::Neutral => colors::TEXT_MUTED,
                            PreferenceLevel::Dislikes => colors::ACCENT_WARNING,
                            PreferenceLevel::Fears => colors::FUEL_CRITICAL,
                        };
                        // Draw preference at top right
                        draw_text(pref_text, center_x + 60.0, route_y + 25.0, 14.0, pref_color);
                    }
                }

                // Weather warning (Left side, below desc) - with slow pulse
                if !weather_warning.is_empty() {
                    draw_text(&weather_warning, center_x - 140.0, route_y + 70.0, 12.0, pulsing_warning_color());
                }
            }

            // Draw risk tags (Right side, below preference)
            // Logic:
            // 1. Generate stable risks using seed
            // 2. Check mastery level for visibility
            use crate::engine::RouteService;
            let seed = game_state.rides_completed as u64 + game_state.current_passenger.as_ref().map(|p| p.id as u64).unwrap_or(0) + i as u64;
            let risk_tags = RouteService::generate_risk_tags(
                *route_type,
                Some(&game_state.current_weather),
                Some(&game_state.time_of_day),
                game_state.current_passenger.as_ref(), // Pass passenger for context
                Some(seed)
            );

            // Determine visibility based on route mastery
            let mastery = game_state.get_route_mastery(*route_type);
            let visible_count = if mastery >= 5 { 3 } else if mastery >= 3 { 2 } else if mastery >= 1 { 1 } else { 0 };

            let tag_x = center_x + 60.0;
            let tag_y = route_y + 45.0; // Start below preference
            
            for (tag_idx, tag) in risk_tags.iter().enumerate() {
                let is_visible = tag_idx < visible_count;
                let (text, color) = if is_visible {
                    (tag.name(), colors::ACCENT_WARNING) 
                } else {
                    ("???", colors::TEXT_MUTED)
                };
                
                // Draw tag
                draw_text(text, tag_x, tag_y + (tag_idx as f32 * 18.0), 12.0, color);
            }

            route_y += 110.0;
        }
    }

    UiAction::None
}

/// Draw the interaction screen (Mid-Ride Event)
pub fn draw_interaction(game_state: &GameState, _game_data: Option<&GameData>, player_stats: &crate::state::PlayerStats) -> UiAction {
    if let Some(event) = &game_state.current_event {
         let rect = UiRect::centered_x(100.0, 600.0, 500.0);
         draw_panel(rect, colors::PANEL_BG);
         
         let inner = rect.inset(spacing::PADDING_MD);
         let mut y = inner.y;

         // Title
         draw_text(&event.title, inner.x, y + 24.0, fonts::SIZE_LG, colors::ACCENT_WARNING);
         y += 50.0;

         // Description
         draw_text(&event.description, inner.x, y + 16.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);
         y += 60.0; // Space for choices

         // Choices
         for (i, choice) in event.choices.iter().enumerate() {
             let btn_h = 60.0;
             let _btn_rect = UiRect::new(inner.x, y, inner.w, btn_h);
             
             
             // Check traits for hints
             let mut hint_text = None;
             if let Some(req_trait) = &choice.required_trait {
                 if let Some(passenger) = &game_state.current_passenger {
                     // Check if passenger has trait AND backstory is unlocked
                     if passenger.traits.contains(req_trait) {
                         // Check unlock (assuming player_stats check matches logic)
                         // Wait, player_stats.is_backstory_unlocked(id)
                         if player_stats.is_backstory_unlocked(passenger.id) {
                            hint_text = Some(format!("{}'s {} helps!", passenger.name, req_trait));
                         }
                     }
                 }
             }

             if button(inner.x, y, inner.w, btn_h, &choice.description) {
                 return UiAction::SelectEventChoice(i);
             }
             
             // Draw hint if available (overlay on button or next to it)
             if let Some(hint) = hint_text {
                 draw_text(&hint, inner.x + 10.0, y - 5.0, 12.0, colors::FUEL_GOOD);
             }

             y += btn_h + 20.0;
         }

    } else if let Some(ref passenger) = game_state.current_passenger {
        // Fallback or legacy interaction (START of ride dialogue if any?)
        // Currently `Interaction` phase is reused for start pickup too? 
        // No, Pickup phase transitions to Interaction.
        // My implementation in RideService sets current_event when transitioning to Interaction.
        // So `current_event` SHOULD be present.
        // But if I want to support legacy behavior just in case:
        let rect = UiRect::centered_x(150.0, 500.0, 150.0);
        draw_panel(rect, colors::PANEL_BG);

        let inner = rect.inset(spacing::PADDING_MD);

        // Name
        draw_text(
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
            draw_text(&preview, inner.x, inner.y + 60.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);
        }

        // Continue Button
        if button(
            screen_width() / 2.0 - 100.0,
            rect.bottom() + 40.0,
            200.0,
            50.0,
            "Continue"
        ) {
            return UiAction::Continue;
        }
    }
    UiAction::None
}

/// Draw the dropoff screen
pub fn draw_dropoff(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    if let Some(ref completion) = game_state.last_ride_completion {
        let rect = UiRect::centered_x(100.0, 500.0, 500.0);
        // CompletionSummary now needs game_data
        let completion_action = CompletionSummary::draw(completion, rect, game_data);

        // Show trade offer if available
        if let Some((ref passenger_name, ref offered_item)) = game_state.pending_trade {
            if let Some(data) = game_data {
                let trade_y = rect.bottom() + 40.0;
                let trade_rect = UiRect::centered_x(trade_y, 450.0, 200.0);
                draw_panel(trade_rect, Color::from_rgba(40, 40, 60, 240));
    
                let inner = trade_rect.inset(spacing::PADDING_MD);
    
                // Title
                draw_text(
                    &data.localization.ui.game.trade.title,
                    inner.x,
                    inner.y + 24.0,
                    fonts::SIZE_LG,
                    colors::ACCENT_SKY,
                );
    
                // Message
                let msg = data.localization.ui.game.trade.wants_to_trade
                    .replace("{}", passenger_name);
                draw_text(&msg, inner.x, inner.y + 50.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);
    
                // Offered item
                let rarity_color = match offered_item.rarity {
                    Rarity::Common => colors::TEXT_MUTED,
                    Rarity::Uncommon => colors::ACCENT_SKY,
                    Rarity::Rare => Color::from_hex(0x87CEEB),
                    Rarity::Legendary => colors::ACCENT_WARNING,
                };
                
                let offer_text = data.localization.ui.game.trade.offering
                    .replace("{}", &offered_item.name);
                draw_text(&offer_text, inner.x, inner.y + 80.0, fonts::SIZE_MD, rarity_color);
    
                // Show what they want if available
                draw_text(
                    &data.localization.ui.game.trade.any_item,
                    inner.x,
                    inner.y + 105.0,
                    fonts::SIZE_SM,
                    colors::TEXT_MUTED,
                );
    
                // Buttons
                let btn_y = inner.y + 140.0;
                let btn_w = 200.0;
                let btn_h = 40.0;
                let center_x = screen_width() / 2.0;
    
                // Show inventory items for selection
                if !game_state.inventory.is_empty() {
                    draw_text(
                        &data.localization.ui.game.trade.select,
                        inner.x,
                        btn_y - 20.0,
                        fonts::SIZE_SM,
                        colors::TEXT_PRIMARY,
                    );
    
                    for (i, item) in game_state.inventory.iter().take(3).enumerate() {
                        if !item.can_trade {
                            continue;
                        }
                        let item_btn_y = btn_y + (i as f32 * 45.0);
                        if button(center_x - btn_w / 2.0, item_btn_y, btn_w, 35.0, &item.name) {
                            return UiAction::AcceptTrade(i);
                        }
                    }
    
                    // Decline button
                    let decline_y = btn_y + (150.0);
                    if button(center_x - btn_w / 2.0, decline_y, btn_w, btn_h, &data.localization.ui.game.trade.decline) {
                        return UiAction::DeclineTrade;
                    }
                } else {
                    // No items to trade
                    draw_text(
                        &data.localization.ui.game.trade.empty,
                        inner.x,
                        btn_y,
                        fonts::SIZE_SM,
                        colors::ACCENT_WARNING,
                    );
                    if button(center_x - btn_w / 2.0, btn_y + 30.0, btn_w, btn_h, &data.localization.ui.common.continue_text) {
                        return UiAction::DeclineTrade;
                    }
                }
            }
        } else {
            return completion_action;
        }
    }
    UiAction::None
}

/// Draw the guideline decision screen
pub fn draw_guideline_decision(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    if let Some(ref guideline) = game_state.active_guideline {
        if let Some(data) = game_data {
            let center_x = screen_width() / 2.0;
            let rect = UiRect::centered_x(100.0, 500.0, 450.0);
            draw_panel(rect, Color::from_rgba(30, 30, 50, 250));
    
            let inner = rect.inset(spacing::PADDING_LG);
            let mut y = inner.y;
    
            // Title
            draw_text(
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
            let timer_text = data.localization.ui.game.guidelines.timer
                .replace("{:.1}", &format!("{:.1}", time_left));
                
            draw_text(&timer_text, inner.x, y + 20.0, fonts::SIZE_LG, timer_color);
            y += 50.0;
    
            // Guideline info
            draw_text(&data.localization.ui.game.guidelines.label, inner.x, y + 18.0, fonts::SIZE_MD, colors::TEXT_MUTED);
            y += 25.0;
            draw_text(&guideline.title, inner.x, y + 18.0, fonts::SIZE_LG, colors::ACCENT_SKY);
            y += 35.0;
    
            // Description (truncated)
            let desc_preview = if guideline.description.len() > 60 {
                format!("{}...", &guideline.description[..60])
            } else {
                guideline.description.clone()
            };
            draw_text(&desc_preview, inner.x, y + 16.0, fonts::SIZE_SM, colors::TEXT_PRIMARY);
            y += 50.0;
    
            // Detected tells
            draw_text(&data.localization.ui.game.guidelines.tells_label, inner.x, y + 18.0, fonts::SIZE_MD, colors::TEXT_MUTED);
            y += 30.0;
    
            let relevant_tells: Vec<_> = game_state.detected_tells.iter()
                .filter(|t| t.related_guideline == Some(guideline.id))
                .collect();
    
            if relevant_tells.is_empty() {
                draw_text(&data.localization.ui.game.guidelines.no_tells, inner.x + 20.0, y + 16.0, fonts::SIZE_SM, colors::TEXT_MUTED);
                y += 25.0;
            } else {
                for tell in relevant_tells.iter().take(3) {
                    let (intensity_text, intensity_color) = match tell.tell.intensity {
                        data::TellIntensity::Subtle => (&data.localization.ui.game.guidelines.intensity.subtle, colors::TEXT_MUTED),
                        data::TellIntensity::Moderate => (&data.localization.ui.game.guidelines.intensity.moderate, colors::ACCENT_WARNING),
                        data::TellIntensity::Obvious => (&data.localization.ui.game.guidelines.intensity.obvious, colors::FUEL_CRITICAL),
                    };
    
                    let tell_text = format!("• [{}] {}", intensity_text, tell.tell.description);
                    draw_text(&tell_text, inner.x + 20.0, y + 16.0, fonts::SIZE_SM, intensity_color);
                    y += 25.0;
                }
            }
    
            y += 30.0;
    
            // Decision buttons
            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_spacing = 20.0;
    
            // Follow guideline button (left)
            if button(center_x - btn_w - btn_spacing / 2.0, y, btn_w, btn_h, &data.localization.ui.game.guidelines.follow) {
                return UiAction::FollowGuideline;
            }
    
            // Break guideline button (right)
            if button(center_x + btn_spacing / 2.0, y, btn_w, btn_h, &data.localization.ui.game.guidelines.break_guideline) {
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

/// Draw the inventory modal
pub fn draw_inventory_modal(game_state: &GameState, game_data: Option<&GameData>) {
    if let Some(data) = game_data {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 200));
    
        // Panel
        let panel_w = 700.0;
        let panel_h = 550.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;
        let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);
    
        draw_panel_bordered(panel_rect, colors::PANEL_BG, colors::ACCENT_SKY, 3.0);
    
        let inner = panel_rect.inset(spacing::PADDING_LG);
        let mut y = inner.y;
    
        // Title
        draw_text(&data.localization.ui.game.inventory.title, inner.x, y, fonts::SIZE_XL, colors::ACCENT_SKY);
        y += 40.0;
    
        // Help text
        draw_text(&data.localization.ui.game.inventory.hint, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 30.0;
    
        // Item count
        // "Items: {}"
        let count_text = data.localization.ui.game.inventory.count
            .replace("{}", &game_state.inventory.len().to_string());
        draw_text(&count_text, inner.x, y, fonts::SIZE_MD, colors::TEXT_PRIMARY);
        y += 35.0;
    
        // Draw items
        if game_state.inventory.is_empty() {
            draw_text(&data.localization.ui.game.inventory.empty, inner.x, y, fonts::SIZE_MD, colors::TEXT_MUTED);
        } else {
            for (i, item) in game_state.inventory.iter().enumerate() {
                // Item background
                let item_bg = if i % 2 == 0 {
                    Color::from_rgba(45, 45, 68, 255)
                } else {
                    Color::from_rgba(37, 37, 56, 255)
                };
                draw_rectangle(inner.x, y - 5.0, inner.w, 60.0, item_bg);
    
                // Rarity color
                let rarity_color = match item.rarity {
                    Rarity::Common => colors::TEXT_SECONDARY,
                    Rarity::Uncommon => colors::ACCENT_PRIMARY,
                    Rarity::Rare => colors::ACCENT_SKY,
                    Rarity::Legendary => colors::ACCENT_GOLD,
                };
    
                // Item name
                draw_text(&item.name, inner.x + 10.0, y + 15.0, fonts::SIZE_MD, rarity_color);
    
                // Rarity badge
                let rarity_text = format!("{:?}", item.rarity);
                draw_text(&rarity_text, inner.x + 10.0, y + 35.0, fonts::SIZE_XS, colors::TEXT_MUTED);
    
                // Source
                // "from {}"
                let source_text = data.localization.ui.game.inventory.source
                    .replace("{}", &item.source);
                draw_text(&source_text, inner.x + 100.0, y + 35.0, fonts::SIZE_XS, colors::TEXT_MUTED);
    
                // Can use indicator
                if item.can_use {
                    let use_text = &data.localization.ui.game.inventory.click_to_use;
                    draw_text(use_text, inner.x + inner.w - 120.0, y + 25.0, fonts::SIZE_SM, colors::ACCENT_PRIMARY);
                }
    
                y += 65.0;
    
                // Check if we're running out of space
                if y > inner.y + panel_h - 80.0 {
                    draw_text("...", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                    break;
                }
            }
        }
    }
}

/// Draw the rules panel
pub fn draw_rules_panel(game_state: &GameState, game_data: Option<&GameData>) {
    // Semi-transparent overlay
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 200));

    // Panel
    let panel_w = 600.0;
    let panel_h = 500.0;
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

    draw_panel_bordered(panel_rect, colors::PANEL_BG, colors::ACCENT_PRIMARY, 3.0);

    let inner = panel_rect.inset(spacing::PADDING_LG);
    let mut y = inner.y;

    if let Some(data) = game_data {
        // Title
        draw_text(&data.localization.ui.game.rules.title, inner.x, y, fonts::SIZE_XL, colors::ACCENT_PRIMARY);
        y += 40.0;

        // Help text
        draw_text(&data.localization.ui.game.rules.hint, inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 30.0;
    } else {
        // Fallback
        // Title
        draw_text("CURRENT RULES", inner.x, y, fonts::SIZE_XL, colors::ACCENT_PRIMARY);
        y += 40.0;

        // Help text
        draw_text("Press R to close", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 30.0;
    }

    // Draw rules
    for rule in &game_state.current_rules {
        // Rule title with difficulty color
        let difficulty_color = match rule.difficulty {
            data::Difficulty::Easy => colors::FUEL_GOOD,
            data::Difficulty::Medium => colors::ACCENT_WARNING,
            data::Difficulty::Hard => colors::FUEL_LOW,
            data::Difficulty::Expert => colors::FUEL_CRITICAL,
            data::Difficulty::Nightmare => colors::ACCENT_DANGER,
        };

        draw_text(&rule.title, inner.x, y, fonts::SIZE_MD, difficulty_color);
        y += 25.0;

        // Rule description (wrapped if too long)
        let desc = if rule.description.len() > 70 {
            format!("{}...", &rule.description[..70])
        } else {
            rule.description.clone()
        };
        draw_text(&desc, inner.x + 10.0, y, fonts::SIZE_SM, colors::TEXT_SECONDARY);
        y += 30.0;

        // Check if we're running out of space
        if y > inner.y + panel_h - 80.0 {
            draw_text("...", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
            break;
        }
    }
}

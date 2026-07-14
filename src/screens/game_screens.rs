//! Game phase screens: Waiting, Driving, Interaction, DropOff, Guidelines.
//!
//! These screens are shown during active gameplay.

use macroquad::prelude::*;

use crate::data::{self, GameData, PreferenceLevel, Rarity, RouteType, TimePhase};
use crate::state::{DrivingPhase, GamePhase, GameState};
use crate::ui::{
    colors, draw_cockpit_background, draw_glass_button, draw_glass_panel, draw_passenger_portrait,
    draw_small_caps, draw_wrapped_text, fonts, get_fuel_color, layout, spacing, CompletionSummary,
    UiAction, UiRect,
};
use macroquad_toolkit::ui::draw_ui_text;

/// Get a pulsing color for warning text (slow, gentle pulse)
fn pulsing_warning_color() -> Color {
    // 3 second full cycle (0% -> 100% -> 0%)
    let pulse = (get_time() * 2.0 * std::f64::consts::PI / 3.0).sin() as f32;
    // Map from [-1, 1] to [0, 1] for full transparency range
    let alpha = (pulse + 1.0) / 2.0;
    Color::new(
        colors::ACCENT_WARNING.r,
        colors::ACCENT_WARNING.g,
        colors::ACCENT_WARNING.b,
        alpha,
    )
}

fn draw_bottom_taxi_scene(rect: UiRect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.006, 0.008, 0.009, 0.92),
    );
    draw_rectangle(rect.x, rect.y, rect.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.08));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);

    let road_y = rect.y + rect.h * 0.70;
    draw_rectangle(
        rect.x,
        road_y,
        rect.w,
        rect.h * 0.30,
        Color::new(0.020, 0.023, 0.022, 1.0),
    );
    for i in 0..9 {
        let x = rect.x + rect.w * (i as f32 / 9.0);
        draw_rectangle(
            x + rect.w * 0.035,
            road_y + rect.h * 0.16,
            rect.w * 0.045,
            3.0,
            Color::new(0.95, 0.58, 0.08, 0.30),
        );
    }

    for i in 0..5 {
        let x = rect.x + rect.w * (0.18 + i as f32 * 0.16);
        let lamp_y = rect.y + rect.h * (0.20 + (i % 2) as f32 * 0.06);
        draw_line(
            x,
            lamp_y + 24.0,
            x,
            road_y + 28.0,
            2.0,
            Color::new(0.25, 0.22, 0.18, 0.70),
        );
        draw_circle(x, lamp_y, 5.0, colors::CAB_YELLOW);
        draw_circle(x, lamp_y, 24.0, Color::new(0.95, 0.58, 0.08, 0.08));
    }

    let scale = (rect.w / 1100.0).clamp(0.85, 1.35);
    let body_w = 350.0 * scale;
    let body_h = 74.0 * scale;
    let body_x = rect.center_x() - body_w / 2.0;
    let body_y = rect.y + rect.h * 0.48;
    draw_rectangle(
        body_x,
        body_y,
        body_w,
        body_h,
        Color::new(0.74, 0.45, 0.06, 0.95),
    );
    draw_rectangle(
        body_x + body_w * 0.15,
        body_y - body_h * 0.44,
        body_w * 0.46,
        body_h * 0.44,
        Color::new(0.55, 0.34, 0.06, 0.92),
    );
    draw_rectangle(
        body_x + body_w * 0.21,
        body_y - body_h * 0.31,
        body_w * 0.16,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.88),
    );
    draw_rectangle(
        body_x + body_w * 0.40,
        body_y - body_h * 0.31,
        body_w * 0.17,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.88),
    );
    draw_rectangle(
        body_x + body_w * 0.27,
        body_y - body_h * 0.66,
        body_w * 0.22,
        body_h * 0.18,
        colors::CAB_YELLOW,
    );
    draw_ui_text(
        "TAXI",
        body_x + body_w * 0.32,
        body_y - body_h * 0.51,
        fonts::SIZE_SM * scale,
        colors::BLACK,
    );
    draw_rectangle(
        body_x + body_w * 0.03,
        body_y + body_h * 0.46,
        body_w * 0.14,
        body_h * 0.14,
        Color::new(0.95, 0.10, 0.04, 0.60),
    );
    draw_rectangle(
        body_x + body_w * 0.84,
        body_y + body_h * 0.45,
        body_w * 0.11,
        body_h * 0.12,
        Color::new(1.0, 0.86, 0.44, 0.70),
    );
    draw_circle(
        body_x + body_w * 0.17,
        body_y + body_h * 0.78,
        body_h * 0.28,
        colors::BLACK,
    );
    draw_circle(
        body_x + body_w * 0.82,
        body_y + body_h * 0.78,
        body_h * 0.28,
        colors::BLACK,
    );
    draw_circle(
        body_x + body_w * 0.17,
        body_y + body_h * 0.78,
        body_h * 0.13,
        Color::new(0.12, 0.13, 0.12, 1.0),
    );
    draw_circle(
        body_x + body_w * 0.82,
        body_y + body_h * 0.78,
        body_h * 0.13,
        Color::new(0.12, 0.13, 0.12, 1.0),
    );
    draw_rectangle(
        body_x - body_w * 0.10,
        body_y + body_h * 0.88,
        body_w * 1.20,
        6.0,
        Color::new(0.95, 0.58, 0.08, 0.18),
    );
}

fn draw_metric_tile(rect: UiRect, label: &str, value: &str, color: Color) {
    draw_glass_panel(rect, colors::BORDER_DIM);
    let inner = rect.inset(12.0);
    draw_small_caps(
        label,
        inner.x,
        inner.y + 12.0,
        fonts::SIZE_XS,
        colors::TEXT_MUTED,
    );
    draw_ui_text(value, inner.x, inner.y + 42.0, fonts::SIZE_LG, color);
}

// Update signatures to accept player_stats
pub fn draw_game(
    game_state: &GameState,
    game_data: Option<&GameData>,
    player_stats: &crate::state::PlayerStats,
) -> UiAction {
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
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.28).clamp(220.0, 310.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(data) = game_data {
        let panel_w = (screen_width() - 140.0).min(1060.0);
        let panel_h = (scene_rect.y - layout::STATUS_BAR_HEIGHT - 78.0).clamp(390.0, 470.0);
        let panel = UiRect::centered_x(layout::STATUS_BAR_HEIGHT + 36.0, panel_w, panel_h);
        draw_glass_panel(panel, colors::BORDER);
        let inner = panel.inset(spacing::PADDING_MD);
        let left_w = inner.w * 0.56;
        let right_x = inner.x + left_w + 28.0;
        let right_w = inner.w - left_w - 28.0;
        let mut y = inner.y + 16.0;

        draw_small_caps("Dispatch", inner.x, y, fonts::SIZE_SM, colors::CAB_YELLOW);
        y += 36.0;
        draw_ui_text(
            &data.localization.ui.game.waiting.looking,
            inner.x,
            y,
            fonts::SIZE_XXL,
            colors::TEXT_PRIMARY,
        );
        y += 52.0;

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
        let fuel_text = data
            .localization
            .ui
            .game
            .waiting
            .fuel_label
            .replace("{:.0}", &format!("{:.0}", fuel_pct))
            .replace("{}", fuel_status)
            .replace("%", &data.localization.ui.common.percent);
        draw_wrapped_text(
            &fuel_text,
            inner.x,
            y,
            left_w,
            fonts::SIZE_LG,
            26.0,
            fuel_color,
            2,
        );
        y += 46.0;

        let inv_hint = data
            .localization
            .ui
            .game
            .waiting
            .inventory_hint
            .replace("{}", &game_state.inventory.len().to_string());
        draw_wrapped_text(
            &inv_hint,
            inner.x,
            y,
            left_w,
            fonts::SIZE_SM,
            19.0,
            colors::TEXT_MUTED,
            2,
        );

        if fuel_pct < game_state.max_fuel {
            let refuel_y = inner.y + inner.h - 112.0;
            draw_small_caps(
                &data.localization.ui.game.waiting.refuel_title,
                inner.x,
                refuel_y,
                fonts::SIZE_SM,
                colors::TEXT_SECONDARY,
            );

            let btn_w = (left_w - 16.0) / 2.0;
            let btn_h = 44.0;

            let fuel_needed = game_state.max_fuel - fuel_pct;
            let full_cost = (fuel_needed * data.constants.fuel.cost_per_percent) as u32;
            let full_label = data
                .localization
                .ui
                .game
                .waiting
                .full_tank
                .replace("{}", &full_cost.to_string());
            let can_afford_full = game_state.earnings >= full_cost;

            if draw_glass_button(
                UiRect::new(inner.x, refuel_y + 26.0, btn_w, btn_h),
                &full_label,
                colors::ACCENT_SKY,
                can_afford_full,
            ) {
                return UiAction::RefuelFull;
            }

            let partial_amount = 25.0_f32.min(fuel_needed);
            let partial_cost = (partial_amount * data.constants.fuel.cost_per_percent) as u32;
            let partial_label = data
                .localization
                .ui
                .game
                .waiting
                .partial
                .replace("{}", &partial_cost.to_string());
            let can_afford_partial = game_state.earnings >= partial_cost;

            if draw_glass_button(
                UiRect::new(inner.x + btn_w + 16.0, refuel_y + 26.0, btn_w, btn_h),
                &partial_label,
                colors::ACCENT_SKY,
                can_afford_partial,
            ) {
                return UiAction::RefuelPartial;
            }
        }

        let hours = game_state.time_remaining / 60;
        let mins = game_state.time_remaining % 60;
        let tile_gap = 12.0;
        let tile_w = (right_w - tile_gap) / 2.0;
        let tile_h = 86.0;
        draw_metric_tile(
            UiRect::new(right_x, inner.y + 18.0, tile_w, tile_h),
            "Fuel",
            &format!("{:.0}%", fuel_pct),
            fuel_color,
        );
        draw_metric_tile(
            UiRect::new(right_x + tile_w + tile_gap, inner.y + 18.0, tile_w, tile_h),
            "Earnings",
            &format!("${}", game_state.earnings),
            colors::ACCENT_GOLD,
        );
        draw_metric_tile(
            UiRect::new(right_x, inner.y + 18.0 + tile_h + tile_gap, tile_w, tile_h),
            "Time",
            &format!("{}:{:02}", hours, mins),
            colors::TEXT_PRIMARY,
        );
        draw_metric_tile(
            UiRect::new(
                right_x + tile_w + tile_gap,
                inner.y + 18.0 + tile_h + tile_gap,
                tile_w,
                tile_h,
            ),
            "Rides",
            &game_state.rides_completed.to_string(),
            colors::TEXT_PRIMARY,
        );

        let weather_rect = UiRect::new(
            right_x,
            inner.y + 18.0 + (tile_h + tile_gap) * 2.0,
            right_w,
            76.0,
        );
        draw_metric_tile(
            weather_rect,
            "Weather",
            game_state.current_weather.weather_type.name(),
            colors::ACCENT_SKY,
        );

        let earned_enough = game_state.earnings >= game_state.minimum_earnings;
        let find_y = panel.bottom() - 72.0;
        if earned_enough {
            let action_gap = 14.0;
            let action_w = (right_w - action_gap) / 2.0;
            let end_rect = UiRect::new(right_x, find_y, action_w, 50.0);
            if draw_glass_button(end_rect, "Cash Out Shift", colors::FUEL_GOOD, true) {
                return UiAction::EndShift;
            }
            let find_rect = UiRect::new(right_x + action_w + action_gap, find_y, action_w, 50.0);
            if draw_glass_button(
                find_rect,
                &data.localization.ui.game.waiting.find_passenger,
                colors::CAB_YELLOW,
                true,
            ) {
                return UiAction::Continue;
            }
            return UiAction::None;
        }

        let find_rect = UiRect::new(right_x, find_y, right_w, 50.0);
        if draw_glass_button(
            find_rect,
            &data.localization.ui.game.waiting.find_passenger,
            colors::CAB_YELLOW,
            true,
        ) {
            return UiAction::Continue;
        }

        UiAction::None
    } else {
        UiAction::None
    }
}

/// Draw the ride request screen
pub fn draw_ride_request(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.30).clamp(220.0, 320.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(ref passenger) = game_state.current_passenger {
        let panel_w = screen_width().min(1040.0);
        let panel_h = (screen_height() * 0.50).clamp(440.0, 540.0);
        let panel = UiRect::centered_x(layout::STATUS_BAR_HEIGHT + 36.0, panel_w, panel_h);
        draw_glass_panel(panel, colors::BORDER);
        let inner = panel.inset(spacing::PADDING_MD);

        let portrait_size = (inner.h - 12.0).min(inner.w * 0.47).clamp(360.0, 500.0);
        let portrait_rect = UiRect::new(inner.x, inner.y, portrait_size, portrait_size);
        draw_passenger_portrait(portrait_rect, passenger.id);

        let info_x = portrait_rect.x + portrait_rect.w + 28.0;
        let info_w = inner.x + inner.w - info_x;
        let mut y = inner.y + 12.0;

        draw_small_caps(
            "Ride Request",
            info_x,
            y,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        y += 38.0;
        draw_ui_text(
            &passenger.name,
            info_x,
            y,
            fonts::SIZE_XXL,
            colors::CAB_YELLOW,
        );
        y += 30.0;

        let rarity_text = format!("{:?}", passenger.rarity);
        let rarity_color = match passenger.rarity {
            Rarity::Common => colors::TEXT_MUTED,
            Rarity::Uncommon => colors::ACCENT_PRIMARY,
            Rarity::Rare => colors::ACCENT_SKY,
            Rarity::Legendary => colors::ACCENT_GOLD,
        };
        draw_small_caps(&rarity_text, info_x, y, fonts::SIZE_XS, rarity_color);
        y += 34.0;

        y = draw_wrapped_text(
            &passenger.description,
            info_x,
            y,
            info_w,
            fonts::SIZE_MD,
            21.0,
            colors::TEXT_SECONDARY,
            2,
        ) + 14.0;

        let route = format!("{} -> {}", passenger.pickup, passenger.destination);
        y = draw_wrapped_text(
            &route,
            info_x,
            y,
            info_w,
            fonts::SIZE_MD,
            21.0,
            colors::TEXT_PRIMARY,
            2,
        ) + 18.0;

        draw_ui_text(
            &format!("${}", passenger.fare),
            info_x,
            y,
            fonts::SIZE_XL,
            colors::ACCENT_GOLD,
        );
        y += 42.0;

        if let Some(dialogue_text) = game_state.current_passenger_dialogue.as_ref() {
            let preview = if dialogue_text.len() > 100 {
                format!("\"{}...\"", &dialogue_text[..100])
            } else {
                format!("\"{}\"", dialogue_text)
            };
            draw_wrapped_text(
                &preview,
                info_x,
                y,
                info_w,
                fonts::SIZE_SM,
                18.0,
                colors::TEXT_MUTED,
                3,
            );
        }

        let (accept_text, decline_text) = if let Some(data) = game_data {
            (
                data.localization.ui.common.accept_space.as_str(),
                data.localization.ui.common.decline_esc.as_str(),
            )
        } else {
            ("Accept (SPACE)", "Decline (ESC)")
        };

        let btn_h = 48.0;
        let gap = 14.0;
        let btn_w = (info_w - gap) / 2.0;
        let btn_y = panel.bottom() - spacing::PADDING_MD - btn_h;
        if draw_glass_button(
            UiRect::new(info_x, btn_y, btn_w, btn_h),
            accept_text,
            colors::ACCENT_PRIMARY,
            true,
        ) {
            return UiAction::AcceptRide;
        }
        if draw_glass_button(
            UiRect::new(info_x + btn_w + gap, btn_y, btn_w, btn_h),
            decline_text,
            colors::ACCENT_DANGER,
            true,
        ) {
            return UiAction::DeclineRide;
        }
    }
    UiAction::None
}

/// Draw the driving/route selection screen
pub fn draw_driving(
    game_state: &GameState,
    game_data: Option<&GameData>,
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

    if let Some(data) = game_data {
        let margin = 26.0;
        let top = layout::STATUS_BAR_HEIGHT + 22.0;
        let gap = 18.0;
        let passenger_w = (screen_width() * 0.17).clamp(260.0, 330.0);
        let left_w = screen_width() - margin * 2.0 - gap - passenger_w;
        let left_x = margin;
        let right_x = left_x + left_w + gap;
        let content_bottom = scene_rect.y - 22.0;

        let header = UiRect::new(left_x, top, left_w, 88.0);
        draw_glass_panel(header, colors::BORDER_DIM);
        let header_inner = header.inset(14.0);

        let phase_text = match game_state.driving_phase {
            Some(DrivingPhase::Pickup) => &data.localization.ui.game.driving.pickup,
            Some(DrivingPhase::Destination) => &data.localization.ui.game.driving.destination,
            None => "Driving...",
        };

        draw_ui_text(
            phase_text,
            header_inner.x,
            header_inner.y + 26.0,
            fonts::SIZE_XL,
            colors::TEXT_PRIMARY,
        );

        if let Some(need_state) = &game_state.current_passenger_need_state {
            let stability = crate::engine::PassengerStateMachine::get_stability_percent(need_state);
            let critical = crate::engine::PassengerStateMachine::is_critical(need_state);
            let state_text = format!("Passenger stability: {}%", stability);
            let state_color = if critical {
                colors::FUEL_CRITICAL
            } else {
                colors::FUEL_GOOD
            };
            draw_small_caps(
                &state_text,
                header_inner.x,
                header_inner.y + 52.0,
                fonts::SIZE_XS,
                state_color,
            );
        }

        if let Some(dialogue) = &game_state.current_dialogue {
            let speaker = match dialogue.speaker {
                crate::state::DialogueSpeaker::Passenger => "Passenger",
                crate::state::DialogueSpeaker::Driver => "Driver",
                crate::state::DialogueSpeaker::Narrator => "Dispatch",
            };
            let elapsed = (get_time() - dialogue.timestamp).max(0.0);
            let line = format!("{} ({:.0}s ago): {}", speaker, elapsed, dialogue.text);
            let preview = if line.len() > 86 {
                format!("{}...", &line[..86])
            } else {
                line
            };
            draw_ui_text(
                &preview,
                header_inner.x,
                header_inner.y + 68.0,
                fonts::SIZE_XS,
                colors::TEXT_SECONDARY,
            );
        }

        if let Some(ride) = &game_state.current_ride {
            let target = match game_state.driving_phase {
                Some(DrivingPhase::Pickup) => &ride.pickup_location,
                Some(DrivingPhase::Destination) => &ride.destination_location,
                None => &ride.destination_location,
            };
            let elapsed = (get_time() - ride.start_time).max(0.0);
            let ride_text = format!("{} -> {} | {:.0}s", ride.passenger.name, target, elapsed);
            draw_small_caps(
                &ride_text,
                header_inner.x + left_w * 0.54,
                header_inner.y + 52.0,
                fonts::SIZE_XS,
                colors::TEXT_MUTED,
            );
        }

        let r = &data.localization.ui.game.driving.routes;
        let routes = [
            (RouteType::Normal, &r.normal, &r.normal_desc, "[1]"),
            (RouteType::Shortcut, &r.shortcut, &r.shortcut_desc, "[2]"),
            (RouteType::Scenic, &r.scenic, &r.scenic_desc, "[3]"),
            (RouteType::Police, &r.police, &r.police_desc, "[4]"),
        ];

        let passenger_opt = game_state.current_passenger.as_ref();
        let passenger_knowledge = passenger_opt
            .map(|passenger| player_stats.get_almanac_entry(passenger.id).knowledge_level)
            .unwrap_or(0);
        let reveal_route_preferences = passenger_knowledge >= 2;
        let reveal_preference_reasons = passenger_knowledge >= 3;
        let mut route_y = header.bottom() + 10.0;
        let route_h = ((content_bottom - route_y - 24.0) / 4.0 - 8.0).clamp(66.0, 82.0);
        for (i, (route_type, name, desc, key)) in routes.iter().enumerate() {
            let is_blocked = game_state
                .environmental_hazards
                .iter()
                .any(|h| h.blocks_route(*route_type));

            let mut weather_warning = String::new();
            if matches!(
                game_state.current_weather.intensity,
                data::WeatherIntensity::Heavy
            ) && *route_type == RouteType::Shortcut
            {
                let w_type = game_state.current_weather.weather_type.name();
                weather_warning = data
                    .localization
                    .ui
                    .game
                    .driving
                    .weather_warning
                    .replace("{}", w_type);
            }
            if matches!(
                game_state.time_of_day.phase,
                TimePhase::Night | TimePhase::Latenight
            ) && *route_type == RouteType::Shortcut
            {
                if !weather_warning.is_empty() {
                    weather_warning.push_str(" + Night");
                } else {
                    weather_warning = data.localization.ui.game.driving.night_warning.clone();
                }
            }

            let route_usage = player_stats.get_route_usage(*route_type);
            let route_risk_known = route_usage > 0;
            let preference = passenger_opt.and_then(|p| p.get_route_preference(*route_type));
            let border = if is_blocked {
                colors::ACCENT_DANGER
            } else if reveal_route_preferences {
                if let Some(pref) = preference {
                    match pref.preference {
                        PreferenceLevel::Loves | PreferenceLevel::Likes => colors::FUEL_GOOD,
                        PreferenceLevel::Dislikes => colors::ACCENT_WARNING,
                        PreferenceLevel::Fears => colors::FUEL_CRITICAL,
                        PreferenceLevel::Neutral => colors::BORDER,
                    }
                } else {
                    colors::BORDER
                }
            } else {
                colors::BORDER
            };

            let route_title_color = if reveal_route_preferences {
                if let Some(pref) = preference {
                    match pref.preference {
                        PreferenceLevel::Loves | PreferenceLevel::Likes => colors::FUEL_GOOD,
                        PreferenceLevel::Dislikes => colors::ACCENT_WARNING,
                        PreferenceLevel::Fears => colors::FUEL_CRITICAL,
                        PreferenceLevel::Neutral => colors::TEXT_PRIMARY,
                    }
                } else if i == 0 {
                    colors::CAB_YELLOW
                } else {
                    colors::TEXT_PRIMARY
                }
            } else if i == 0 {
                colors::CAB_YELLOW
            } else {
                colors::TEXT_PRIMARY
            };

            let preference_label = if reveal_route_preferences {
                preference.and_then(|pref| {
                    let pref_text = pref.preference.display_text();
                    let pref_label = pref_text
                        .chars()
                        .filter(|ch| ch.is_ascii())
                        .collect::<String>();
                    let pref_label = pref_label.trim();
                    if pref_label.is_empty() {
                        None
                    } else {
                        let pref_color = match pref.preference {
                            PreferenceLevel::Loves | PreferenceLevel::Likes => colors::FUEL_GOOD,
                            PreferenceLevel::Neutral => colors::TEXT_MUTED,
                            PreferenceLevel::Dislikes => colors::ACCENT_WARNING,
                            PreferenceLevel::Fears => colors::FUEL_CRITICAL,
                        };
                        Some((format!("Client {}", pref_label), pref_color))
                    }
                })
            } else {
                None
            };

            let route_detail = if reveal_preference_reasons {
                preference.map(|pref| pref.reason.as_str()).unwrap_or(*desc)
            } else {
                *desc
            };

            let tint = if is_blocked {
                Color::new(0.30, 0.04, 0.03, 0.28)
            } else if reveal_route_preferences {
                if let Some(pref) = preference {
                    match pref.preference {
                        PreferenceLevel::Loves => Color::new(0.06, 0.28, 0.08, 0.18),
                        PreferenceLevel::Likes => Color::new(0.06, 0.22, 0.10, 0.16),
                        PreferenceLevel::Neutral => Color::new(0.0, 0.0, 0.0, 0.0),
                        PreferenceLevel::Dislikes => Color::new(0.32, 0.18, 0.04, 0.14),
                        PreferenceLevel::Fears => Color::new(0.32, 0.04, 0.04, 0.18),
                    }
                } else {
                    Color::new(0.0, 0.0, 0.0, 0.0)
                }
            } else {
                Color::new(0.0, 0.0, 0.0, 0.0)
            };

            let visible_count = if route_usage >= 5 {
                3
            } else if route_usage >= 3 {
                2
            } else if route_usage >= 1 {
                1
            } else {
                0
            };

            let card = UiRect::new(left_x, route_y, left_w, route_h);
            if !is_blocked && draw_glass_button(card, "", border, true) {
                return UiAction::SelectRoute(i);
            }
            if is_blocked {
                draw_glass_button(card, "", colors::ACCENT_DANGER, false);
            }
            draw_rectangle(card.x, card.y, card.w, card.h, tint);
            draw_rectangle_lines(card.x, card.y, card.w, card.h, 1.0, border);

            let route_constants = &data.constants.game_constants;
            let (time_cost, fuel_cost, risk) = match route_type {
                RouteType::Normal => (
                    route_constants.time_cost_normal,
                    route_constants.fuel_cost_normal,
                    route_constants.risk_normal,
                ),
                RouteType::Shortcut => (
                    route_constants.time_cost_shortcut,
                    route_constants.fuel_cost_shortcut,
                    route_constants.risk_shortcut,
                ),
                RouteType::Scenic => (
                    route_constants.time_cost_scenic,
                    route_constants.fuel_cost_scenic,
                    route_constants.risk_scenic,
                ),
                RouteType::Police => (
                    route_constants.time_cost_police,
                    route_constants.fuel_cost_police,
                    route_constants.risk_police,
                ),
            };

            let (risk_label, risk_color) = if !route_risk_known {
                ("Unknown", colors::TEXT_MUTED)
            } else if risk <= data.constants.risk.safe {
                ("Safe", colors::FUEL_GOOD)
            } else if risk <= data.constants.risk.low_risk {
                ("Low", colors::FUEL_GOOD)
            } else if risk <= data.constants.risk.medium_risk {
                ("Medium", colors::ACCENT_WARNING)
            } else {
                ("High", colors::FUEL_CRITICAL)
            };

            if is_blocked {
                draw_ui_text(
                    &data.localization.ui.game.driving.blocked,
                    card.x + 18.0,
                    card.y + 26.0,
                    fonts::SIZE_MD,
                    colors::FUEL_CRITICAL,
                );
                if let Some(hazard) = game_state
                    .environmental_hazards
                    .iter()
                    .find(|h| h.blocks_route(*route_type))
                {
                    draw_wrapped_text(
                        &hazard.description,
                        card.x + 18.0,
                        card.y + 48.0,
                        card.w - 36.0,
                        fonts::SIZE_XS,
                        16.0,
                        colors::ACCENT_WARNING,
                        2,
                    );
                }
            } else {
                draw_ui_text(
                    key,
                    card.x + 16.0,
                    card.y + 25.0,
                    fonts::SIZE_MD,
                    colors::TEXT_MUTED,
                );
                draw_small_caps(
                    name,
                    card.x + 54.0,
                    card.y + 25.0,
                    fonts::SIZE_MD,
                    route_title_color,
                );
                if let Some((label, color)) = &preference_label {
                    draw_small_caps(label, card.x + 176.0, card.y + 25.0, fonts::SIZE_XS, *color);
                }
                draw_ui_text(
                    route_detail,
                    card.x + 54.0,
                    card.y + 45.0,
                    fonts::SIZE_XS,
                    colors::TEXT_SECONDARY,
                );
                draw_small_caps(
                    &format!("Duration  {} min", time_cost),
                    card.x + card.w - 154.0,
                    card.y + 25.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
                draw_small_caps(
                    &format!("Fuel  -{}%", fuel_cost),
                    card.x + card.w - 154.0,
                    card.y + 41.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
                draw_small_caps(
                    &format!("Hazards  {}", risk_label),
                    card.x + card.w - 154.0,
                    card.y + 57.0,
                    fonts::SIZE_XS,
                    risk_color,
                );

                if !weather_warning.is_empty() {
                    draw_ui_text(
                        &weather_warning,
                        card.x + 54.0,
                        card.y + 58.0,
                        fonts::SIZE_XS,
                        pulsing_warning_color(),
                    );
                }
            }

            use crate::engine::RouteService;
            let seed = game_state.rides_completed as u64
                + game_state
                    .current_passenger
                    .as_ref()
                    .map(|p| p.id as u64)
                    .unwrap_or(0)
                + i as u64;
            let risk_tags = RouteService::generate_risk_tags(
                *route_type,
                Some(&game_state.current_weather),
                Some(&game_state.time_of_day),
                game_state.current_passenger.as_ref(), // Pass passenger for context
                Some(seed),
            );

            let tag_x = card.x + card.w - 244.0;
            let tag_y = card.y + 25.0;
            for (tag_idx, tag) in risk_tags.iter().enumerate() {
                let is_visible = tag_idx < visible_count;
                let (text, color) = if is_visible {
                    (
                        format!("{}: {}", tag.name(), tag.description()),
                        colors::ACCENT_WARNING,
                    )
                } else {
                    ("???".to_string(), colors::TEXT_MUTED)
                };
                let label = if text.len() > 28 {
                    format!("{}...", &text[..28])
                } else {
                    text
                };
                draw_ui_text(
                    &label,
                    tag_x,
                    tag_y + (tag_idx as f32 * 15.0),
                    fonts::SIZE_XS,
                    color,
                );
            }

            route_y += route_h + 8.0;
        }

        if let Some(passenger) = &game_state.current_passenger {
            let passenger_rect = UiRect::new(right_x, top, passenger_w, content_bottom - top);
            crate::ui::PassengerCard::draw(
                passenger,
                passenger_rect,
                false,
                game_state.current_passenger_dialogue.as_ref(),
                game_data,
            );
        }
    }

    UiAction::None
}

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
        let rect = UiRect::centered_x(layout::STATUS_BAR_HEIGHT + 34.0, rect_w, rect_h);
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
        let rect = UiRect::centered_x(150.0, 500.0, 150.0);
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

/// Draw the dropoff screen
pub fn draw_dropoff(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.27).clamp(210.0, 300.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(ref completion) = game_state.last_ride_completion {
        let rect_w = (screen_width() - 140.0).min(880.0);
        let rect_h = (scene_rect.y - layout::STATUS_BAR_HEIGHT - 76.0).clamp(440.0, 520.0);
        let rect = UiRect::centered_x(layout::STATUS_BAR_HEIGHT + 34.0, rect_w, rect_h);
        // CompletionSummary now needs game_data
        let completion_action = CompletionSummary::draw(completion, rect, game_data);

        if let Some(route) = game_state.route_history.last() {
            let route_text = format!(
                "Last leg: {:?} {:?} | fuel -{} | time -{} | {:.0}s ago",
                route.driving_phase,
                route.route_type,
                route.fuel_cost,
                route.time_cost,
                (get_time() - route.timestamp).max(0.0)
            );
            draw_ui_text(
                &route_text,
                rect.x + 16.0,
                rect.bottom() - 70.0,
                12.0,
                colors::TEXT_MUTED,
            );
        }

        // Show trade offer if available
        if let Some((ref passenger_name, ref offered_item)) = game_state.pending_trade {
            if let Some(data) = game_data {
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.50),
                );
                let trade_rect = UiRect::centered_x(146.0, screen_width().min(460.0), 330.0);
                draw_glass_panel(trade_rect, colors::ACCENT_SKY);

                let inner = trade_rect.inset(spacing::PADDING_MD);

                draw_ui_text(
                    &data.localization.ui.game.trade.title,
                    inner.x,
                    inner.y + 24.0,
                    fonts::SIZE_LG,
                    colors::ACCENT_SKY,
                );

                // Message
                let msg = data
                    .localization
                    .ui
                    .game
                    .trade
                    .wants_to_trade
                    .replace("{}", passenger_name);
                draw_ui_text(
                    &msg,
                    inner.x,
                    inner.y + 50.0,
                    fonts::SIZE_MD,
                    colors::TEXT_PRIMARY,
                );

                // Offered item
                let rarity_color = match offered_item.rarity {
                    Rarity::Common => colors::TEXT_MUTED,
                    Rarity::Uncommon => colors::ACCENT_SKY,
                    Rarity::Rare => Color::from_hex(0x87CEEB),
                    Rarity::Legendary => colors::ACCENT_WARNING,
                };

                let offer_text = data
                    .localization
                    .ui
                    .game
                    .trade
                    .offering
                    .replace("{}", &offered_item.name);
                draw_ui_text(
                    &offer_text,
                    inner.x,
                    inner.y + 80.0,
                    fonts::SIZE_MD,
                    rarity_color,
                );

                // Show what they want if available
                draw_ui_text(
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
                    draw_ui_text(
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
                        if draw_glass_button(
                            UiRect::new(center_x - btn_w / 2.0, item_btn_y, btn_w, 35.0),
                            &item.name,
                            colors::ACCENT_SKY,
                            true,
                        ) {
                            return UiAction::AcceptTrade(i);
                        }
                    }

                    let decline_y = btn_y + (150.0);
                    if draw_glass_button(
                        UiRect::new(center_x - btn_w / 2.0, decline_y, btn_w, btn_h),
                        &data.localization.ui.game.trade.decline,
                        colors::ACCENT_DANGER,
                        true,
                    ) {
                        return UiAction::DeclineTrade;
                    }
                } else {
                    // No items to trade
                    draw_ui_text(
                        &data.localization.ui.game.trade.empty,
                        inner.x,
                        btn_y,
                        fonts::SIZE_SM,
                        colors::ACCENT_WARNING,
                    );
                    if draw_glass_button(
                        UiRect::new(center_x - btn_w / 2.0, btn_y + 30.0, btn_w, btn_h),
                        &data.localization.ui.common.continue_text,
                        colors::ACCENT_SKY,
                        true,
                    ) {
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
            let rect = UiRect::centered_x(layout::STATUS_BAR_HEIGHT + 34.0, rect_w, rect_h);
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

/// Draw the inventory modal
pub fn draw_inventory_modal(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    if let Some(data) = game_data {
        // Semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(0, 0, 0, 200),
        );

        // Panel
        let panel_w = (screen_width() - 180.0).min(920.0);
        let panel_h = (screen_height() - 180.0).min(640.0);
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;
        let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

        draw_glass_panel(panel_rect, colors::ACCENT_SKY);

        let inner = panel_rect.inset(spacing::PADDING_LG);
        let mut y = inner.y;

        // Title
        draw_ui_text(
            &data.localization.ui.game.inventory.title,
            inner.x,
            y,
            fonts::SIZE_XL,
            colors::ACCENT_SKY,
        );
        y += 40.0;

        // Help text
        draw_ui_text(
            &data.localization.ui.game.inventory.hint,
            inner.x,
            y,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        y += 30.0;

        // Item count
        // "Items: {}"
        let count_text = data
            .localization
            .ui
            .game
            .inventory
            .count
            .replace("{}", &game_state.inventory.len().to_string());
        draw_ui_text(
            &count_text,
            inner.x,
            y,
            fonts::SIZE_MD,
            colors::TEXT_PRIMARY,
        );
        y += 35.0;

        // Draw items
        if game_state.inventory.is_empty() {
            draw_ui_text(
                &data.localization.ui.game.inventory.empty,
                inner.x,
                y,
                fonts::SIZE_MD,
                colors::TEXT_MUTED,
            );
        } else {
            let row_h = 72.0;
            for (i, item) in game_state.inventory.iter().enumerate() {
                // Item background
                let item_bg = if i % 2 == 0 {
                    Color::new(0.08, 0.10, 0.10, 0.76)
                } else {
                    Color::new(0.05, 0.065, 0.065, 0.76)
                };
                draw_rectangle(inner.x, y - 5.0, inner.w, row_h, item_bg);
                draw_rectangle_lines(inner.x, y - 5.0, inner.w, row_h, 1.0, colors::BORDER_DIM);

                // Rarity color
                let rarity_color = match item.rarity {
                    Rarity::Common => colors::TEXT_SECONDARY,
                    Rarity::Uncommon => colors::ACCENT_PRIMARY,
                    Rarity::Rare => colors::ACCENT_SKY,
                    Rarity::Legendary => colors::ACCENT_GOLD,
                };

                // Item name
                draw_ui_text(
                    &item.name,
                    inner.x + 10.0,
                    y + 15.0,
                    fonts::SIZE_MD,
                    rarity_color,
                );

                // Rarity badge
                let rarity_text = format!("{:?}", item.rarity);
                draw_ui_text(
                    &rarity_text,
                    inner.x + 10.0,
                    y + 35.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );

                // Source
                // "from {}"
                let source_text = data
                    .localization
                    .ui
                    .game
                    .inventory
                    .source
                    .replace("{}", &item.source);
                draw_ui_text(
                    &source_text,
                    inner.x + inner.w * 0.34,
                    y + 35.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );

                // Can use indicator
                if item.can_use {
                    let use_text = &data.localization.ui.game.inventory.click_to_use;
                    if draw_glass_button(
                        UiRect::new(inner.x + inner.w - 156.0, y + 9.0, 136.0, 40.0),
                        use_text,
                        colors::ACCENT_SKY,
                        true,
                    ) {
                        return UiAction::UseItem(i);
                    }
                }

                y += row_h + 10.0;

                // Check if we're running out of space
                if y > inner.y + panel_h - 80.0 {
                    draw_ui_text(
                        &data.localization.ui.game.inventory.more,
                        inner.x,
                        y,
                        fonts::SIZE_SM,
                        colors::TEXT_MUTED,
                    );
                    break;
                }
            }
        }
    }

    UiAction::None
}

/// Draw the rules panel
pub fn draw_rules_panel(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    // Semi-transparent overlay
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(0, 0, 0, 200),
    );

    // Panel
    let panel_w = (screen_width() - 180.0).min(900.0);
    let panel_h = (screen_height() - 180.0).min(620.0);
    let panel_x = (screen_width() - panel_w) / 2.0;
    let panel_y = (screen_height() - panel_h) / 2.0;
    let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

    draw_glass_panel(panel_rect, colors::ACCENT_PRIMARY);

    let inner = panel_rect.inset(spacing::PADDING_LG);
    let mut y = inner.y + 24.0;

    if let Some(data) = game_data {
        // Title
        draw_ui_text(
            &data.localization.ui.game.rules.title,
            inner.x,
            y,
            fonts::SIZE_XL,
            colors::ACCENT_PRIMARY,
        );
        y += 42.0;

        // Help text
        draw_ui_text(
            &data.localization.ui.game.rules.hint,
            inner.x,
            y,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        y += 42.0;
    } else {
        // Fallback
        // Title
        draw_ui_text(
            "CURRENT RULES",
            inner.x,
            y,
            fonts::SIZE_XL,
            colors::ACCENT_PRIMARY,
        );
        y += 42.0;

        // Help text
        draw_ui_text(
            "Press R to close",
            inner.x,
            y,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        y += 42.0;
    }

    if game_state.current_passenger.is_some() {
        draw_small_caps(
            "CAB CONTROLS",
            inner.x,
            y,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        y += 24.0;

        let controls = cab_controls_for_rules_panel(game_state.game_phase);
        let cols = 4;
        let gap = 10.0;
        let btn_h = 38.0;
        let btn_w = (inner.w - gap * (cols as f32 - 1.0)) / cols as f32;
        for (idx, (key, label, action_key, enabled)) in controls.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;
            let rect = UiRect::new(
                inner.x + col as f32 * (btn_w + gap),
                y + row as f32 * (btn_h + gap),
                btn_w,
                btn_h,
            );
            let text = format!("[{}] {}", key, label);
            if draw_glass_button(rect, &text, colors::BORDER, *enabled) {
                return UiAction::PerformRuleAction((*action_key).to_string());
            }
        }
        y += 2.0 * (btn_h + gap) + 20.0;
    }

    let rule_card_h = 92.0;
    // Draw rules
    for (rule_idx, rule) in game_state.current_rules.iter().enumerate() {
        let card = UiRect::new(inner.x, y, inner.w, rule_card_h);
        draw_glass_panel(card, colors::BORDER_DIM);

        // Rule title with difficulty color
        let difficulty_color = match rule.difficulty {
            data::Difficulty::Easy => colors::FUEL_GOOD,
            data::Difficulty::Medium => colors::ACCENT_WARNING,
            data::Difficulty::Hard => colors::FUEL_LOW,
            data::Difficulty::Expert => colors::FUEL_CRITICAL,
            data::Difficulty::Nightmare => colors::ACCENT_DANGER,
        };

        let key = format!("[{}]", rule_idx + 1);
        draw_small_caps(
            &key,
            card.x + 14.0,
            card.y + 38.0,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );
        draw_ui_text(
            &rule.title,
            card.x + 58.0,
            card.y + 34.0,
            fonts::SIZE_MD,
            difficulty_color,
        );

        // Rule description (wrapped if too long)
        let desc = if rule.description.len() > 70 {
            format!("{}...", &rule.description[..70])
        } else {
            rule.description.clone()
        };
        draw_ui_text(
            &desc,
            card.x + 58.0,
            card.y + 62.0,
            fonts::SIZE_SM,
            colors::TEXT_SECONDARY,
        );
        y = card.bottom() + 14.0;

        // Check if we're running out of space
        if y > inner.y + panel_h - 80.0 {
            draw_ui_text("...", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
            break;
        }
    }

    UiAction::None
}

fn cab_controls_for_rules_panel(
    game_phase: crate::state::GamePhase,
) -> [(&'static str, &'static str, &'static str, bool); 8] {
    let ride_active = matches!(
        game_phase,
        crate::state::GamePhase::RideRequest
            | crate::state::GamePhase::Driving
            | crate::state::GamePhase::Interaction
            | crate::state::GamePhase::GuidelineDecision
            | crate::state::GamePhase::DropOff
    );
    [
        ("E", "Eye Contact", "eye_contact", ride_active),
        ("M", "Music", "play_music", ride_active),
        (
            "T",
            "Accept Tip",
            "accept_tip",
            game_phase == crate::state::GamePhase::DropOff,
        ),
        ("W", "Open Window", "open_window", ride_active),
        ("Y", "Wipers", "use_wipers", ride_active),
        ("H", "Headlights Off", "drive_dark", ride_active),
        ("A", "AC", "use_ac", ride_active),
        (
            "S",
            "Stop Cab",
            "stop_vehicle",
            matches!(
                game_phase,
                crate::state::GamePhase::Driving | crate::state::GamePhase::Interaction
            ),
        ),
    ]
}

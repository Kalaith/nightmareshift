//! Menu screens: Main Menu, Loading, Briefing.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::GameData;
use crate::state::{GameState, PlayerStats};
use crate::ui::{UiAction, colors};

/// Draw the loading screen
pub fn draw_loading(game_data: Option<&GameData>) -> UiAction {
    let text = if let Some(data) = game_data {
        &data.localization.system.loading
    } else {
        "Loading..."
    };
    
    let font_size = 32.0;
    let text_width = measure_text(text, None, font_size as u16, 1.0).width;
    draw_text(
        text,
        screen_width() / 2.0 - text_width / 2.0,
        screen_height() / 2.0,
        font_size,
        WHITE,
    );
    UiAction::None
}

/// Draw the main menu
pub fn draw_main_menu(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    // Default strings if data missing (shouldn't happen)
    let title_text = if let Some(d) = game_data { &d.localization.ui.main_menu.title } else { "NIGHTMARE SHIFT" };
    let subtitle_text = if let Some(d) = game_data { &d.localization.ui.main_menu.subtitle } else { "Survive the night." };
    
    // Title
    let font_size = 48.0;
    let text_width = measure_text(title_text, None, font_size as u16, 1.0).width;
    draw_text(
        title_text,
        screen_width() / 2.0 - text_width / 2.0,
        150.0,
        font_size,
        colors::ACCENT_DANGER,
    );

    // Subtitle
    let sub_size = 20.0;
    let sub_width = measure_text(subtitle_text, None, sub_size as u16, 1.0).width;
    draw_text(
        subtitle_text,
        screen_width() / 2.0 - sub_width / 2.0,
        190.0,
        sub_size,
        colors::TEXT_SECONDARY,
    );

    if let Some(data) = game_data {
        // Stats
        let stats_y = 250.0;

 
        // Note: replace matches all, replacen limits count. 
        // The json is "Shifts Completed: {} | Total Earnings: ${} | Rides: {}"
        // 1st {} -> shifts, 2nd {} -> earnings, 3rd {} -> rides
        // Standard .replace() will replace ALL {} with the first arg.
        // We need formatted string or sequential replacement.
        // Since format! works with literals, we can't easily use it with runtime strings without hacks.
        // Let's use simple string replacement chain with unique placeholder or just assume order works if we use replacen(..., 1) sequentially.
        
        let stats = data.localization.ui.main_menu.stats
            .replacen("{}", &player_stats.total_shifts_completed.to_string(), 1)
            .replacen("{}", &player_stats.total_earnings.to_string(), 1)
            .replacen("{}", &player_stats.total_rides_completed.to_string(), 1);

        let stats_width = measure_text(&stats, None, 16, 1.0).width;
        draw_text(
            &stats,
            screen_width() / 2.0 - stats_width / 2.0,
            stats_y,
            16.0,
            colors::TEXT_MUTED,
        );

        // Achievements
        let unlocked_count = player_stats.achievements.iter().filter(|a| a.unlocked).count();
        let total_count = player_stats.achievements.len();
        let achievements_text = data.localization.ui.main_menu.achievements
            .replacen("{}", &unlocked_count.to_string(), 1)
            .replacen("{}", &total_count.to_string(), 1);
            
        let achievements_width = measure_text(&achievements_text, None, 16, 1.0).width;
        draw_text(
            &achievements_text,
            screen_width() / 2.0 - achievements_width / 2.0,
            stats_y + 25.0,
            16.0,
            colors::ACCENT_GOLD,
        );

        // Start button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() / 2.0 - 50.0,
            300.0,
            50.0,
            &data.localization.ui.main_menu.start_space
        ) {
            return UiAction::StartGame;
        }

        // Meta-progression buttons
        let button_y = screen_height() / 2.0 + 20.0;
        let button_spacing = 60.0;

        // Skill Tree button
        let skill_btn_text = data.localization.ui.meta.skill_tree.button
            .replace("{}", &player_stats.bank_balance.to_string());
        
        if button(
            screen_width() / 2.0 - 150.0,
            button_y,
            300.0,
            50.0,
            &skill_btn_text
        ) {
            return UiAction::OpenSkillTree;
        }

        // Almanac button
        let almanac_btn_text = data.localization.ui.meta.almanac.button
            .replace("{}", &player_stats.lore_fragments.to_string());

        if button(
            screen_width() / 2.0 - 150.0,
            button_y + button_spacing,
            300.0,
            50.0,
            &almanac_btn_text
        ) {
            return UiAction::OpenAlmanac;
        }

        // Leaderboard button
        if button(
            screen_width() / 2.0 - 150.0,
            button_y + button_spacing * 2.0,
            300.0,
            50.0,
            &data.localization.ui.meta.leaderboard.button
        ) {
            return UiAction::OpenLeaderboard;
        }
    }

    UiAction::None
}

/// Draw the briefing screen
pub fn draw_briefing(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.briefing.title;
        let font_size = 36.0;
        draw_text(title, 50.0, 60.0, font_size, colors::ACCENT_WARNING);

        // Rules
        draw_text(&data.localization.ui.briefing.rules_title, 50.0, 120.0, 24.0, WHITE);

        let mut y = 160.0;
        for (i, rule) in game_state.current_rules.iter().enumerate() {
            let rule_text = format!("{}. {} - {}", i + 1, rule.title, rule.description);
            draw_text(&rule_text, 70.0, y, 18.0, colors::TEXT_SECONDARY);
            y += 30.0;
        }

        // Weather
        y += 20.0;
        // LOCALIZATION TODO: Weather description localization
        // For now, construct manually but using "Weather:" label
        let weather_label = &data.localization.ui.briefing.weather_title;
        let weather_text = format!(
            "{} {} {} - {}",
            weather_label,
            game_state.current_weather.icon,
            format!("{:?}", game_state.current_weather.weather_type),
            game_state.current_weather.description
        );
        draw_text(&weather_text, 50.0, y, 18.0, colors::ACCENT_SKY);

        // Start button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() - 80.0,
            300.0,
            50.0,
            &data.localization.ui.briefing.begin_space
        ) {
            return UiAction::StartGame;
        }
    }
    
    UiAction::None
}

/// Draw the game over screen
pub fn draw_game_over(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;
    
    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.game_over.title;
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            150.0,
            title_size,
            colors::FUEL_CRITICAL,
        );

        // Reason
        if let Some(ref reason) = game_state.game_over_reason {
            let reason_width = measure_text(reason, None, 20, 1.0).width;
            draw_text(
                reason,
                center_x - reason_width / 2.0,
                220.0,
                20.0,
                colors::TEXT_SECONDARY,
            );
        }

        // Stats
        let score = game_state.calculate_score(&data.constants);
        
        // "Earnings: ${} | Rides: {} | Score: {}"
        let stats = data.localization.ui.game_over.stats
            .replacen("{}", &game_state.earnings.to_string(), 1)
            .replacen("{}", &game_state.rides_completed.to_string(), 1)
            .replacen("{}", &score.to_string(), 1);

        let stats_width = measure_text(&stats, None, 18, 1.0).width;
        draw_text(
            &stats,
            center_x - stats_width / 2.0,
            280.0,
            18.0,
            WHITE,
        );

        // Try Again Button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() - 120.0,
            300.0,
            50.0,
            &data.localization.ui.common.try_again
        ) {
            return UiAction::TryAgain;
        }
    }
    
    UiAction::None
}

/// Draw the success screen
pub fn draw_success(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.success.title;
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            150.0,
            title_size,
            colors::ACCENT_GOLD,
        );

        // Subtitle
        let subtitle = &data.localization.ui.success.subtitle;
        let sub_width = measure_text(subtitle, None, 24, 1.0).width;
        draw_text(
            subtitle,
            center_x - sub_width / 2.0,
            200.0,
            24.0,
            colors::ACCENT_PRIMARY,
        );

        // Stats
        let y = 260.0;
        
        let earnings_text = data.localization.ui.success.total_earnings
            .replace("{}", &game_state.earnings.to_string());
        draw_text(
            &earnings_text,
            center_x - 150.0,
            y,
            20.0,
            colors::ACCENT_GOLD,
        );
        
        let rides_text = data.localization.ui.success.rides_completed
            .replace("{}", &game_state.rides_completed.to_string());
        draw_text(
            &rides_text,
            center_x - 150.0,
            y + 30.0,
            20.0,
            WHITE,
        );

        let bonus_text = data.localization.ui.success.survival_bonus
            .replace("{}", &data.constants.game_constants.survival_bonus.to_string());
        draw_text(
            &bonus_text,
            center_x - 150.0,
            y + 60.0,
            20.0,
            colors::FUEL_GOOD,
        );
        
        let score_text = data.localization.ui.success.final_score
            .replace("{}", &game_state.calculate_score(&data.constants).to_string());
        draw_text(
            &score_text,
            center_x - 150.0,
            y + 100.0,
            24.0,
            colors::ACCENT_DANGER,
        );

        // Continue Button
        if button(
             center_x - 100.0,
             screen_height() - 100.0,
             200.0,
             40.0,
             &data.localization.ui.common.continue_text
        ) {
            return UiAction::ReturnToMenu;
        }
    }
    
    UiAction::None
}

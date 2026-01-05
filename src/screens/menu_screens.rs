//! Menu screens: Main Menu, Loading, Briefing.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::GameData;
use crate::state::{GameState, PlayerStats};
use crate::ui::{UiAction, colors};

/// Draw the loading screen
pub fn draw_loading() -> UiAction {
    let text = "Loading...";
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
pub fn draw_main_menu(player_stats: &PlayerStats) -> UiAction {
    // Title
    let title = "🚕 NIGHTMARE SHIFT";
    let font_size = 48.0;
    let text_width = measure_text(title, None, font_size as u16, 1.0).width;
    draw_text(
        title,
        screen_width() / 2.0 - text_width / 2.0,
        150.0,
        font_size,
        colors::ACCENT_DANGER,
    );

    // Subtitle
    let subtitle = "Survive the night. Follow the rules. Maybe.";
    let sub_size = 20.0;
    let sub_width = measure_text(subtitle, None, sub_size as u16, 1.0).width;
    draw_text(
        subtitle,
        screen_width() / 2.0 - sub_width / 2.0,
        190.0,
        sub_size,
        colors::TEXT_SECONDARY,
    );

    // Stats
    let stats_y = 250.0;
    let stats = format!(
        "Shifts Completed: {} | Total Earnings: ${} | Rides: {}",
        player_stats.total_shifts_completed,
        player_stats.total_earnings,
        player_stats.total_rides_completed,
    );
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
    let achievements_text = format!("🏆 Achievements: {}/{}", unlocked_count, total_count);
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
        "Start Shift (SPACE)"
    ) {
        return UiAction::StartGame;
    }

    // Meta-progression buttons
    let button_y = screen_height() / 2.0 + 20.0;
    let button_spacing = 60.0;

    // Skill Tree button
    if button(
        screen_width() / 2.0 - 150.0,
        button_y,
        300.0,
        50.0,
        &format!("🌳 Skill Tree (Bank: ${})", player_stats.bank_balance)
    ) {
        return UiAction::OpenSkillTree;
    }

    // Almanac button
    if button(
        screen_width() / 2.0 - 150.0,
        button_y + button_spacing,
        300.0,
        50.0,
        &format!("📖 Almanac (Lore: {})", player_stats.lore_fragments)
    ) {
        return UiAction::OpenAlmanac;
    }

    // Leaderboard button
    if button(
        screen_width() / 2.0 - 150.0,
        button_y + button_spacing * 2.0,
        300.0,
        50.0,
        "🏆 Leaderboard"
    ) {
        return UiAction::OpenLeaderboard;
    }

    UiAction::None
}

/// Draw the briefing screen
pub fn draw_briefing(game_state: &GameState) -> UiAction {
    // Title
    let title = "📋 SHIFT BRIEFING";
    let font_size = 36.0;
    draw_text(title, 50.0, 60.0, font_size, colors::ACCENT_WARNING);

    // Rules
    draw_text("Tonight's Rules:", 50.0, 120.0, 24.0, WHITE);

    let mut y = 160.0;
    for (i, rule) in game_state.current_rules.iter().enumerate() {
        let rule_text = format!("{}. {} - {}", i + 1, rule.title, rule.description);
        draw_text(&rule_text, 70.0, y, 18.0, colors::TEXT_SECONDARY);
        y += 30.0;
    }

    // Weather
    y += 20.0;
    let weather_text = format!(
        "Weather: {} {} - {}",
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
        "Begin Shift (SPACE)"
    ) {
        return UiAction::StartGame;
    }
    
    UiAction::None
}

/// Draw the game over screen
pub fn draw_game_over(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    // Title
    let title = "💀 GAME OVER";
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
    let score = if let Some(data) = game_data {
        game_state.calculate_score(&data.constants)
    } else {
        0
    };

    let stats = format!(
        "Earnings: ${} | Rides: {} | Score: {}",
        game_state.earnings,
        game_state.rides_completed,
        score,
    );
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
        "Try Again (SPACE)"
    ) {
        return UiAction::TryAgain;
    }
    
    UiAction::None
}

/// Draw the success screen
pub fn draw_success(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    // Title
    let title = "🌅 SHIFT COMPLETE!";
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
    let subtitle = "You survived the night!";
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
    draw_text(
        &format!("Total Earnings: ${}", game_state.earnings),
        center_x - 150.0,
        y,
        20.0,
        colors::ACCENT_GOLD,
    );
    draw_text(
        &format!("Rides Completed: {}", game_state.rides_completed),
        center_x - 150.0,
        y + 30.0,
        20.0,
        WHITE,
    );

    if let Some(data) = game_data {
        draw_text(
            &format!("Survival Bonus: +${}", data.constants.scoring.survival_bonus),
            center_x - 150.0,
            y + 60.0,
            20.0,
            colors::FUEL_GOOD,
        );
        draw_text(
            &format!("Final Score: {}", game_state.calculate_score(&data.constants)),
            center_x - 150.0,
            y + 100.0,
            24.0,
            colors::ACCENT_DANGER,
        );
    }

    // Continue Button
    if button(
         center_x - 100.0,
         screen_height() - 100.0,
         200.0,
         40.0,
         "Continue"
    ) {
        return UiAction::ReturnToMenu;
    }
    
    UiAction::None
}

//! The main menu: driver record panel and the primary command list.

use macroquad::prelude::*;

use crate::data::GameData;
use crate::state::{Persistence, PlayerStats};
use crate::ui::{
    colors, draw_glass_panel, draw_noir_city_background, draw_small_caps, draw_wrapped_text, fonts,
    UiAction, UiRect,
};
use macroquad_toolkit::ui::draw_ui_text;

use super::widgets::draw_menu_command;

/// Draw the main menu
pub fn draw_main_menu(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();

    // Default strings if data missing (shouldn't happen)
    let title_text = if let Some(d) = game_data {
        &d.localization.ui.main_menu.title
    } else {
        "NIGHTMARE SHIFT"
    };
    let subtitle_text = if let Some(d) = game_data {
        &d.localization.ui.main_menu.subtitle
    } else {
        "Survive the night."
    };

    let menu_scale = (screen_width() / 1920.0)
        .min(screen_height() / 1080.0)
        .clamp(0.45, 1.0);
    let title_x = (70.0 * menu_scale).clamp(30.0, 70.0);
    let title_size = (72.0 * menu_scale).clamp(32.0, 72.0);
    let title_gap = title_size * 0.92;
    let mut title_y = (112.0 * menu_scale).clamp(50.0, 122.0);
    for line in title_text.split_whitespace() {
        draw_ui_text(line, title_x, title_y, title_size, colors::TEXT_PRIMARY);
        title_y += title_gap;
    }
    draw_small_caps(
        subtitle_text,
        title_x,
        title_y + 12.0,
        fonts::SIZE_MD,
        colors::CAB_YELLOW,
    );

    if let Some(data) = game_data {
        let stats = data
            .localization
            .ui
            .main_menu
            .stats
            .replacen("{}", &player_stats.total_shifts_completed.to_string(), 1)
            .replacen("{}", &player_stats.total_earnings.to_string(), 1)
            .replacen("{}", &player_stats.total_rides_completed.to_string(), 1);

        let progression = format!(
            "Experience Lv. {} | Suggested Difficulty {}",
            player_stats.experience_level(),
            player_stats.suggested_difficulty() + 1
        );
        let unlocked_count = player_stats
            .achievements
            .iter()
            .filter(|a| a.unlocked)
            .count();
        let total_count = player_stats.achievements.len();
        let achievements_text = data
            .localization
            .ui
            .main_menu
            .achievements
            .replacen("{}", &unlocked_count.to_string(), 1)
            .replacen("{}", &total_count.to_string(), 1);

        if screen_width() >= 980.0 && screen_height() >= 560.0 {
            let stats_rect = UiRect::new((screen_width() - 306.0).max(470.0), 88.0, 260.0, 132.0);
            draw_glass_panel(stats_rect, colors::BORDER_DIM);
            let stats_inner = stats_rect.inset(14.0);
            draw_small_caps(
                "Driver Record",
                stats_inner.x,
                stats_inner.y + 12.0,
                fonts::SIZE_SM,
                colors::CAB_YELLOW,
            );
            draw_wrapped_text(
                &stats,
                stats_inner.x,
                stats_inner.y + 38.0,
                stats_inner.w,
                fonts::SIZE_XS,
                16.0,
                colors::TEXT_SECONDARY,
                2,
            );
            draw_wrapped_text(
                &progression,
                stats_inner.x,
                stats_inner.y + 74.0,
                stats_inner.w,
                fonts::SIZE_XS,
                16.0,
                colors::TEXT_MUTED,
                2,
            );
            draw_ui_text(
                &achievements_text,
                stats_inner.x,
                stats_inner.y + 112.0,
                fonts::SIZE_XS,
                colors::ACCENT_GOLD,
            );
        }

        let menu_w = (screen_width() * 0.36).clamp(230.0, 380.0);
        let menu_h = (62.0 * menu_scale).clamp(38.0, 62.0);
        let gap = (12.0 * menu_scale).clamp(6.0, 12.0);
        let menu_items = if Persistence::save_exists() { 5.0 } else { 4.0 };
        let total_menu_h = menu_h * menu_items + gap * (menu_items - 1.0);
        let right_margin = 46.0 * menu_scale;
        let taxi_right = screen_width() * 0.36;
        let taxi_clear_x = taxi_right + (28.0 * menu_scale).clamp(14.0, 28.0);
        let max_menu_x = screen_width() - menu_w - right_margin;
        let base_menu_x = (screen_width() * 0.16)
            .max(title_x + 185.0 * menu_scale)
            .min(max_menu_x);
        let menu_x = if taxi_clear_x <= max_menu_x {
            base_menu_x.max(taxi_clear_x)
        } else {
            base_menu_x
        };
        let min_menu_y = title_y + 86.0 * menu_scale;
        let max_menu_y = (screen_height() - total_menu_h - 54.0 * menu_scale).max(min_menu_y);
        let menu_y = (screen_height() * 0.38).max(min_menu_y).min(max_menu_y);

        let start_label = data
            .localization
            .ui
            .main_menu
            .start_space
            .replace("(SPACE)", "");
        if draw_menu_command(
            UiRect::new(menu_x, menu_y, menu_w, menu_h),
            "wheel",
            start_label.trim(),
            "SPACE",
            colors::CAB_YELLOW,
            menu_scale,
        ) {
            return UiAction::StartGame;
        }

        let skill_btn_text = data
            .localization
            .ui
            .meta
            .skill_tree
            .button
            .replace("{}", &player_stats.bank_balance.to_string());
        let skill_detail = skill_btn_text
            .split_once('(')
            .map(|(_, detail)| detail.trim_end_matches(')').to_string())
            .unwrap_or_else(|| format!("${} Available", player_stats.bank_balance));
        if draw_menu_command(
            UiRect::new(menu_x, menu_y + (menu_h + gap), menu_w, menu_h),
            "tree",
            "Skill Tree",
            &skill_detail,
            colors::TEXT_SECONDARY,
            menu_scale,
        ) {
            return UiAction::OpenSkillTree;
        }

        let almanac_btn_text = data
            .localization
            .ui
            .meta
            .almanac
            .button
            .replace("{}", &player_stats.lore_fragments.to_string());
        let almanac_detail = almanac_btn_text
            .split_once('(')
            .map(|(_, detail)| detail.trim_end_matches(')').to_string())
            .unwrap_or_else(|| format!("{} Lore Fragments", player_stats.lore_fragments));
        if draw_menu_command(
            UiRect::new(menu_x, menu_y + (menu_h + gap) * 2.0, menu_w, menu_h),
            "book",
            "Almanac",
            &almanac_detail,
            colors::TEXT_SECONDARY,
            menu_scale,
        ) {
            return UiAction::OpenAlmanac;
        }

        let leaderboard_btn_text = data
            .localization
            .ui
            .meta
            .leaderboard
            .button
            .chars()
            .filter(|ch| ch.is_ascii())
            .collect::<String>()
            .trim()
            .to_string();
        if draw_menu_command(
            UiRect::new(menu_x, menu_y + (menu_h + gap) * 3.0, menu_w, menu_h),
            "trophy",
            &leaderboard_btn_text,
            "Best Runs",
            colors::TEXT_SECONDARY,
            menu_scale,
        ) {
            return UiAction::OpenLeaderboard;
        }

        if Persistence::save_exists()
            && draw_menu_command(
                UiRect::new(menu_x, menu_y + (menu_h + gap) * 4.0, menu_w, menu_h),
                "delete",
                "Delete Save",
                "Reset Progress",
                colors::ACCENT_DANGER,
                menu_scale,
            )
        {
            return UiAction::DeleteSave;
        }
    }

    UiAction::None
}

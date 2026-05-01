//! Menu screens: Main Menu, Loading, Briefing.

use macroquad::prelude::*;

use crate::data::GameData;
use crate::state::{GameState, Persistence, PlayerStats};
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_small_caps,
    draw_wrapped_text, fonts, spacing, UiAction, UiRect,
};

fn draw_menu_icon(kind: &str, cx: f32, cy: f32, scale: f32, color: Color) {
    let s = scale;
    let stroke = (2.0 * s).max(1.0);
    match kind {
        "wheel" => {
            draw_circle_lines(cx, cy, 14.0 * s, stroke, color);
            draw_circle(cx, cy, 3.0 * s, color);
            draw_line(cx, cy, cx, cy - 12.0 * s, stroke, color);
            draw_line(cx, cy, cx - 11.0 * s, cy + 7.0 * s, stroke, color);
            draw_line(cx, cy, cx + 11.0 * s, cy + 7.0 * s, stroke, color);
        }
        "tree" => {
            draw_rectangle(cx - 3.0 * s, cy + 8.0 * s, 6.0 * s, 11.0 * s, color);
            draw_triangle(
                Vec2::new(cx, cy - 18.0 * s),
                Vec2::new(cx - 16.0 * s, cy + 4.0 * s),
                Vec2::new(cx + 16.0 * s, cy + 4.0 * s),
                Color::new(color.r, color.g, color.b, 0.58),
            );
            draw_triangle(
                Vec2::new(cx, cy - 7.0 * s),
                Vec2::new(cx - 18.0 * s, cy + 14.0 * s),
                Vec2::new(cx + 18.0 * s, cy + 14.0 * s),
                Color::new(color.r, color.g, color.b, 0.68),
            );
        }
        "book" => {
            draw_rectangle_lines(
                cx - 18.0 * s,
                cy - 14.0 * s,
                16.0 * s,
                28.0 * s,
                stroke,
                color,
            );
            draw_rectangle_lines(
                cx + 2.0 * s,
                cy - 14.0 * s,
                16.0 * s,
                28.0 * s,
                stroke,
                color,
            );
            draw_line(cx, cy - 13.0 * s, cx, cy + 15.0 * s, stroke, color);
        }
        "trophy" => {
            draw_rectangle_lines(
                cx - 10.0 * s,
                cy - 15.0 * s,
                20.0 * s,
                18.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx - 16.0 * s,
                cy - 11.0 * s,
                cx - 10.0 * s,
                cy - 6.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx + 16.0 * s,
                cy - 11.0 * s,
                cx + 10.0 * s,
                cy - 6.0 * s,
                stroke,
                color,
            );
            draw_line(cx, cy + 3.0 * s, cx, cy + 14.0 * s, stroke, color);
            draw_rectangle_lines(
                cx - 12.0 * s,
                cy + 14.0 * s,
                24.0 * s,
                6.0 * s,
                stroke,
                color,
            );
        }
        "delete" => {
            draw_rectangle_lines(
                cx - 12.0 * s,
                cy - 10.0 * s,
                24.0 * s,
                26.0 * s,
                stroke,
                color,
            );
            draw_rectangle(cx - 15.0 * s, cy - 16.0 * s, 30.0 * s, 4.0 * s, color);
            draw_line(
                cx - 5.0 * s,
                cy - 6.0 * s,
                cx - 5.0 * s,
                cy + 11.0 * s,
                stroke,
                color,
            );
            draw_line(
                cx + 5.0 * s,
                cy - 6.0 * s,
                cx + 5.0 * s,
                cy + 11.0 * s,
                stroke,
                color,
            );
        }
        _ => {
            draw_circle_lines(cx, cy, 13.0 * s, stroke, color);
        }
    }
}

fn draw_briefing_taxi_scene(rect: UiRect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.010, 0.012, 0.012, 0.92),
    );
    draw_rectangle(rect.x, rect.y, rect.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.10));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);

    let road_y = rect.y + rect.h * 0.68;
    draw_rectangle(
        rect.x,
        road_y,
        rect.w,
        rect.h * 0.32,
        Color::new(0.020, 0.024, 0.022, 1.0),
    );
    for i in 0..8 {
        let x = rect.x + rect.w * (i as f32 / 8.0);
        draw_rectangle(
            x + rect.w * 0.03,
            road_y + rect.h * 0.14,
            rect.w * 0.05,
            3.0,
            Color::new(0.95, 0.58, 0.08, 0.32),
        );
    }

    for i in 0..6 {
        let x = rect.x + rect.w * (0.12 + i as f32 * 0.15);
        let lamp_top = rect.y + rect.h * (0.16 + (i % 2) as f32 * 0.05);
        draw_line(
            x,
            lamp_top + 26.0,
            x,
            road_y + 26.0,
            2.0,
            Color::new(0.28, 0.24, 0.19, 0.70),
        );
        draw_circle(x, lamp_top, 5.0, colors::CAB_YELLOW);
        draw_circle(x, lamp_top, 24.0, Color::new(0.95, 0.58, 0.08, 0.08));
    }

    let tx = rect.x + rect.w * 0.50;
    let ty = rect.y + rect.h * 0.47;
    let scale = (rect.w / 820.0).clamp(0.72, 1.18);
    let body_w = 330.0 * scale;
    let body_h = 72.0 * scale;
    let body_x = tx - body_w / 2.0;
    let body_y = ty;
    draw_rectangle(
        body_x,
        body_y,
        body_w,
        body_h,
        Color::new(0.74, 0.45, 0.06, 0.95),
    );
    draw_rectangle(
        body_x + body_w * 0.14,
        body_y - body_h * 0.44,
        body_w * 0.46,
        body_h * 0.44,
        Color::new(0.55, 0.34, 0.06, 0.92),
    );
    draw_rectangle(
        body_x + body_w * 0.20,
        body_y - body_h * 0.31,
        body_w * 0.15,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.86),
    );
    draw_rectangle(
        body_x + body_w * 0.39,
        body_y - body_h * 0.31,
        body_w * 0.17,
        body_h * 0.25,
        Color::new(0.06, 0.09, 0.10, 0.86),
    );
    draw_rectangle(
        body_x + body_w * 0.26,
        body_y - body_h * 0.66,
        body_w * 0.22,
        body_h * 0.18,
        colors::CAB_YELLOW,
    );
    draw_text(
        "TAXI",
        body_x + body_w * 0.31,
        body_y - body_h * 0.51,
        fonts::SIZE_SM * scale,
        colors::BLACK,
    );
    draw_rectangle(
        body_x + body_w * 0.03,
        body_y + body_h * 0.46,
        body_w * 0.14,
        body_h * 0.14,
        Color::new(0.95, 0.10, 0.04, 0.58),
    );
    draw_rectangle(
        body_x + body_w * 0.84,
        body_y + body_h * 0.45,
        body_w * 0.11,
        body_h * 0.12,
        Color::new(1.0, 0.86, 0.44, 0.68),
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

fn draw_menu_command(
    rect: UiRect,
    icon: &str,
    label: &str,
    detail: &str,
    accent: Color,
    scale: f32,
) -> bool {
    let clicked = draw_glass_button(rect, "", accent, true);
    let icon_col_w = (60.0 * scale).clamp(42.0, 60.0);
    let label_size = (fonts::SIZE_LG * scale).clamp(13.0, fonts::SIZE_LG);
    let detail_size = (fonts::SIZE_XS * scale).clamp(8.0, fonts::SIZE_XS);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        icon_col_w,
        rect.h - 2.0,
        Color::new(0.0, 0.0, 0.0, 0.16),
    );
    draw_menu_icon(
        icon,
        rect.x + icon_col_w / 2.0,
        rect.y + rect.h / 2.0,
        scale,
        colors::TEXT_MUTED,
    );
    let text_x = rect.x + icon_col_w + 16.0 * scale;
    draw_small_caps(
        label,
        text_x,
        rect.y + rect.h * 0.46,
        label_size,
        colors::TEXT_PRIMARY,
    );
    draw_small_caps(
        detail,
        text_x,
        rect.y + rect.h * 0.78,
        detail_size,
        colors::TEXT_MUTED,
    );
    clicked
}

/// Draw the loading screen
pub fn draw_loading(game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let text = if let Some(data) = game_data {
        &data.localization.ui.common.loading
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
        colors::TEXT_PRIMARY,
    );
    if let Some(data) = game_data {
        let system_width = measure_text(&data.localization.system.loading, None, 14, 1.0).width;
        draw_text(
            &data.localization.system.loading,
            screen_width() / 2.0 - system_width / 2.0,
            screen_height() / 2.0 + 28.0,
            14.0,
            colors::TEXT_SECONDARY,
        );
        let meta_text = format!(
            "{} {} v{}",
            data.localization.meta.language,
            data.localization.meta.code,
            data.localization.meta.version
        );
        let meta_width = measure_text(&meta_text, None, 14, 1.0).width;
        draw_text(
            &meta_text,
            screen_width() / 2.0 - meta_width / 2.0,
            screen_height() / 2.0 + 48.0,
            14.0,
            colors::TEXT_MUTED,
        );
    }
    UiAction::None
}

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
        draw_text(line, title_x, title_y, title_size, colors::TEXT_PRIMARY);
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
            draw_text(
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

        if Persistence::save_exists() {
            if draw_menu_command(
                UiRect::new(menu_x, menu_y + (menu_h + gap) * 4.0, menu_w, menu_h),
                "delete",
                "Delete Save",
                "Reset Progress",
                colors::ACCENT_DANGER,
                menu_scale,
            ) {
                return UiAction::DeleteSave;
            }
        }
    }

    UiAction::None
}

/// Draw the briefing screen
pub fn draw_briefing(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();

    if let Some(data) = game_data {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let margin = (screen_w * 0.045).clamp(30.0, 70.0);
        let header_h = 100.0;
        let footer_h = 76.0;
        let content_top = header_h + 30.0;
        let content_bottom = screen_h - footer_h;
        let gap = 18.0;

        let header = UiRect::new(margin, 28.0, screen_w - margin * 2.0, header_h);
        draw_glass_panel(header, colors::BORDER_DIM);
        let header_inner = header.inset(18.0);

        draw_text(
            &data.localization.ui.briefing.title,
            header_inner.x,
            header_inner.y + 40.0,
            44.0,
            colors::CAB_YELLOW,
        );
        draw_small_caps(
            "Dispatch briefing for tonight's shift",
            header_inner.x,
            header_inner.y + 70.0,
            fonts::SIZE_MD,
            colors::TEXT_MUTED,
        );
        draw_small_caps(
            "Review active rules before accepting passengers.",
            header_inner.x + header_inner.w * 0.58,
            header_inner.y + 58.0,
            fonts::SIZE_SM,
            colors::TEXT_SECONDARY,
        );

        let top_h = ((content_bottom - content_top) * 0.58).clamp(360.0, 520.0);
        let scene_h = content_bottom - content_top - top_h - gap;
        let left_w = ((screen_w - margin * 2.0 - gap) * 0.62).clamp(560.0, 1050.0);
        let right_w = screen_w - margin * 2.0 - gap - left_w;
        let rules_panel = UiRect::new(margin, content_top, left_w, top_h);
        let conditions_panel = UiRect::new(margin + left_w + gap, content_top, right_w, top_h);
        let scene_panel = UiRect::new(
            margin,
            content_top + top_h + gap,
            screen_w - margin * 2.0,
            scene_h,
        );

        draw_glass_panel(rules_panel, colors::BORDER_DIM);
        draw_glass_panel(conditions_panel, colors::BORDER_DIM);

        let rules_inner = rules_panel.inset(18.0);
        draw_small_caps(
            &data.localization.ui.briefing.rules_title,
            rules_inner.x,
            rules_inner.y + 16.0,
            fonts::SIZE_LG,
            colors::CAB_YELLOW,
        );

        let mut y = rules_inner.y + 54.0;
        for (i, rule) in game_state.current_rules.iter().enumerate() {
            let card_h = 76.0;
            let card = UiRect::new(rules_inner.x, y, rules_inner.w, card_h);
            draw_rectangle(
                card.x,
                card.y,
                card.w,
                card.h,
                Color::new(0.025, 0.030, 0.032, 0.92),
            );
            draw_rectangle(card.x, card.y, 4.0, card.h, colors::CAB_YELLOW);
            draw_rectangle_lines(card.x, card.y, card.w, card.h, 1.0, colors::BORDER_DIM);
            draw_small_caps(
                &format!("Rule {}", i + 1),
                card.x + 18.0,
                card.y + 24.0,
                fonts::SIZE_SM,
                colors::CAB_YELLOW,
            );
            draw_text(
                &rule.title,
                card.x + 96.0,
                card.y + 27.0,
                fonts::SIZE_LG,
                colors::TEXT_PRIMARY,
            );
            draw_wrapped_text(
                &rule.description,
                card.x + 96.0,
                card.y + 50.0,
                card.w - 116.0,
                fonts::SIZE_XS,
                15.0,
                colors::TEXT_SECONDARY,
                2,
            );
            y += card_h + 12.0;
            if y > rules_panel.bottom() - 22.0 {
                break;
            }
        }
        let target_y = (rules_panel.bottom() - 112.0).max(y + 10.0);
        if target_y + 84.0 <= rules_panel.bottom() - 16.0 {
            draw_small_caps(
                "Shift Targets",
                rules_inner.x,
                target_y - 14.0,
                fonts::SIZE_SM,
                colors::TEXT_MUTED,
            );
            let stat_gap = 12.0;
            let stat_w = (rules_inner.w - stat_gap * 2.0) / 3.0;
            let stats = [
                (
                    "Fuel",
                    format!("{:.0}%", game_state.fuel),
                    colors::FUEL_GOOD,
                ),
                (
                    "Minimum",
                    format!("${}", game_state.minimum_earnings),
                    colors::ACCENT_GOLD,
                ),
                (
                    "Difficulty",
                    format!("{}", game_state.difficulty_level + 1),
                    colors::ACCENT_SKY,
                ),
            ];
            for (idx, (label, value, color)) in stats.iter().enumerate() {
                let rect = UiRect::new(
                    rules_inner.x + idx as f32 * (stat_w + stat_gap),
                    target_y,
                    stat_w,
                    74.0,
                );
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.025, 0.030, 0.032, 0.92),
                );
                draw_rectangle(rect.x, rect.y, 4.0, rect.h, *color);
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);
                draw_text(value, rect.x + 18.0, rect.y + 31.0, fonts::SIZE_XL, *color);
                draw_small_caps(
                    label,
                    rect.x + 18.0,
                    rect.y + 55.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
            }
        }

        let conditions_inner = conditions_panel.inset(18.0);
        draw_small_caps(
            "Night Conditions",
            conditions_inner.x,
            conditions_inner.y + 16.0,
            fonts::SIZE_LG,
            colors::CAB_YELLOW,
        );

        let mut side_y = conditions_inner.y + 58.0;
        let weather_label = &data.localization.ui.briefing.weather_title;
        let weather_text = format!(
            "{} {} - {}",
            weather_label,
            game_state.current_weather.weather_type.name(),
            game_state.current_weather.description
        );
        draw_rectangle(
            conditions_inner.x,
            side_y,
            conditions_inner.w,
            108.0,
            Color::new(0.020, 0.040, 0.050, 0.92),
        );
        draw_rectangle(conditions_inner.x, side_y, 4.0, 108.0, colors::ACCENT_SKY);
        draw_rectangle_lines(
            conditions_inner.x,
            side_y,
            conditions_inner.w,
            108.0,
            1.0,
            colors::BORDER_DIM,
        );
        draw_text(
            game_state.current_weather.weather_type.name(),
            conditions_inner.x + 18.0,
            side_y + 34.0,
            fonts::SIZE_XL,
            colors::ACCENT_SKY,
        );
        side_y = draw_wrapped_text(
            &weather_text,
            conditions_inner.x + 18.0,
            side_y + 62.0,
            conditions_inner.w - 36.0,
            fonts::SIZE_SM,
            18.0,
            colors::TEXT_SECONDARY,
            3,
        ) + 18.0;

        let weather_rule_ids = crate::engine::WeatherService::get_weather_triggered_rules(
            &game_state.current_weather,
            &game_state.time_of_day,
        );
        if !weather_rule_ids.is_empty() {
            let advisory = format!("Weather advisories active: {}", weather_rule_ids.len());
            side_y = draw_wrapped_text(
                &advisory,
                conditions_inner.x,
                side_y,
                conditions_inner.w,
                fonts::SIZE_SM,
                18.0,
                colors::ACCENT_WARNING,
                2,
            ) + 14.0;
        }

        if !game_state.environmental_hazards.is_empty() {
            draw_small_caps(
                "Active Hazards",
                conditions_inner.x,
                side_y,
                fonts::SIZE_LG,
                colors::ACCENT_WARNING,
            );
            side_y += 30.0;
            for hazard in game_state.environmental_hazards.iter().take(3) {
                let text = format!("{} - {}", hazard.location, hazard.description);
                side_y = draw_wrapped_text(
                    &text,
                    conditions_inner.x,
                    side_y,
                    conditions_inner.w,
                    fonts::SIZE_SM,
                    18.0,
                    colors::TEXT_SECONDARY,
                    2,
                ) + 10.0;
            }
        } else {
            draw_small_caps(
                "No active environmental hazards reported.",
                conditions_inner.x,
                side_y,
                fonts::SIZE_SM,
                colors::TEXT_MUTED,
            );
        }
        let condition_stats_y = (conditions_panel.bottom() - 112.0).max(side_y + 12.0);
        if condition_stats_y + 82.0 <= conditions_panel.bottom() - 16.0 {
            draw_small_caps(
                "Readings",
                conditions_inner.x,
                condition_stats_y - 14.0,
                fonts::SIZE_SM,
                colors::TEXT_MUTED,
            );
            let stat_gap = 10.0;
            let stat_w = (conditions_inner.w - stat_gap * 2.0) / 3.0;
            let readings = [
                (
                    "Visibility",
                    format!("{}%", game_state.current_weather.visibility),
                    colors::ACCENT_SKY,
                ),
                (
                    "Activity",
                    format!("{}%", game_state.time_of_day.supernatural_activity),
                    colors::ACCENT_WARNING,
                ),
                (
                    "Hour",
                    format!("{:02}:00", game_state.time_of_day.hour),
                    colors::TEXT_SECONDARY,
                ),
            ];
            for (idx, (label, value, color)) in readings.iter().enumerate() {
                let rect = UiRect::new(
                    conditions_inner.x + idx as f32 * (stat_w + stat_gap),
                    condition_stats_y,
                    stat_w,
                    74.0,
                );
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.025, 0.030, 0.032, 0.92),
                );
                draw_rectangle(rect.x, rect.y, 4.0, rect.h, *color);
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);
                draw_text(value, rect.x + 14.0, rect.y + 31.0, fonts::SIZE_LG, *color);
                draw_small_caps(
                    label,
                    rect.x + 14.0,
                    rect.y + 55.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
            }
        }

        draw_briefing_taxi_scene(scene_panel);

        let btn_rect = UiRect::new(screen_w / 2.0 - 170.0, screen_h - 62.0, 340.0, 48.0);
        if draw_glass_button(
            btn_rect,
            &data.localization.ui.briefing.begin_space,
            colors::CAB_YELLOW,
            true,
        ) {
            return UiAction::StartGame;
        }
    }

    UiAction::None
}

/// Draw the game over screen
pub fn draw_game_over(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        let panel = UiRect::centered_x(112.0, screen_width().min(540.0), 330.0);
        draw_glass_panel(panel, colors::ACCENT_DANGER);
        let inner = panel.inset(spacing::PADDING_LG);

        let title = &data.localization.ui.game_over.title;
        let title_size = 46.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            inner.y + 50.0,
            title_size,
            colors::FUEL_CRITICAL,
        );

        if let Some(ref reason) = game_state.game_over_reason {
            draw_wrapped_text(
                reason,
                inner.x,
                inner.y + 92.0,
                inner.w,
                fonts::SIZE_MD,
                22.0,
                colors::TEXT_SECONDARY,
                3,
            );
        }

        let score = game_state.calculate_score(&data.constants);
        let stats = data
            .localization
            .ui
            .game_over
            .stats
            .replacen("{}", &game_state.earnings.to_string(), 1)
            .replacen("{}", &game_state.rides_completed.to_string(), 1)
            .replacen("{}", &score.to_string(), 1);

        draw_text(
            &stats,
            inner.x,
            inner.y + 186.0,
            fonts::SIZE_MD,
            colors::TEXT_PRIMARY,
        );

        if draw_glass_button(
            UiRect::new(center_x - 150.0, panel.bottom() - 70.0, 300.0, 46.0),
            &data.localization.ui.common.try_again,
            colors::ACCENT_DANGER,
            true,
        ) {
            return UiAction::TryAgain;
        }
    }

    UiAction::None
}

/// Draw the success screen
pub fn draw_success(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        let panel = UiRect::centered_x(104.0, screen_width().min(540.0), 350.0);
        draw_glass_panel(panel, colors::ACCENT_GOLD);
        let inner = panel.inset(spacing::PADDING_LG);

        let title = &data.localization.ui.success.title;
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            inner.y + 50.0,
            title_size,
            colors::ACCENT_GOLD,
        );

        let subtitle = &data.localization.ui.success.subtitle;
        let sub_width = measure_text(subtitle, None, 24, 1.0).width;
        draw_text(
            subtitle,
            center_x - sub_width / 2.0,
            inner.y + 92.0,
            24.0,
            colors::ACCENT_PRIMARY,
        );

        let y = inner.y + 142.0;

        let earnings_text = data
            .localization
            .ui
            .success
            .total_earnings
            .replace("{}", &game_state.earnings.to_string());
        draw_text(&earnings_text, inner.x, y, 20.0, colors::ACCENT_GOLD);

        let rides_text = data
            .localization
            .ui
            .success
            .rides_completed
            .replace("{}", &game_state.rides_completed.to_string());
        draw_text(&rides_text, inner.x, y + 30.0, 20.0, WHITE);

        let bonus_text = data.localization.ui.success.survival_bonus.replace(
            "{}",
            &data.constants.game_constants.survival_bonus.to_string(),
        );
        draw_text(&bonus_text, inner.x, y + 60.0, 20.0, colors::FUEL_GOOD);

        let score_text = data.localization.ui.success.final_score.replace(
            "{}",
            &game_state.calculate_score(&data.constants).to_string(),
        );
        draw_text(&score_text, inner.x, y + 100.0, 24.0, colors::ACCENT_DANGER);

        if draw_glass_button(
            UiRect::new(center_x - 100.0, panel.bottom() - 62.0, 200.0, 42.0),
            &data.localization.ui.common.continue_text,
            colors::ACCENT_GOLD,
            true,
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

//! Meta-progression screens: Skill Tree, Almanac, and Leaderboard.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::{UiAction, colors};

/// Draw the skill tree screen
pub fn draw_skill_tree(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.meta.skill_tree.title;
        let title_size = 36.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            colors::ACCENT_PRIMARY,
        );

        // Bank balance
        // "Bank Balance: ${}"
        let balance = data.localization.ui.meta.skill_tree.bank_balance
            .replace("{}", &player_stats.bank_balance.to_string());
            
        let balance_width = measure_text(&balance, None, 20, 1.0).width;
        draw_text(
            &balance,
            center_x - balance_width / 2.0,
            100.0,
            20.0,
            colors::ACCENT_GOLD,
        );

        let mut y = 150.0;
        let categories = vec!["survival", "occult", "efficiency"];
        // Localization categories
        let loc_cats = &data.localization.ui.meta.skill_tree.categories;
        let category_names = vec![
            &loc_cats.survival,
            &loc_cats.occult,
            &loc_cats.efficiency,
        ];

        for (cat_idx, category) in categories.iter().enumerate() {
            // Category header
            draw_text(
                category_names[cat_idx],
                50.0,
                y,
                24.0,
                colors::ACCENT_WARNING,
            );
            y += 35.0;

            // Skills in category
            for skill in data.skills.iter().filter(|s| &s.category == category) {
                let is_unlocked = player_stats.is_skill_unlocked(&skill.id);
                let can_unlock = skill.can_unlock(&player_stats.unlocked_skills)
                    && !is_unlocked
                    && player_stats.bank_balance >= skill.cost;

                let color = if is_unlocked {
                    colors::FUEL_GOOD // Green - unlocked
                } else if can_unlock {
                    colors::ACCENT_PRIMARY // Cyan - can afford
                } else {
                    colors::TEXT_MUTED // Gray - locked
                };

                let status = if is_unlocked {
                    format!(" {}", data.localization.ui.meta.skill_tree.unlocked)
                } else {
                    String::new()
                };

                let text = format!("{} {} - ${}{}", skill.icon, skill.name, skill.cost, status);
                draw_text(&text, 70.0, y, 18.0, color);
                y += 25.0;

                // Description
                draw_text(&skill.description, 90.0, y, 14.0, colors::TEXT_SECONDARY);
                y += 25.0;

                // Purchase button if can unlock
                if can_unlock {
                    if button(90.0, y - 20.0, 150.0, 30.0, &data.localization.ui.meta.skill_tree.purchase) {
                        return UiAction::PurchaseSkill(skill.id.clone());
                    }
                }
            }

            y += 15.0;
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            &data.localization.ui.common.back_button
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

/// Draw the almanac screen
pub fn draw_almanac(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.meta.almanac.title;
        let title_size = 36.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            colors::ACCENT_PRIMARY,
        );

        // Lore fragments
        // "Lore Fragments: {}"
        let fragments = data.localization.ui.meta.almanac.fragments
            .replace("{}", &player_stats.lore_fragments.to_string());
            
        let fragments_width = measure_text(&fragments, None, 20, 1.0).width;
        draw_text(
            &fragments,
            center_x - fragments_width / 2.0,
            100.0,
            20.0,
            colors::ACCENT_GOLD,
        );

        let mut y = 150.0;

        for passenger in &data.passengers {
            let entry = player_stats.get_almanac_entry(passenger.id);
            // Ideally level names should be localized too, but they come from data.almanac structure which might contain strings from JSON
            // For now we use what we have, or fallback to localized "Unknown"
            let level_name = data.almanac.get_level(entry.knowledge_level)
                .map(|l| l.name.as_str())
                .unwrap_or(&data.localization.ui.meta.almanac.unknown_level);

            let color = if entry.encountered {
                colors::ACCENT_PRIMARY
            } else {
                Color::from_hex(0x555555)
            };

            let text = format!(
                "{} {} - Level {}: {}",
                passenger.emoji,
                passenger.name,
                entry.knowledge_level,
                level_name
            );
            draw_text(&text, 50.0, y, 18.0, color);
            y += 25.0;

            // Show upgrade button if encountered and not max level
            if entry.encountered && entry.knowledge_level < 3 {
                let cost = data.almanac.get_upgrade_cost(entry.knowledge_level + 1);
                let can_afford = player_stats.lore_fragments >= cost;

                // "Upgrade (Cost: {} fragments)"
                let button_text = data.localization.ui.meta.almanac.upgrade_cost
                    .replace("{}", &cost.to_string());

                let button_color = if can_afford {
                    colors::FUEL_GOOD
                } else {
                    colors::TEXT_MUTED
                };

                if can_afford && button(70.0, y - 20.0, 200.0, 30.0, &button_text) {
                    return UiAction::UpgradeAlmanacKnowledge(passenger.id);
                } else if !can_afford {
                    draw_text(&button_text, 70.0, y, 14.0, button_color);
                    y += 20.0;
                }
            }

            y += 10.0;

            // Scroll handling - simple pagination
            if y > screen_height() - 100.0 {
                break;
            }
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            &data.localization.ui.common.back_button
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

/// Draw the leaderboard and achievements screen
pub fn draw_leaderboard(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    // We need data for localization
    if let Some(data) = game_data {
        // Title
        let title = &data.localization.ui.meta.leaderboard.title;
        let title_size = 32.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            colors::ACCENT_PRIMARY,
        );

        // Leaderboard section
        let subtitle = &data.localization.ui.meta.leaderboard.top_runs;
        draw_text(
            subtitle,
            50.0,
            110.0,
            20.0,
            colors::ACCENT_WARNING,
        );

        let mut y = 140.0;

        if player_stats.leaderboard.is_empty() {
            let msg = &data.localization.ui.meta.leaderboard.no_runs;
            let msg_width = measure_text(msg, None, 18, 1.0).width;
            draw_text(
                msg,
                center_x - msg_width / 2.0,
                y + 50.0,
                18.0,
                colors::TEXT_MUTED,
            );
        } else {
            for (idx, entry) in player_stats.leaderboard.iter().enumerate() {
                let rank_color = match idx {
                    0 => colors::ACCENT_GOLD, // Gold
                    1 => WHITE, // Silver (Simplified)
                    2 => colors::ACCENT_WARNING, // Bronze (Simplified)
                    _ => WHITE,
                };

                let status_icon = if entry.survived { "✓" } else { "✗" };
                let status_color = if entry.survived {
                    colors::FUEL_GOOD
                } else {
                    colors::FUEL_CRITICAL
                };

                // Rank and score: "#{} Score: {}"
                let rank_text = data.localization.ui.meta.leaderboard.score_entry
                    .replacen("{}", &(idx + 1).to_string(), 1)
                    .replacen("{}", &entry.score.to_string(), 1);

                draw_text(&rank_text, 50.0, y, 20.0, rank_color);

                // Status
                draw_text(status_icon, 250.0, y, 20.0, status_color);

                y += 25.0;

                // Details: "  {} passengers | Difficulty {} | {} rule violations | {}"
                let details = data.localization.ui.meta.leaderboard.details
                    .replacen("{}", &entry.passengers_transported.to_string(), 1)
                    .replacen("{}", &entry.difficulty_level.to_string(), 1)
                    .replacen("{}", &entry.rules_violated.to_string(), 1)
                    .replacen("{}", &entry.date, 1);
                    
                draw_text(&details, 70.0, y, 14.0, colors::TEXT_SECONDARY);

                y += 30.0;
            }
        }

        // Achievements section
        let achievements_x = screen_width() / 2.0 + 50.0;
        let mut achievements_y = 110.0;

        draw_text(
            &data.localization.ui.meta.leaderboard.achievements,
            achievements_x,
            achievements_y,
            20.0,
            colors::ACCENT_WARNING,
        );
        achievements_y += 30.0;

        for achievement in &player_stats.achievements {
            let color = if achievement.unlocked {
                colors::FUEL_GOOD
            } else {
                colors::TEXT_MUTED
            };

            let status = if achievement.unlocked { "✓" } else { "✗" };
            let text = format!("{} {}", status, achievement.name);
            draw_text(&text, achievements_x, achievements_y, 16.0, color);
            achievements_y += 20.0;

            // Description
            draw_text(&achievement.description, achievements_x + 15.0, achievements_y, 12.0, colors::TEXT_MUTED);
            achievements_y += 25.0;
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            &data.localization.ui.common.back_button
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

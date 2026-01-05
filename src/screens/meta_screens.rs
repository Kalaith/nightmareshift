//! Meta-progression screens: Skill Tree, Almanac, and Leaderboard.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::{UiAction, colors};

/// Draw the skill tree screen
pub fn draw_skill_tree(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    // Title
    let title = "🌳 SKILL TREE";
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
    let balance = format!("Bank Balance: ${}", player_stats.bank_balance);
    let balance_width = measure_text(&balance, None, 20, 1.0).width;
    draw_text(
        &balance,
        center_x - balance_width / 2.0,
        100.0,
        20.0,
        colors::ACCENT_GOLD,
    );

    if let Some(data) = game_data {
        let mut y = 150.0;
        let categories = vec!["survival", "occult", "efficiency"];
        let category_names = vec!["🛡️ Survival", "👁️ Occult", "💰 Efficiency"];

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
                    "✓ UNLOCKED"
                } else {
                    ""
                };

                let text = format!("{} {} - ${} {}", skill.icon, skill.name, skill.cost, status);
                draw_text(&text, 70.0, y, 18.0, color);
                y += 25.0;

                // Description
                draw_text(&skill.description, 90.0, y, 14.0, colors::TEXT_SECONDARY);
                y += 25.0;

                // Purchase button if can unlock
                if can_unlock {
                    if button(90.0, y - 20.0, 150.0, 30.0, "Purchase") {
                        return UiAction::PurchaseSkill(skill.id.clone());
                    }
                }
            }

            y += 15.0;
        }
    }

    // Back button
    if button(
        center_x - 100.0,
        screen_height() - 60.0,
        200.0,
        40.0,
        "Back to Menu (ESC)"
    ) {
        return UiAction::ReturnToMenu;
    }

    UiAction::None
}

/// Draw the almanac screen
pub fn draw_almanac(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    // Title
    let title = "📖 ALMANAC";
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
    let fragments = format!("Lore Fragments: {}", player_stats.lore_fragments);
    let fragments_width = measure_text(&fragments, None, 20, 1.0).width;
    draw_text(
        &fragments,
        center_x - fragments_width / 2.0,
        100.0,
        20.0,
        colors::ACCENT_GOLD,
    );

    if let Some(data) = game_data {
        let mut y = 150.0;

        for passenger in &data.passengers {
            let entry = player_stats.get_almanac_entry(passenger.id);
            let level_name = data.almanac.get_level(entry.knowledge_level)
                .map(|l| l.name.as_str())
                .unwrap_or("Unknown");

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

                let button_text = format!("Upgrade (Cost: {} fragments)", cost);
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
    }

    // Back button
    if button(
        center_x - 100.0,
        screen_height() - 60.0,
        200.0,
        40.0,
        "Back to Menu (ESC)"
    ) {
        return UiAction::ReturnToMenu;
    }

    UiAction::None
}

/// Draw the leaderboard and achievements screen
pub fn draw_leaderboard(player_stats: &PlayerStats) -> UiAction {
    let center_x = screen_width() / 2.0;

    // Title
    let title = "🏆 LEADERBOARD & ACHIEVEMENTS";
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
    let subtitle = "Top 10 Best Runs";
    draw_text(
        subtitle,
        50.0,
        110.0,
        20.0,
        colors::ACCENT_WARNING,
    );

    let mut y = 140.0;

    if player_stats.leaderboard.is_empty() {
        let msg = "No completed shifts yet. Finish a shift to see your scores!";
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

            // Rank and score
            let rank_text = format!("#{} Score: {}", idx + 1, entry.score);
            draw_text(&rank_text, 50.0, y, 20.0, rank_color);

            // Status
            draw_text(status_icon, 250.0, y, 20.0, status_color);

            y += 25.0;

            // Details
            let details = format!(
                "  {} passengers | Difficulty {} | {} rule violations | {}",
                entry.passengers_transported,
                entry.difficulty_level,
                entry.rules_violated,
                entry.date
            );
            draw_text(&details, 70.0, y, 14.0, colors::TEXT_SECONDARY);

            y += 30.0;
        }
    }

    // Achievements section
    let achievements_x = screen_width() / 2.0 + 50.0;
    let mut achievements_y = 110.0;

    draw_text(
        "Achievements",
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
            colors::TEXT_MUTED // Using MUTED for locked instead of custom dark grey
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
        "Back to Menu (ESC)"
    ) {
        return UiAction::ReturnToMenu;
    }

    UiAction::None
}

//! Meta-progression screens: Skill Tree, Almanac, and Leaderboard.

use macroquad::prelude::*;
use macroquad_toolkit::ui::button;

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::{UiAction, colors};

/// Scroll offset for skill tree (static for simplicity)
static mut SKILL_TREE_SCROLL: f32 = 0.0;
static mut SKILL_TREE_SCROLL_TARGET: f32 = 0.0;

/// Draw the skill tree screen
pub fn draw_skill_tree(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        // Handle mouse wheel scroll with smooth interpolation
        let (_wheel_x, wheel_y) = mouse_wheel();
        unsafe {
            // Update target based on wheel input
            SKILL_TREE_SCROLL_TARGET = (SKILL_TREE_SCROLL_TARGET - wheel_y * 60.0).max(0.0);
            // Smoothly lerp current scroll toward target
            SKILL_TREE_SCROLL = SKILL_TREE_SCROLL + (SKILL_TREE_SCROLL_TARGET - SKILL_TREE_SCROLL) * 0.15;
        }
        let scroll_offset = unsafe { SKILL_TREE_SCROLL };

        // Title (fixed, not scrolled)
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

        // Bank balance (fixed)
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

        // Scroll hint
        draw_text("(Scroll with mouse wheel)", center_x - 80.0, 120.0, 12.0, colors::TEXT_MUTED);

        // Clip region start (content area)
        let content_top = 140.0;
        let content_bottom = screen_height() - 80.0;
        
        let mut y = content_top - scroll_offset;
        let categories = vec!["survival", "occult", "efficiency"];
        let loc_cats = &data.localization.ui.meta.skill_tree.categories;
        let category_names = vec![
            &loc_cats.survival,
            &loc_cats.occult,
            &loc_cats.efficiency,
        ];

        for (cat_idx, category) in categories.iter().enumerate() {
            // Category header
            if y > content_top - 30.0 && y < content_bottom {
                draw_text(
                    category_names[cat_idx],
                    50.0,
                    y,
                    24.0,
                    colors::ACCENT_WARNING,
                );
            }
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

                // Only draw if visible
                if y > content_top - 50.0 && y < content_bottom + 50.0 {
                    let text = format!("{} {} - ${}{}", skill.icon, skill.name, skill.cost, status);
                    draw_text(&text, 70.0, y, 18.0, color);
                }
                y += 25.0;

                // Description
                if y > content_top - 50.0 && y < content_bottom + 50.0 {
                    draw_text(&skill.description, 90.0, y, 14.0, colors::TEXT_SECONDARY);
                }
                y += 25.0;

                // Purchase button if can unlock and visible
                if can_unlock && y > content_top && y < content_bottom {
                    if button(90.0, y - 5.0, 150.0, 30.0, &data.localization.ui.meta.skill_tree.purchase) {
                        return UiAction::PurchaseSkill(skill.id.clone());
                    }
                    y += 40.0;
                } else if can_unlock {
                    y += 40.0; // Reserve space even if not visible
                }
            }

            y += 15.0;
        }

        // Back button (fixed at bottom)
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

/// Scroll offset for almanac (static for simplicity)
static mut ALMANAC_SCROLL: f32 = 0.0;
static mut ALMANAC_SCROLL_TARGET: f32 = 0.0;
static mut ALMANAC_SELECTED: Option<u32> = None;

/// Draw the almanac screen
pub fn draw_almanac(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    let center_x = screen_width() / 2.0;

    if let Some(data) = game_data {
        // Handle smooth scroll (much slower speed)
        let (_wheel_x, wheel_y) = mouse_wheel();
        unsafe {
            ALMANAC_SCROLL_TARGET = (ALMANAC_SCROLL_TARGET - wheel_y * 15.0).max(0.0);
            ALMANAC_SCROLL = ALMANAC_SCROLL + (ALMANAC_SCROLL_TARGET - ALMANAC_SCROLL) * 0.08;
        }
        let scroll_offset = unsafe { ALMANAC_SCROLL };
        let selected_id = unsafe { ALMANAC_SELECTED };

        // Title (fixed)
        let title = &data.localization.ui.meta.almanac.title;
        let title_size = 36.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            50.0,
            title_size,
            colors::ACCENT_PRIMARY,
        );

        // Lore fragments (fixed)
        let fragments = data.localization.ui.meta.almanac.fragments
            .replace("{}", &player_stats.lore_fragments.to_string());
        let fragments_width = measure_text(&fragments, None, 20, 1.0).width;
        draw_text(
            &fragments,
            center_x - fragments_width / 2.0,
            85.0,
            20.0,
            colors::ACCENT_GOLD,
        );

        // Scroll hint
        draw_text("(Scroll with mouse wheel, click to expand)", center_x - 120.0, 105.0, 12.0, colors::TEXT_MUTED);

        // Content area
        let content_top = 120.0;
        let content_bottom = screen_height() - 70.0;
        let card_width = screen_width().min(500.0) - 40.0;
        let card_x = center_x - card_width / 2.0;

        let mut y = content_top - scroll_offset;
        let mouse_pos = mouse_position();
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        for passenger in &data.passengers {
            let entry = player_stats.get_almanac_entry(passenger.id);
            let level_name = data.almanac.get_level(entry.knowledge_level)
                .map(|l| l.name.as_str())
                .unwrap_or(&data.localization.ui.meta.almanac.unknown_level);

            let is_selected = selected_id == Some(passenger.id);
            
            // Calculate card height based on state and knowledge level
            let base_height = if entry.encountered && entry.knowledge_level < 3 { 95.0 } else { 70.0 };
            // More height for higher knowledge levels (more info to show)
            let expanded_extra = if is_selected && entry.encountered { 
                match entry.knowledge_level {
                    0 => 60.0,  // Just basic info
                    1 => 100.0, // Description + traits
                    2 => 160.0, // + route preferences
                    _ => 180.0, // + backstory
                }
            } else { 
                0.0 
            };
            let card_height = base_height + expanded_extra;

            // Only draw if visible
            if y + card_height > content_top && y < content_bottom {
                // Calculate button area first to exclude from card click
                let btn_y = y + 58.0 + expanded_extra;
                let btn_rect = (card_x + 10.0, btn_y, 180.0, 30.0);
                let can_show_button = entry.encountered && entry.knowledge_level < 3;
                
                // Check if mouse is over the button area
                let is_over_button = can_show_button 
                    && mouse_pos.0 >= btn_rect.0 
                    && mouse_pos.0 <= btn_rect.0 + btn_rect.2
                    && mouse_pos.1 >= btn_rect.1 
                    && mouse_pos.1 <= btn_rect.1 + btn_rect.3;

                // Check for click on card (but NOT on button)
                let card_rect = (card_x, y, card_width, card_height);
                let is_hovered = mouse_pos.0 >= card_rect.0 
                    && mouse_pos.0 <= card_rect.0 + card_rect.2
                    && mouse_pos.1 >= card_rect.1 
                    && mouse_pos.1 <= card_rect.1 + card_rect.3
                    && mouse_pos.1 > content_top
                    && mouse_pos.1 < content_bottom;

                // Only toggle expand if clicking on card but NOT on the button
                if is_hovered && mouse_clicked && entry.encountered && !is_over_button {
                    unsafe {
                        if ALMANAC_SELECTED == Some(passenger.id) {
                            ALMANAC_SELECTED = None; // Deselect
                        } else {
                            ALMANAC_SELECTED = Some(passenger.id); // Select
                        }
                    }
                }

                // Card background
                let bg_color = if is_selected {
                    Color::from_hex(0x3a3a4a) // Highlighted when selected
                } else if entry.encountered {
                    Color::from_hex(0x2a2a3a)
                } else {
                    Color::from_hex(0x1a1a24)
                };
                draw_rectangle(card_x, y, card_width, card_height, bg_color);
                
                // Border
                let border_color = if is_selected {
                    colors::ACCENT_GOLD
                } else if entry.encountered {
                    colors::ACCENT_PRIMARY
                } else {
                    Color::from_hex(0x333344)
                };
                draw_rectangle_lines(card_x, y, card_width, card_height, if is_selected { 2.0 } else { 1.0 }, border_color);

                // Emoji and name
                let name_color = if entry.encountered { WHITE } else { colors::TEXT_MUTED };
                draw_text(&format!("{}", passenger.emoji), card_x + 10.0, y + 28.0, 24.0, name_color);
                draw_text(&passenger.name, card_x + 45.0, y + 26.0, 18.0, name_color);

                // Level indicator (right side)
                let level_text = format!("Lv.{}", entry.knowledge_level);
                let level_color = match entry.knowledge_level {
                    0 => colors::TEXT_MUTED,
                    1 => colors::ACCENT_SKY,
                    2 => colors::ACCENT_GOLD,
                    _ => colors::FUEL_GOOD,
                };
                draw_text(&level_text, card_x + card_width - 50.0, y + 26.0, 16.0, level_color);

                // Status text
                let status_color = if entry.encountered { colors::TEXT_SECONDARY } else { colors::TEXT_MUTED };
                draw_text(level_name, card_x + 45.0, y + 48.0, 14.0, status_color);

                // Expanded details when selected
                if is_selected && entry.encountered {
                    let mut details_y = y + base_height - 10.0;
                    
                    // Level 0+: Basic description
                    draw_text(&passenger.description, card_x + 15.0, details_y, 13.0, colors::TEXT_SECONDARY);
                    details_y += 18.0;
                    
                    // Level 1+: Traits
                    if entry.knowledge_level >= 1 {
                        if !passenger.traits.is_empty() {
                            let traits_text = format!("Traits: {}", passenger.traits.join(", "));
                            draw_text(&traits_text, card_x + 15.0, details_y, 12.0, colors::ACCENT_SKY);
                        }
                        details_y += 18.0;
                    }
                    
                    // Level 2+: Route preferences
                    if entry.knowledge_level >= 2 {
                        draw_text("Route Preferences:", card_x + 15.0, details_y, 12.0, colors::ACCENT_WARNING);
                        details_y += 16.0;
                        
                        for pref in &passenger.route_preferences {
                            let (icon, color) = match pref.preference {
                                crate::data::PreferenceLevel::Loves => ("💚", colors::FUEL_GOOD),
                                crate::data::PreferenceLevel::Likes => ("👍", colors::ACCENT_SKY),
                                crate::data::PreferenceLevel::Neutral => ("➖", colors::TEXT_MUTED),
                                crate::data::PreferenceLevel::Dislikes => ("👎", colors::ACCENT_WARNING),
                                crate::data::PreferenceLevel::Fears => ("💀", colors::FUEL_CRITICAL),
                            };
                            let route_name = format!("{:?}", pref.route);
                            let pref_text = format!("{} {} - {}", icon, route_name, pref.reason);
                            draw_text(&pref_text, card_x + 25.0, details_y, 11.0, color);
                            details_y += 14.0;
                        }
                    } else {
                        draw_text("🔒 Upgrade to Lv.2 to see route preferences", card_x + 15.0, details_y, 11.0, colors::TEXT_MUTED);
                        details_y += 16.0;
                    }
                    
                    // Level 3: Backstory
                    if entry.knowledge_level >= 3 {
                        let backstory_preview = if passenger.backstory_details.len() > 80 {
                            format!("📖 {}...", &passenger.backstory_details[..80])
                        } else {
                            format!("📖 {}", passenger.backstory_details)
                        };
                        draw_text(&backstory_preview, card_x + 15.0, details_y, 11.0, colors::ACCENT_GOLD);
                    } else if entry.knowledge_level >= 2 {
                        draw_text("🔒 Upgrade to Lv.3 to unlock backstory", card_x + 15.0, details_y, 11.0, colors::TEXT_MUTED);
                    }
                }

                // Upgrade button if can upgrade
                if entry.encountered && entry.knowledge_level < 3 {
                    let cost = data.almanac.get_upgrade_cost(entry.knowledge_level + 1);
                    let can_afford = player_stats.lore_fragments >= cost;
                    let button_text = format!("⬆ Upgrade ({} fragments)", cost);
                    let btn_y = y + 58.0 + expanded_extra;

                    if can_afford {
                        if button(card_x + 10.0, btn_y, 180.0, 30.0, &button_text) {
                            return UiAction::UpgradeAlmanacKnowledge(passenger.id);
                        }
                    } else {
                        draw_text(&button_text, card_x + 15.0, btn_y + 20.0, 13.0, colors::TEXT_MUTED);
                    }
                }
            }

            y += card_height + 8.0;
        }

        // Back button (fixed at bottom)
        if button(
            center_x - 100.0,
            screen_height() - 55.0,
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

//! Meta-progression screens: Skill Tree, Almanac, and Leaderboard.

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_small_caps,
    draw_wrapped_text, fonts, UiAction, UiRect,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Scroll offset for skill tree (static for simplicity)
static mut SKILL_TREE_SCROLL: f32 = 0.0;
static mut SKILL_TREE_SCROLL_TARGET: f32 = 0.0;

fn skill_category_label(category: &str, data: &GameData) -> String {
    let loc_cats = &data.localization.ui.meta.skill_tree.categories;
    match category {
        "survival" => loc_cats.survival.clone(),
        "occult" => loc_cats.occult.clone(),
        "efficiency" => loc_cats.efficiency.clone(),
        _ => category.to_string(),
    }
}

fn draw_skill_category_mark(category: &str, x: f32, y: f32, color: Color) {
    match category {
        "survival" => {
            draw_rectangle_lines(x - 10.0, y - 13.0, 20.0, 25.0, 2.0, color);
            draw_line(x, y - 18.0, x, y + 18.0, 2.0, color);
            draw_line(x - 14.0, y - 4.0, x + 14.0, y - 4.0, 2.0, color);
        }
        "occult" => {
            draw_circle_lines(x, y, 16.0, 2.0, color);
            draw_circle_lines(x, y, 6.0, 1.5, color);
            draw_line(x - 20.0, y, x + 20.0, y, 1.5, color);
        }
        "efficiency" => {
            draw_circle_lines(x, y, 15.0, 2.0, color);
            draw_line(x - 9.0, y + 10.0, x + 11.0, y - 10.0, 2.0, color);
            draw_line(x - 3.0, y + 10.0, x + 13.0, y + 10.0, 2.0, color);
        }
        _ => {
            draw_circle_lines(x, y, 14.0, 2.0, color);
        }
    }
}

fn draw_skill_card(
    rect: UiRect,
    skill: &crate::data::Skill,
    is_unlocked: bool,
    can_unlock: bool,
    player_stats: &PlayerStats,
    data: &GameData,
) -> UiAction {
    let (border, status, status_color) = if is_unlocked {
        (
            colors::FUEL_GOOD,
            data.localization.ui.meta.skill_tree.unlocked.as_str(),
            colors::FUEL_GOOD,
        )
    } else if can_unlock {
        (colors::CAB_YELLOW, "Available", colors::CAB_YELLOW)
    } else {
        (colors::BORDER_DIM, "Locked", colors::TEXT_MUTED)
    };

    let bg = if is_unlocked {
        Color::new(0.035, 0.090, 0.045, 0.82)
    } else if can_unlock {
        Color::new(0.100, 0.075, 0.025, 0.82)
    } else {
        colors::GLASS
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle(rect.x, rect.y, 4.0, rect.h, border);
    draw_rectangle(rect.x, rect.y, rect.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.10));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, border);

    let icon_x = rect.x + 24.0;
    let icon_y = rect.y + 30.0;
    draw_circle_lines(icon_x, icon_y, 15.0, 1.5, colors::TEXT_MUTED);
    let initials = skill
        .name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>();
    let initials_width = measure_ui_text(&initials, None, fonts::SIZE_XS as u16, 1.0).width;
    draw_ui_text(
        &initials,
        icon_x - initials_width / 2.0,
        icon_y + 4.0,
        fonts::SIZE_XS,
        colors::TEXT_SECONDARY,
    );

    let text_x = rect.x + 54.0;
    draw_ui_text(
        &skill.name,
        text_x,
        rect.y + 27.0,
        fonts::SIZE_MD,
        colors::TEXT_PRIMARY,
    );
    draw_small_caps(
        &format!("${}  {}", skill.cost, status),
        text_x,
        rect.y + 47.0,
        fonts::SIZE_XS,
        status_color,
    );

    let desc_bottom = draw_wrapped_text(
        &skill.description,
        text_x,
        rect.y + 66.0,
        rect.w - 72.0,
        fonts::SIZE_XS,
        15.0,
        colors::TEXT_SECONDARY,
        2,
    );

    let prereq_text = if skill.prerequisites.is_empty() {
        "No prerequisite".to_string()
    } else {
        let met = skill
            .prerequisites
            .iter()
            .filter(|id| player_stats.unlocked_skills.contains(id))
            .count();
        format!("Prerequisites {}/{}", met, skill.prerequisites.len())
    };
    draw_small_caps(
        &prereq_text,
        text_x,
        (desc_bottom + 12.0).min(rect.y + rect.h - 18.0),
        fonts::SIZE_XS,
        colors::TEXT_MUTED,
    );

    if can_unlock {
        let buy_rect = UiRect::new(rect.x + rect.w - 116.0, rect.y + rect.h - 36.0, 96.0, 26.0);
        if draw_glass_button(
            buy_rect,
            &data.localization.ui.meta.skill_tree.purchase,
            colors::CAB_YELLOW,
            true,
        ) {
            return UiAction::PurchaseSkill(skill.id.clone());
        }
    }

    UiAction::None
}

/// Draw the skill tree screen
pub fn draw_skill_tree(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();

    if let Some(data) = game_data {
        // Handle mouse wheel scroll with smooth interpolation
        let (_wheel_x, wheel_y) = mouse_wheel();
        unsafe {
            // Update target based on wheel input
            SKILL_TREE_SCROLL_TARGET = (SKILL_TREE_SCROLL_TARGET - wheel_y * 60.0).max(0.0);
            // Smoothly lerp current scroll toward target
            SKILL_TREE_SCROLL =
                SKILL_TREE_SCROLL + (SKILL_TREE_SCROLL_TARGET - SKILL_TREE_SCROLL) * 0.15;
        }
        let scroll_offset = unsafe { SKILL_TREE_SCROLL };

        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let margin = (screen_w * 0.045).clamp(30.0, 70.0);
        let header_h = 92.0;
        let footer_h = 66.0;
        let content_top = header_h + 26.0;
        let content_bottom = screen_h - footer_h;
        let content_h = content_bottom - content_top;

        let header_rect = UiRect::new(margin, 28.0, screen_w - margin * 2.0, header_h);
        draw_glass_panel(header_rect, colors::BORDER_DIM);
        let header_inner = header_rect.inset(18.0);

        let title = &data.localization.ui.meta.skill_tree.title;
        draw_ui_text(
            title,
            header_inner.x,
            header_inner.y + 34.0,
            fonts::SIZE_XXL,
            colors::FUEL_GOOD,
        );

        let balance = data
            .localization
            .ui
            .meta
            .skill_tree
            .bank_balance
            .replace("{}", &player_stats.bank_balance.to_string());
        draw_small_caps(
            &balance,
            header_inner.x,
            header_inner.y + 62.0,
            fonts::SIZE_MD,
            colors::CAB_YELLOW,
        );
        draw_small_caps(
            "Scroll to inspect upgrades. Purchase buttons appear when requirements are met.",
            header_inner.x + header_inner.w * 0.52,
            header_inner.y + 52.0,
            fonts::SIZE_XS,
            colors::TEXT_MUTED,
        );

        let categories = ["survival", "occult", "efficiency"];
        let wide_layout = screen_w >= 1080.0;
        let col_gap = 18.0;
        let col_count = if wide_layout {
            categories.len() as f32
        } else {
            1.0
        };
        let column_w = ((screen_w - margin * 2.0 - col_gap * (col_count - 1.0)) / col_count)
            .clamp(260.0, 560.0);
        let card_h = if wide_layout { 122.0 } else { 112.0 };
        let card_gap = 10.0;
        let mut max_content_bottom: f32 = 0.0;

        for (cat_idx, category) in categories.iter().enumerate() {
            let column_x = if wide_layout {
                margin + cat_idx as f32 * (column_w + col_gap)
            } else {
                center_x - column_w / 2.0
            };
            let mut y = if wide_layout {
                content_top
            } else {
                content_top + cat_idx as f32 * 10.0
            } - scroll_offset;

            let skills_in_category = data
                .skills
                .iter()
                .filter(|s| s.category == *category)
                .collect::<Vec<_>>();
            let unlocked_count = skills_in_category
                .iter()
                .filter(|skill| player_stats.is_skill_unlocked(&skill.id))
                .count();

            let category_h = 58.0 + skills_in_category.len() as f32 * (card_h + card_gap) + 12.0;
            if !wide_layout {
                y += max_content_bottom.max(0.0);
                max_content_bottom += category_h + 18.0;
            }

            if y + category_h > content_top && y < content_bottom {
                let panel_rect =
                    UiRect::new(column_x, y, column_w, category_h.min(content_h + 80.0));
                draw_glass_panel(panel_rect, colors::BORDER_DIM);
                draw_skill_category_mark(category, column_x + 28.0, y + 30.0, colors::CAB_YELLOW);
                draw_small_caps(
                    &skill_category_label(category, data),
                    column_x + 56.0,
                    y + 26.0,
                    fonts::SIZE_LG,
                    colors::CAB_YELLOW,
                );
                draw_small_caps(
                    &format!("{}/{} unlocked", unlocked_count, skills_in_category.len()),
                    column_x + 56.0,
                    y + 46.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
            }

            y += 58.0;

            // Skills in category
            for skill in skills_in_category {
                let is_unlocked = player_stats.is_skill_unlocked(&skill.id);
                let can_unlock = skill.can_unlock(&player_stats.unlocked_skills)
                    && !is_unlocked
                    && player_stats.bank_balance >= skill.cost;

                let card_rect = UiRect::new(column_x + 12.0, y, column_w - 24.0, card_h);
                if y + card_h > content_top && y < content_bottom {
                    let action = draw_skill_card(
                        card_rect,
                        skill,
                        is_unlocked,
                        can_unlock,
                        player_stats,
                        data,
                    );
                    if action != UiAction::None {
                        return action;
                    }
                }
                y += card_h + card_gap;
            }

            max_content_bottom = max_content_bottom.max(y - content_top + scroll_offset);
        }

        draw_rectangle(
            0.0,
            0.0,
            screen_w,
            content_top - 8.0,
            Color::new(0.0, 0.0, 0.0, 0.10),
        );
        draw_rectangle(
            0.0,
            content_bottom,
            screen_w,
            screen_h - content_bottom,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        let back_rect = UiRect::new(center_x - 108.0, screen_h - 56.0, 216.0, 40.0);
        if draw_glass_button(
            back_rect,
            &data.localization.ui.common.back_button,
            colors::ACCENT_SKY,
            true,
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
    draw_noir_city_background();

    if let Some(data) = game_data {
        // Handle smooth scroll (much slower speed)
        let (_wheel_x, wheel_y) = mouse_wheel();
        unsafe {
            ALMANAC_SCROLL_TARGET = (ALMANAC_SCROLL_TARGET - wheel_y * 15.0).max(0.0);
            ALMANAC_SCROLL = ALMANAC_SCROLL + (ALMANAC_SCROLL_TARGET - ALMANAC_SCROLL) * 0.08;
        }
        let scroll_offset = unsafe { ALMANAC_SCROLL };
        let selected_id = unsafe { ALMANAC_SELECTED };

        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let margin = (screen_w * 0.045).clamp(30.0, 70.0);
        let header_h = 92.0;
        let footer_h = 66.0;
        let content_top = header_h + 28.0;
        let content_bottom = screen_h - footer_h;
        let content_h = content_bottom - content_top;

        let header_rect = UiRect::new(margin, 28.0, screen_w - margin * 2.0, header_h);
        draw_glass_panel(header_rect, colors::BORDER_DIM);
        let header_inner = header_rect.inset(18.0);

        let title = &data.localization.ui.meta.almanac.title;
        draw_ui_text(
            title,
            header_inner.x,
            header_inner.y + 34.0,
            fonts::SIZE_XXL,
            colors::CAB_YELLOW,
        );

        let fragments = data
            .localization
            .ui
            .meta
            .almanac
            .fragments
            .replace("{}", &player_stats.lore_fragments.to_string());
        draw_small_caps(
            &fragments,
            header_inner.x,
            header_inner.y + 62.0,
            fonts::SIZE_MD,
            colors::ACCENT_GOLD,
        );
        draw_small_caps(
            "Click encountered passengers to expand. Spend lore fragments to reveal more.",
            header_inner.x + header_inner.w * 0.52,
            header_inner.y + 52.0,
            fonts::SIZE_XS,
            colors::TEXT_MUTED,
        );
        draw_rectangle(
            margin,
            content_top - 2.0,
            screen_w - margin * 2.0,
            content_h + 4.0,
            Color::new(0.0, 0.0, 0.0, 0.34),
        );

        let wide_layout = screen_w >= 1180.0;
        let column_count = if wide_layout { 2.0 } else { 1.0 };
        let gap = 18.0;
        let card_width = ((screen_w - margin * 2.0 - gap * (column_count - 1.0)) / column_count)
            .clamp(310.0, 860.0);
        let left_x = if wide_layout {
            center_x - card_width - gap / 2.0
        } else {
            center_x - card_width / 2.0
        };

        let mut y = content_top - scroll_offset;
        let mut right_y = content_top - scroll_offset;
        let mut content_extent: f32 = 0.0;
        let mouse_pos = mouse_position();
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        for (idx, passenger) in data.passengers.iter().enumerate() {
            let entry = player_stats.get_almanac_entry(passenger.id);
            let level_name = data
                .almanac
                .get_level(entry.knowledge_level)
                .map(|l| l.name.as_str())
                .unwrap_or(&data.localization.ui.meta.almanac.unknown_level);

            let is_selected = selected_id == Some(passenger.id);

            // Calculate card height based on state and knowledge level
            let base_height = if entry.encountered && entry.knowledge_level < 3 {
                95.0
            } else {
                70.0
            };
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
            let card_height = base_height + expanded_extra + 18.0;
            let card_x = if wide_layout && idx % 2 == 1 {
                left_x + card_width + gap
            } else {
                left_x
            };
            let card_y = if wide_layout && idx % 2 == 1 {
                right_y
            } else {
                y
            };

            // Only draw if visible
            if card_y + card_height > content_top && card_y < content_bottom {
                // Calculate button area first to exclude from card click
                let btn_y = card_y + card_height - 42.0;
                let btn_rect = (card_x + card_width - 220.0, btn_y, 198.0, 30.0);
                let can_show_button = entry.encountered && entry.knowledge_level < 3;

                // Check if mouse is over the button area
                let is_over_button = can_show_button
                    && mouse_pos.0 >= btn_rect.0
                    && mouse_pos.0 <= btn_rect.0 + btn_rect.2
                    && mouse_pos.1 >= btn_rect.1
                    && mouse_pos.1 <= btn_rect.1 + btn_rect.3;

                // Check for click on card (but NOT on button)
                let card_rect = (card_x, card_y, card_width, card_height);
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

                let border_color = if is_selected {
                    colors::ACCENT_GOLD
                } else if entry.encountered {
                    colors::ACCENT_SKY
                } else {
                    colors::BORDER_DIM
                };
                let bg_color = if is_selected {
                    Color::new(0.095, 0.075, 0.030, 0.94)
                } else if entry.encountered {
                    Color::new(0.030, 0.055, 0.060, 0.93)
                } else {
                    Color::new(0.025, 0.030, 0.032, 0.92)
                };
                draw_rectangle(card_x, card_y, card_width, card_height, bg_color);
                draw_rectangle(card_x, card_y, 4.0, card_height, border_color);
                draw_rectangle(
                    card_x,
                    card_y,
                    card_width,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.10),
                );
                draw_rectangle_lines(
                    card_x,
                    card_y,
                    card_width,
                    card_height,
                    if is_selected { 2.0 } else { 1.0 },
                    border_color,
                );

                let name_color = if entry.encountered {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_MUTED
                };
                let portrait_x = card_x + 32.0;
                let portrait_y = card_y + 34.0;
                draw_circle_lines(portrait_x, portrait_y, 18.0, 1.5, border_color);
                let initials = passenger
                    .name
                    .split_whitespace()
                    .filter_map(|part| part.chars().next())
                    .take(2)
                    .collect::<String>();
                let initials_width =
                    measure_ui_text(&initials, None, fonts::SIZE_XS as u16, 1.0).width;
                draw_ui_text(
                    &initials,
                    portrait_x - initials_width / 2.0,
                    portrait_y + 4.0,
                    fonts::SIZE_XS,
                    name_color,
                );
                let text_x = card_x + 64.0;
                let display_name = if entry.encountered {
                    passenger.name.as_str()
                } else {
                    "Unknown Passenger"
                };
                draw_ui_text(
                    display_name,
                    text_x,
                    card_y + 29.0,
                    fonts::SIZE_LG,
                    name_color,
                );

                let level_text = format!("Lv.{}", entry.knowledge_level);
                let level_color = match entry.knowledge_level {
                    0 => colors::TEXT_MUTED,
                    1 => colors::ACCENT_SKY,
                    2 => colors::ACCENT_GOLD,
                    _ => colors::FUEL_GOOD,
                };
                draw_small_caps(
                    &level_text,
                    card_x + card_width - 72.0,
                    card_y + 29.0,
                    fonts::SIZE_SM,
                    level_color,
                );

                let status_color = if entry.encountered {
                    colors::TEXT_SECONDARY
                } else {
                    colors::TEXT_MUTED
                };
                draw_small_caps(
                    level_name,
                    text_x,
                    card_y + 50.0,
                    fonts::SIZE_XS,
                    status_color,
                );

                // Expanded details when selected
                if is_selected && entry.encountered {
                    let mut details_y = card_y + 78.0;

                    // Level 0+: Basic description
                    details_y = draw_wrapped_text(
                        &passenger.description,
                        text_x,
                        details_y,
                        card_width - 92.0,
                        fonts::SIZE_XS,
                        15.0,
                        colors::TEXT_SECONDARY,
                        2,
                    );
                    details_y += 12.0;

                    // Level 1+: Traits
                    if entry.knowledge_level >= 1 {
                        if !passenger.traits.is_empty() {
                            let traits_text = format!("Traits: {}", passenger.traits.join(", "));
                            details_y = draw_wrapped_text(
                                &traits_text,
                                text_x,
                                details_y,
                                card_width - 92.0,
                                fonts::SIZE_XS,
                                15.0,
                                colors::ACCENT_SKY,
                                2,
                            );
                        }
                        details_y += 10.0;
                    }

                    // Level 2+: Route preferences
                    if entry.knowledge_level >= 2 {
                        draw_small_caps(
                            "Route Preferences:",
                            text_x,
                            details_y,
                            fonts::SIZE_XS,
                            colors::CAB_YELLOW,
                        );
                        details_y += 16.0;

                        for pref in &passenger.route_preferences {
                            let (label, color) = match pref.preference {
                                crate::data::PreferenceLevel::Loves => ("Loves", colors::FUEL_GOOD),
                                crate::data::PreferenceLevel::Likes => {
                                    ("Likes", colors::ACCENT_SKY)
                                }
                                crate::data::PreferenceLevel::Neutral => {
                                    ("Neutral", colors::TEXT_MUTED)
                                }
                                crate::data::PreferenceLevel::Dislikes => {
                                    ("Dislikes", colors::ACCENT_WARNING)
                                }
                                crate::data::PreferenceLevel::Fears => {
                                    ("Fears", colors::FUEL_CRITICAL)
                                }
                            };
                            let route_name = format!("{:?}", pref.route);
                            let pref_text = format!("{} {} - {}", label, route_name, pref.reason);
                            details_y = draw_wrapped_text(
                                &pref_text,
                                text_x + 10.0,
                                details_y,
                                card_width - 108.0,
                                fonts::SIZE_XS,
                                14.0,
                                color,
                                1,
                            );
                            details_y += 4.0;
                        }
                    } else {
                        draw_small_caps(
                            "Upgrade to Lv.2 to see route preferences",
                            text_x,
                            details_y,
                            fonts::SIZE_XS,
                            colors::TEXT_MUTED,
                        );
                        details_y += 16.0;
                    }

                    // Level 3: Backstory
                    if entry.knowledge_level >= 3 {
                        let backstory_preview = if passenger.backstory_details.len() > 80 {
                            format!("Backstory: {}...", &passenger.backstory_details[..80])
                        } else {
                            format!("Backstory: {}", passenger.backstory_details)
                        };
                        draw_wrapped_text(
                            &backstory_preview,
                            text_x,
                            details_y,
                            card_width - 92.0,
                            fonts::SIZE_XS,
                            14.0,
                            colors::ACCENT_GOLD,
                            2,
                        );
                    } else if entry.knowledge_level >= 2 {
                        draw_small_caps(
                            "Upgrade to Lv.3 to unlock backstory",
                            text_x,
                            details_y,
                            fonts::SIZE_XS,
                            colors::TEXT_MUTED,
                        );
                    }
                } else {
                    let summary = if entry.encountered {
                        passenger.description.as_str()
                    } else {
                        "No confirmed encounter. Complete rides to add this passenger to the almanac."
                    };
                    draw_wrapped_text(
                        summary,
                        text_x,
                        card_y + 72.0,
                        card_width - 92.0,
                        fonts::SIZE_XS,
                        15.0,
                        status_color,
                        2,
                    );
                }

                // Upgrade button if can upgrade
                if entry.encountered && entry.knowledge_level < 3 {
                    let cost = data.almanac.get_upgrade_cost(entry.knowledge_level + 1);
                    let can_afford = player_stats.lore_fragments >= cost;
                    let cost_text = data
                        .localization
                        .ui
                        .meta
                        .almanac
                        .upgrade_cost
                        .replace("{}", &cost.to_string());
                    let button_text = cost_text;

                    if can_afford {
                        if draw_glass_button(
                            UiRect::new(btn_rect.0, btn_rect.1, btn_rect.2, btn_rect.3),
                            &button_text,
                            colors::ACCENT_GOLD,
                            true,
                        ) {
                            return UiAction::UpgradeAlmanacKnowledge(passenger.id);
                        }
                    } else {
                        draw_small_caps(
                            &button_text,
                            card_x + card_width - 220.0,
                            btn_y + 20.0,
                            fonts::SIZE_XS,
                            colors::TEXT_MUTED,
                        );
                    }
                }
            }

            if wide_layout && idx % 2 == 1 {
                right_y += card_height + 12.0;
                content_extent = content_extent.max(right_y - content_top + scroll_offset);
            } else {
                y += card_height + 12.0;
                content_extent = content_extent.max(y - content_top + scroll_offset);
            }
        }

        let max_scroll = (content_extent - content_h).max(0.0);
        unsafe {
            ALMANAC_SCROLL_TARGET = ALMANAC_SCROLL_TARGET.min(max_scroll);
            ALMANAC_SCROLL = ALMANAC_SCROLL.min(max_scroll);
        }

        draw_rectangle(
            0.0,
            content_bottom,
            screen_w,
            screen_h - content_bottom,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        let back_rect = UiRect::new(center_x - 108.0, screen_h - 56.0, 216.0, 40.0);
        if draw_glass_button(
            back_rect,
            &data.localization.ui.common.back_button,
            colors::ACCENT_SKY,
            true,
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

/// Draw the leaderboard and achievements screen
pub fn draw_leaderboard(player_stats: &PlayerStats, game_data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();

    // We need data for localization
    if let Some(data) = game_data {
        let screen_w = screen_width();
        let screen_h = screen_height();
        let center_x = screen_w / 2.0;
        let margin = (screen_w * 0.045).clamp(30.0, 70.0);
        let header_h = 92.0;
        let footer_h = 66.0;
        let content_top = header_h + 28.0;
        let content_bottom = screen_h - footer_h;
        let content_h = content_bottom - content_top;

        let header_rect = UiRect::new(margin, 28.0, screen_w - margin * 2.0, header_h);
        draw_glass_panel(header_rect, colors::BORDER_DIM);
        let header_inner = header_rect.inset(18.0);

        let title = &data.localization.ui.meta.leaderboard.title;
        draw_ui_text(
            title,
            header_inner.x,
            header_inner.y + 34.0,
            fonts::SIZE_XXL,
            colors::CAB_YELLOW,
        );

        let unlocked_count = player_stats
            .achievements
            .iter()
            .filter(|achievement| player_stats.is_achievement_unlocked(&achievement.id))
            .count();
        draw_small_caps(
            &format!(
                "{} runs recorded | {}/{} achievements unlocked",
                player_stats.leaderboard.len(),
                unlocked_count,
                player_stats.achievements.len()
            ),
            header_inner.x,
            header_inner.y + 62.0,
            fonts::SIZE_MD,
            colors::ACCENT_GOLD,
        );

        draw_small_caps(
            "Completed shifts are ranked by final score. Achievements track long-term progress.",
            header_inner.x + header_inner.w * 0.52,
            header_inner.y + 52.0,
            fonts::SIZE_XS,
            colors::TEXT_MUTED,
        );

        let wide_layout = screen_w >= 1080.0;
        let gap = 18.0;
        let left_w = if wide_layout {
            ((screen_w - margin * 2.0 - gap) * 0.58).clamp(500.0, 920.0)
        } else {
            (screen_w - margin * 2.0).clamp(310.0, 760.0)
        };
        let right_w = if wide_layout {
            screen_w - margin * 2.0 - gap - left_w
        } else {
            left_w
        };
        let left_x = if wide_layout {
            margin
        } else {
            center_x - left_w / 2.0
        };
        let right_x = if wide_layout {
            left_x + left_w + gap
        } else {
            center_x - right_w / 2.0
        };

        draw_rectangle(
            margin,
            content_top - 2.0,
            screen_w - margin * 2.0,
            content_h + 4.0,
            Color::new(0.0, 0.0, 0.0, 0.34),
        );

        let left_panel = UiRect::new(left_x, content_top, left_w, content_h);
        draw_glass_panel(left_panel, colors::BORDER_DIM);
        let left_inner = left_panel.inset(18.0);
        draw_small_caps(
            &data.localization.ui.meta.leaderboard.top_runs,
            left_inner.x,
            left_inner.y + 16.0,
            fonts::SIZE_LG,
            colors::CAB_YELLOW,
        );

        let mut y = left_inner.y + 46.0;

        if player_stats.leaderboard.is_empty() {
            let msg = &data.localization.ui.meta.leaderboard.no_runs;
            draw_rectangle(
                left_inner.x,
                y,
                left_inner.w,
                128.0,
                Color::new(0.025, 0.030, 0.032, 0.92),
            );
            draw_rectangle_lines(
                left_inner.x,
                y,
                left_inner.w,
                128.0,
                1.0,
                colors::BORDER_DIM,
            );
            draw_wrapped_text(
                msg,
                left_inner.x + 18.0,
                y + 44.0,
                left_inner.w - 36.0,
                fonts::SIZE_MD,
                20.0,
                colors::TEXT_MUTED,
                3,
            );
        } else {
            for (idx, entry) in player_stats.leaderboard.iter().enumerate() {
                let rank_color = match idx {
                    0 => colors::ACCENT_GOLD,
                    1 => colors::TEXT_PRIMARY,
                    2 => colors::ACCENT_WARNING,
                    _ => colors::TEXT_SECONDARY,
                };

                let card_h = 66.0;
                let card_rect = UiRect::new(left_inner.x, y, left_inner.w, card_h);
                let status_color = if entry.survived {
                    colors::FUEL_GOOD
                } else {
                    colors::FUEL_CRITICAL
                };
                draw_rectangle(
                    card_rect.x,
                    card_rect.y,
                    card_rect.w,
                    card_rect.h,
                    Color::new(0.025, 0.030, 0.032, 0.92),
                );
                draw_rectangle(card_rect.x, card_rect.y, 4.0, card_rect.h, rank_color);
                draw_rectangle(
                    card_rect.x,
                    card_rect.y,
                    card_rect.w,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.10),
                );
                draw_rectangle_lines(
                    card_rect.x,
                    card_rect.y,
                    card_rect.w,
                    card_rect.h,
                    1.0,
                    if idx < 3 {
                        rank_color
                    } else {
                        colors::BORDER_DIM
                    },
                );

                // Rank and score: "#{} Score: {}"
                let rank_text = data
                    .localization
                    .ui
                    .meta
                    .leaderboard
                    .score_entry
                    .replacen("{}", &(idx + 1).to_string(), 1)
                    .replacen("{}", &entry.score.to_string(), 1);

                draw_ui_text(
                    &rank_text,
                    card_rect.x + 18.0,
                    card_rect.y + 27.0,
                    fonts::SIZE_LG,
                    rank_color,
                );
                draw_small_caps(
                    if entry.survived {
                        "Survived"
                    } else {
                        "Lost Shift"
                    },
                    card_rect.x + card_rect.w - 122.0,
                    card_rect.y + 27.0,
                    fonts::SIZE_XS,
                    status_color,
                );

                // Details: "  {} passengers | Difficulty {} | {} rule violations | {}"
                let details = data
                    .localization
                    .ui
                    .meta
                    .leaderboard
                    .details
                    .replacen("{}", &entry.passengers_transported.to_string(), 1)
                    .replacen("{}", &entry.difficulty_level.to_string(), 1)
                    .replacen("{}", &entry.rules_violated.to_string(), 1)
                    .replacen("{}", &entry.date, 1);

                draw_small_caps(
                    &details,
                    card_rect.x + 18.0,
                    card_rect.y + 50.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );

                y += card_h + 10.0;
                if y > left_panel.bottom() - 34.0 {
                    break;
                }
            }
        }

        let right_panel_y = if wide_layout { content_top } else { y + 18.0 };
        let right_panel_h = if wide_layout {
            content_h
        } else {
            (content_bottom - right_panel_y).max(220.0)
        };
        let right_panel = UiRect::new(right_x, right_panel_y, right_w, right_panel_h);
        draw_glass_panel(right_panel, colors::BORDER_DIM);
        let right_inner = right_panel.inset(18.0);
        draw_small_caps(
            &data.localization.ui.meta.leaderboard.achievements,
            right_inner.x,
            right_inner.y + 16.0,
            fonts::SIZE_LG,
            colors::CAB_YELLOW,
        );
        let mut achievements_y = right_inner.y + 46.0;

        for achievement in &player_stats.achievements {
            let unlocked = player_stats.is_achievement_unlocked(&achievement.id);
            let border = if unlocked {
                colors::FUEL_GOOD
            } else {
                colors::BORDER_DIM
            };
            let card_h = 82.0;
            let card_rect = UiRect::new(right_inner.x, achievements_y, right_inner.w, card_h);
            draw_rectangle(
                card_rect.x,
                card_rect.y,
                card_rect.w,
                card_rect.h,
                if unlocked {
                    Color::new(0.035, 0.090, 0.045, 0.88)
                } else {
                    Color::new(0.025, 0.030, 0.032, 0.92)
                },
            );
            draw_rectangle(card_rect.x, card_rect.y, 4.0, card_rect.h, border);
            draw_rectangle_lines(
                card_rect.x,
                card_rect.y,
                card_rect.w,
                card_rect.h,
                1.0,
                border,
            );

            let status = if unlocked { "Unlocked" } else { "Locked" };
            draw_ui_text(
                &achievement.name,
                card_rect.x + 18.0,
                card_rect.y + 27.0,
                fonts::SIZE_MD,
                if unlocked {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_MUTED
                },
            );
            draw_small_caps(
                status,
                card_rect.x + card_rect.w - 96.0,
                card_rect.y + 26.0,
                fonts::SIZE_XS,
                if unlocked {
                    colors::FUEL_GOOD
                } else {
                    colors::TEXT_MUTED
                },
            );

            // Description
            draw_wrapped_text(
                &achievement.description,
                card_rect.x + 18.0,
                card_rect.y + 48.0,
                card_rect.w - 36.0,
                fonts::SIZE_XS,
                14.0,
                colors::TEXT_MUTED,
                2,
            );
            achievements_y += card_h + 10.0;
            if achievements_y > right_panel.bottom() - 34.0 {
                break;
            }
        }

        draw_rectangle(
            0.0,
            content_bottom,
            screen_w,
            screen_h - content_bottom,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );
        let back_rect = UiRect::new(center_x - 108.0, screen_h - 56.0, 216.0, 40.0);
        if draw_glass_button(
            back_rect,
            &data.localization.ui.common.back_button,
            colors::ACCENT_SKY,
            true,
        ) {
            return UiAction::ReturnToMenu;
        }
    }

    UiAction::None
}

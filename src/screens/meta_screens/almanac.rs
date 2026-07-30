//! Almanac screen: passenger knowledge progression, spent via lore fragments.

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_small_caps,
    draw_wrapped_text, fonts, UiAction, UiRect,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, ScrollArea};

/// Height (in unscrolled content-space) a single almanac card occupies,
/// mirroring the sizing rules the draw loop below applies.
/// `story_known` covers a backstory earned in play rather than bought with
/// lore, which shows the same block and so needs the same room.
fn card_height_for(
    entry: &crate::state::AlmanacEntry,
    is_selected: bool,
    story_known: bool,
) -> f32 {
    let base_height = if entry.encountered && entry.knowledge_level < 3 {
        95.0
    } else {
        70.0
    };
    let expanded_extra = if is_selected && entry.encountered {
        match entry.knowledge_level {
            _ if story_known => 180.0,
            0 => 60.0,  // Just basic info
            1 => 100.0, // Description + traits
            2 => 160.0, // + route preferences
            _ => 180.0, // + backstory
        }
    } else {
        0.0
    };
    base_height + expanded_extra + 18.0
}

/// Content height (in unscrolled content-space) needed to lay out every
/// passenger card, mirroring the two-column accumulation the draw loop
/// below performs. Computed as a cheap pre-pass (no drawing) so the
/// `ScrollArea` can clamp and draw its scrollbar before the real layout
/// pass runs.
fn compute_content_height(
    data: &GameData,
    player_stats: &PlayerStats,
    selected_id: Option<u32>,
    wide_layout: bool,
) -> f32 {
    let mut y = 0.0_f32;
    let mut right_y = 0.0_f32;
    let mut extent = 0.0_f32;
    for (idx, passenger) in data.passengers.iter().enumerate() {
        let entry = player_stats.get_almanac_entry(passenger.id);
        let card_height = card_height_for(
            &entry,
            selected_id == Some(passenger.id),
            player_stats.is_backstory_unlocked(passenger.id),
        );
        if wide_layout && idx % 2 == 1 {
            right_y += card_height + 12.0;
            extent = extent.max(right_y);
        } else {
            y += card_height + 12.0;
            extent = extent.max(y);
        }
    }
    extent
}

/// Draw the almanac screen
pub fn draw_almanac(
    player_stats: &PlayerStats,
    game_data: Option<&GameData>,
    scroll: &mut ScrollArea,
    selected: &mut Option<u32>,
) -> UiAction {
    draw_noir_city_background();

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

        let selected_id = *selected;
        let content_height = compute_content_height(data, player_stats, selected_id, wide_layout);
        let list_view = Rect::new(margin, content_top, screen_w - margin * 2.0, content_h);
        scroll.update(list_view, content_height);
        let scroll_offset = scroll.offset();

        let mut y = content_top - scroll_offset;
        let mut right_y = content_top - scroll_offset;
        let mouse_y = mouse_position().1;
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

        for (idx, passenger) in data.passengers.iter().enumerate() {
            let entry = player_stats.get_almanac_entry(passenger.id);
            let level_name = data
                .almanac
                .get_level(entry.knowledge_level)
                .map(|l| l.name.as_str())
                .unwrap_or(&data.localization.ui.meta.almanac.unknown_level);

            let is_selected = selected_id == Some(passenger.id);
            let story_known = player_stats.is_backstory_unlocked(passenger.id);
            let card_height = card_height_for(&entry, is_selected, story_known);
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
                let btn_rect = UiRect::new(card_x + card_width - 220.0, btn_y, 198.0, 30.0);
                let can_show_button = entry.encountered && entry.knowledge_level < 3;

                // Check if mouse is over the button area
                let is_over_button = can_show_button && btn_rect.contains_mouse();

                // Check for click on card (but NOT on button)
                let card_rect = UiRect::new(card_x, card_y, card_width, card_height);
                let is_hovered =
                    card_rect.contains_mouse() && mouse_y > content_top && mouse_y < content_bottom;

                // Only toggle expand if clicking on card but NOT on the button
                if is_hovered && mouse_clicked && entry.encountered && !is_over_button {
                    if *selected == Some(passenger.id) {
                        *selected = None; // Deselect
                    } else {
                        *selected = Some(passenger.id); // Select
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

                    // Level 3, or a story earned rather than bought.
                    //
                    // `unlocked_backstories` is set two ways that have nothing
                    // to do with lore: the ride-completion roll, and reading a
                    // passenger's guideline right, which pays a `StoryUnlock`.
                    // The almanac only ever asked about knowledge level, so a
                    // story earned in play was shown once on the drop-off
                    // summary and then vanished from the one screen built to
                    // hold what the driver knows about people. It also already
                    // counts for something invisible -- a known story raises
                    // that passenger's item drop chance by half again.
                    if entry.knowledge_level >= 3 || story_known {
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

                // What the lore actually buys.
                //
                // Every almanac level authors the list -- level 2 is "Route
                // Preferences, Common Tells, Likes/Dislikes" -- and nothing
                // displayed it, so the one screen where lore is spent showed
                // a price and the name of a tier and never what the tier
                // reveals. Deciding whether to invest was guesswork.
                if entry.encountered && entry.knowledge_level < 3 {
                    // Nothing already in hand is promised again.
                    let already: &[&str] = if story_known { &["Backstory"] } else { &[] };
                    let next_line = data
                        .almanac
                        .get_level(entry.knowledge_level + 1)
                        .and_then(|next| next.reveals_line(already));
                    if let Some(next_line) = next_line {
                        {
                            draw_wrapped_text(
                                &next_line,
                                text_x,
                                btn_y + 20.0,
                                (card_x + card_width - 230.0 - text_x).max(80.0),
                                fonts::SIZE_XS,
                                14.0,
                                colors::TEXT_MUTED,
                                1,
                            );
                        }
                    }
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
                        if draw_glass_button(btn_rect, &button_text, colors::ACCENT_GOLD, true) {
                            return UiAction::UpgradeAlmanacKnowledge(passenger.id);
                        }
                    } else {
                        draw_small_caps(
                            &button_text,
                            btn_rect.x,
                            btn_rect.y + 20.0,
                            fonts::SIZE_XS,
                            colors::TEXT_MUTED,
                        );
                    }
                }
            }

            if wide_layout && idx % 2 == 1 {
                right_y += card_height + 12.0;
            } else {
                y += card_height + 12.0;
            }
        }

        scroll.draw_scrollbar(list_view, content_height);

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

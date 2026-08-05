//! Skill tree screen: spend bank balance to unlock permanent skills.

use crate::data::GameData;
use crate::engine::RideService;
use crate::state::PlayerStats;
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_small_caps,
    draw_wrapped_text, fonts, UiAction, UiRect,
};
use crate::ui::{draw_ui_text, measure_ui_text};
use macroquad::prelude::*;
use macroquad_toolkit::ui::ScrollArea;

fn skill_category_label(category: &str, data: &GameData) -> String {
    let loc_cats = &data.localization.ui.meta.skill_tree.categories;
    match category {
        "survival" => loc_cats.survival.clone(),
        "occult" => loc_cats.occult.clone(),
        "efficiency" => loc_cats.efficiency.clone(),
        // Hardcoded like the other ~145 strings pending localization (TODO.md).
        "comfort" => "Comfort".to_string(),
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
        "comfort" => {
            // A bench seat: back rest and cushion.
            draw_line(x - 14.0, y - 14.0, x - 14.0, y + 8.0, 2.0, color);
            draw_rectangle_lines(x - 14.0, y + 2.0, 28.0, 10.0, 2.0, color);
            draw_line(x + 14.0, y + 12.0, x + 14.0, y + 16.0, 2.0, color);
            draw_line(x - 14.0, y + 12.0, x - 14.0, y + 16.0, 2.0, color);
        }
        _ => {
            draw_circle_lines(x, y, 14.0, 2.0, color);
        }
    }
}

/// For an `ability_unlock` skill, how many passengers carry its trait and how
/// many of those the player has studied. `None` for every other skill.
///
/// The choice an ability skill buys only appears for a passenger who carries
/// the matching trait *and* whom the player has studied to Almanac Lv.1. That
/// second requirement is nowhere in `prerequisites`, so the skill tree used to
/// offer Night Vision for $750 with no hint that an unstudied roster makes it
/// inert — which is most of why a skills-only run measures barely better than
/// no progression at all. The studied count is the number of times the
/// purchase can ever fire.
pub(crate) fn ability_carriers(
    skill: &crate::data::Skill,
    passengers: &[crate::data::Passenger],
    player_stats: &PlayerStats,
) -> Option<(usize, usize)> {
    if skill.effect.effect_type != "ability_unlock" {
        return None;
    }
    let carried = passengers.iter().filter(|passenger| {
        passenger
            .traits
            .iter()
            .any(|name| RideService::trait_skill_id(name) == skill.effect.target)
    });
    Some(carried.fold((0, 0), |(studied, total), passenger| {
        let known = player_stats.get_almanac_entry(passenger.id).knowledge_level >= 1;
        (studied + usize::from(known), total + 1)
    }))
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

    let mut prereq_text = if skill.prerequisites.is_empty() {
        "No prerequisite".to_string()
    } else {
        let met = skill
            .prerequisites
            .iter()
            .filter(|id| player_stats.unlocked_skills.contains(id))
            .count();
        format!("Prerequisites {}/{}", met, skill.prerequisites.len())
    };

    // An ability skill has a second requirement that is not in
    // `prerequisites` at all: the choice it buys only appears for a
    // passenger who carries the matching trait *and* whom the player has
    // studied to Almanac Lv.1. Buying one against an unstudied roster buys
    // nothing, and this screen said nothing about it — which is why a
    // skills-only run measures as barely better than no progression.
    //
    // Naming the count rather than the rule makes it actionable: five
    // passengers carry Night Vision, and the number of them you have
    // studied is the number of times this purchase can ever fire.
    let mut unmet_ability = false;
    if let Some((studied, total)) = ability_carriers(skill, &data.passengers, player_stats) {
        prereq_text = format!("{prereq_text}  |  {studied}/{total} carriers studied");
        unmet_ability = studied == 0;
    }

    draw_small_caps(
        &prereq_text,
        text_x,
        (desc_bottom + 12.0).min(rect.y + rect.h - 18.0),
        fonts::SIZE_XS,
        if unmet_ability {
            colors::ACCENT_WARNING
        } else {
            colors::TEXT_MUTED
        },
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

/// Content height (in unscrolled content-space) needed to lay out every
/// category, mirroring the accumulation the draw loop below performs.
/// Computed as a cheap pre-pass (no drawing) so the `ScrollArea` can clamp
/// and draw its scrollbar correctly before the real layout pass runs.
fn compute_content_height(
    categories: &[&str],
    data: &GameData,
    wide_layout: bool,
    card_h: f32,
    card_gap: f32,
) -> f32 {
    let mut running = 0.0_f32;
    let mut max_extent = 0.0_f32;
    for category in categories {
        let n = data
            .skills
            .iter()
            .filter(|s| s.category == *category)
            .count() as f32;
        let category_h = 58.0 + n * (card_h + card_gap) + 12.0;
        let start = if wide_layout { 0.0 } else { running };
        max_extent = max_extent.max(start + 58.0 + n * (card_h + card_gap));
        if !wide_layout {
            running += category_h + 18.0;
        }
    }
    max_extent
}

/// Draw the skill tree screen
pub fn draw_skill_tree(
    player_stats: &PlayerStats,
    game_data: Option<&GameData>,
    scroll: &mut ScrollArea,
) -> UiAction {
    draw_noir_city_background();

    if let Some(data) = game_data {
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
            header_inner.y + 22.0,
            fonts::SIZE_XS,
            colors::TEXT_MUTED,
        );

        // Sell surplus lore back for bank. Without this the two currencies
        // never meet: lore goes dead once the almanac is mastered while the
        // skill tree stays starved.
        let rate = data.rewards.lore_exchange;
        if rate.is_available() {
            let exchange_rect = UiRect::new(
                header_inner.x + header_inner.w * 0.52,
                header_inner.y + 34.0,
                236.0,
                30.0,
            );
            let label = format!("Trade {} lore -> ${}", rate.lore, rate.bank);
            let affordable = player_stats.lore_fragments >= rate.lore;
            if draw_glass_button(exchange_rect, &label, colors::ACCENT_GOLD, affordable)
                && affordable
            {
                return UiAction::ExchangeLoreForBank;
            }
            draw_small_caps(
                &format!("{} lore fragments held", player_stats.lore_fragments),
                exchange_rect.x + exchange_rect.w + 14.0,
                exchange_rect.y + 20.0,
                fonts::SIZE_XS,
                colors::TEXT_MUTED,
            );
        }

        let categories = ["survival", "occult", "efficiency", "comfort"];
        let wide_layout = screen_w >= 1200.0;
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

        let content_height =
            compute_content_height(&categories, data, wide_layout, card_h, card_gap);
        let list_view = Rect::new(margin, content_top, screen_w - margin * 2.0, content_h);
        scroll.update(list_view, content_height);
        let scroll_offset = scroll.offset();

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
        }

        scroll.draw_scrollbar(list_view, content_height);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{load_passengers, load_skill_tree};

    /// A fresh driver has studied nobody, so every ability skill reads as
    /// zero-of-something and the card warns rather than looking ready.
    #[test]
    fn a_new_driver_has_no_one_to_use_an_ability_on() {
        let passengers = load_passengers();
        let stats = PlayerStats::default();
        let mut ability_skills = 0;
        for skill in load_skill_tree() {
            let Some((studied, total)) = ability_carriers(&skill, &passengers, &stats) else {
                continue;
            };
            ability_skills += 1;
            assert_eq!(studied, 0, "{} reads as studied on a fresh save", skill.id);
            assert!(total > 0, "{} names a trait no passenger carries", skill.id);
        }
        assert!(ability_skills > 0, "no ability skills to check");
    }

    /// Studying a carrier moves that skill's count, so the line is live
    /// rather than a fixed caption.
    #[test]
    fn studying_a_carrier_moves_the_count() {
        let passengers = load_passengers();
        let skill = load_skill_tree()
            .into_iter()
            .find(|skill| skill.effect.effect_type == "ability_unlock")
            .expect("an ability skill");
        let carrier = passengers
            .iter()
            .find(|passenger| {
                passenger
                    .traits
                    .iter()
                    .any(|name| RideService::trait_skill_id(name) == skill.effect.target)
            })
            .expect("a carrier");

        let mut stats = PlayerStats::default();
        let before = ability_carriers(&skill, &passengers, &stats)
            .expect("counts")
            .0;
        stats.mark_passenger_encountered(carrier.id);
        stats.lore_fragments += 999;
        stats.upgrade_almanac_knowledge(carrier.id, 0);
        let after = ability_carriers(&skill, &passengers, &stats)
            .expect("counts")
            .0;

        assert_eq!(before + 1, after, "studying a carrier changed nothing");
    }

    /// Skills that are not ability unlocks have no carrier line at all.
    #[test]
    fn an_ordinary_skill_has_no_carrier_line() {
        let passengers = load_passengers();
        let stats = PlayerStats::default();
        let skill = load_skill_tree()
            .into_iter()
            .find(|skill| skill.effect.effect_type != "ability_unlock")
            .expect("an ordinary skill");
        assert!(ability_carriers(&skill, &passengers, &stats).is_none());
    }
}

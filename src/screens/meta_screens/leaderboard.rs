//! Leaderboard and achievements screen.

use crate::data::GameData;
use crate::state::PlayerStats;
use crate::ui::draw_ui_text;
use crate::ui::{
    colors, draw_glass_button, draw_glass_panel, draw_noir_city_background, draw_small_caps,
    draw_wrapped_text, fonts, UiAction, UiRect,
};
use macroquad::prelude::*;

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

        for achievement in player_stats.achievements.iter() {
            let unlocked = player_stats.is_achievement_unlocked(&achievement.id);
            let border = if unlocked {
                colors::FUEL_GOOD
            } else {
                colors::BORDER_DIM
            };
            // Room for the progress line on a locked card that counts toward
            // something; an unlocked one has nothing left to say.
            let counts = !unlocked
                && (player_stats.achievement_progress(&achievement.id).is_some()
                    || data
                        .rewards
                        .for_achievement(&achievement.id)
                        .describe()
                        .is_some());
            let card_h = if counts { 100.0 } else { 82.0 };
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
            let description_bottom = draw_wrapped_text(
                &achievement.description,
                card_rect.x + 18.0,
                card_rect.y + 48.0,
                card_rect.w - 36.0,
                fonts::SIZE_XS,
                14.0,
                colors::TEXT_MUTED,
                2,
            );

            // How close, for the ones that count toward something.
            //
            // Four of the six do, and the save has tracked all four all
            // along -- two of the counters, `survival_bonuses` and
            // `highest_shift_earnings`, appeared on no screen anywhere. So a
            // card stated a goal, the save knew the answer, and the player was
            // told only "Locked".
            if !unlocked {
                let progress = player_stats.achievement_progress(&achievement.id);
                let worth = data.rewards.for_achievement(&achievement.id).describe();
                // Progress and payout on one line: how close, and why bother.
                let line = match (progress, worth) {
                    (Some(progress), Some(worth)) => Some(format!("{progress}  |  {worth}")),
                    (Some(only), None) | (None, Some(only)) => Some(only),
                    (None, None) => None,
                };
                if let Some(line) = line {
                    draw_small_caps(
                        &line,
                        card_rect.x + 18.0,
                        description_bottom + 4.0,
                        fonts::SIZE_XS,
                        colors::ACCENT_GOLD,
                    );
                }
            }
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

//! The drop-off screen: ride completion summary and the trade offer modal.

use macroquad::prelude::*;

use crate::data::{GameData, Rarity};
use crate::state::GameState;
use crate::ui::{
    colors, draw_cockpit_background, draw_glass_button, draw_glass_panel, fonts, layout, spacing,
    CompletionSummary, UiAction, UiRect,
};
use macroquad_toolkit::ui::draw_ui_text;

use super::scene::draw_bottom_taxi_scene;

/// Draw the dropoff screen
pub fn draw_dropoff(game_state: &GameState, game_data: Option<&GameData>) -> UiAction {
    draw_cockpit_background();

    let scene_h = (screen_height() * 0.27).clamp(210.0, 300.0);
    let scene_rect = UiRect::new(
        70.0,
        screen_height() - scene_h - 34.0,
        screen_width() - 140.0,
        scene_h,
    );
    draw_bottom_taxi_scene(scene_rect);

    if let Some(ref completion) = game_state.last_ride_completion {
        let rect_w = (screen_width() - 140.0).min(880.0);
        let rect_h = (scene_rect.y - layout::STATUS_BAR_HEIGHT - 76.0).clamp(440.0, 520.0);
        let rect = UiRect::centered_x(
            screen_width(),
            layout::STATUS_BAR_HEIGHT + 34.0,
            rect_w,
            rect_h,
        );
        // CompletionSummary now needs game_data
        let completion_action = CompletionSummary::draw(completion, rect, game_data);

        if let Some(route) = game_state.route_history.last() {
            let route_text = format!(
                "Last leg: {:?} {:?} | fuel -{} | time -{} | {:.0}s ago",
                route.driving_phase,
                route.route_type,
                route.fuel_cost,
                route.time_cost,
                (get_time() - route.timestamp).max(0.0)
            );
            draw_ui_text(
                &route_text,
                rect.x + 16.0,
                rect.bottom() - 70.0,
                12.0,
                colors::TEXT_MUTED,
            );
        }

        // Show trade offer if available
        if let Some((ref passenger_name, ref offered_item)) = game_state.pending_trade {
            if let Some(data) = game_data {
                draw_rectangle(
                    0.0,
                    0.0,
                    screen_width(),
                    screen_height(),
                    Color::new(0.0, 0.0, 0.0, 0.50),
                );
                let trade_rect =
                    UiRect::centered_x(screen_width(), 146.0, screen_width().min(460.0), 330.0);
                draw_glass_panel(trade_rect, colors::ACCENT_SKY);

                let inner = trade_rect.inset(spacing::PADDING_MD);

                draw_ui_text(
                    &data.localization.ui.game.trade.title,
                    inner.x,
                    inner.y + 24.0,
                    fonts::SIZE_LG,
                    colors::ACCENT_SKY,
                );

                // Message
                let msg = data
                    .localization
                    .ui
                    .game
                    .trade
                    .wants_to_trade
                    .replace("{}", passenger_name);
                draw_ui_text(
                    &msg,
                    inner.x,
                    inner.y + 50.0,
                    fonts::SIZE_MD,
                    colors::TEXT_PRIMARY,
                );

                // Offered item
                let rarity_color = match offered_item.rarity {
                    Rarity::Common => colors::TEXT_MUTED,
                    Rarity::Uncommon => colors::ACCENT_SKY,
                    Rarity::Rare => Color::from_hex(0x87CEEB),
                    Rarity::Legendary => colors::ACCENT_WARNING,
                };

                let offer_text = data
                    .localization
                    .ui
                    .game
                    .trade
                    .offering
                    .replace("{}", &offered_item.name);
                draw_ui_text(
                    &offer_text,
                    inner.x,
                    inner.y + 80.0,
                    fonts::SIZE_MD,
                    rarity_color,
                );

                // Name what this passenger actually wants. `wantedItems` is
                // authored per passenger and already decides whether a trade is
                // offered at all; without showing it the player is picking
                // blind and the data only ever works behind their back.
                let wanted: &[String] = game_state
                    .current_passenger
                    .as_ref()
                    .map(|p| p.wanted_items.as_slice())
                    .unwrap_or(&[]);
                let wants_text = if wanted.is_empty() {
                    data.localization.ui.game.trade.any_item.clone()
                } else {
                    format!("They are looking for: {}", wanted.join(", "))
                };
                draw_ui_text(
                    &wants_text,
                    inner.x,
                    inner.y + 105.0,
                    fonts::SIZE_SM,
                    if wanted.is_empty() {
                        colors::TEXT_MUTED
                    } else {
                        colors::ACCENT_GOLD
                    },
                );

                // Buttons
                let btn_y = inner.y + 140.0;
                let btn_w = 200.0;
                let btn_h = 40.0;
                let center_x = screen_width() / 2.0;

                // Tradeable items, carrying their real inventory index. The
                // filter must come before the cap: taking the first three
                // items and then skipping untradeable ones hid every
                // tradeable item past index two, and left the player staring
                // at "select an item" with no buttons when the first three
                // happened to be bound or cursed.
                let tradeable: Vec<(usize, &crate::data::InventoryItem)> = game_state
                    .inventory
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.can_be_given_away())
                    .take(3)
                    .collect();

                if !tradeable.is_empty() {
                    draw_ui_text(
                        &data.localization.ui.game.trade.select,
                        inner.x,
                        btn_y - 20.0,
                        fonts::SIZE_SM,
                        colors::TEXT_PRIMARY,
                    );

                    for (slot, (index, item)) in tradeable.iter().enumerate() {
                        let is_wanted = wanted.contains(&item.name);
                        let label = if is_wanted {
                            format!("{} (wanted)", item.name)
                        } else {
                            item.name.clone()
                        };
                        let item_btn_y = btn_y + (slot as f32 * 45.0);
                        if draw_glass_button(
                            UiRect::new(center_x - btn_w / 2.0, item_btn_y, btn_w, 35.0),
                            &label,
                            if is_wanted {
                                colors::ACCENT_GOLD
                            } else {
                                colors::ACCENT_SKY
                            },
                            true,
                        ) {
                            return UiAction::AcceptTrade(*index);
                        }
                    }

                    let decline_y = btn_y + (150.0);
                    if draw_glass_button(
                        UiRect::new(center_x - btn_w / 2.0, decline_y, btn_w, btn_h),
                        &data.localization.ui.game.trade.decline,
                        colors::ACCENT_DANGER,
                        true,
                    ) {
                        return UiAction::DeclineTrade;
                    }
                } else {
                    // Nothing tradeable to offer — inventory may still hold
                    // bound items like the Blessed Medallion.
                    draw_ui_text(
                        &data.localization.ui.game.trade.empty,
                        inner.x,
                        btn_y,
                        fonts::SIZE_SM,
                        colors::ACCENT_WARNING,
                    );
                    if draw_glass_button(
                        UiRect::new(center_x - btn_w / 2.0, btn_y + 30.0, btn_w, btn_h),
                        &data.localization.ui.common.continue_text,
                        colors::ACCENT_SKY,
                        true,
                    ) {
                        return UiAction::DeclineTrade;
                    }
                }
            }
        } else {
            return completion_action;
        }
    }
    UiAction::None
}

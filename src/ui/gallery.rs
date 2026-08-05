//! Deterministic verification surfaces for the shared UI vocabulary.
//!
//! These are capture-only scenes, not player-facing screens. Keeping them in
//! the UI module means a component change can be inspected by itself at every
//! supported viewport instead of hoping a convenient gameplay state happens
//! to exercise it.

use super::*;
use crate::data::GameData;
use crate::state::{GameState, RideCompletion, RideImpact};
use macroquad::prelude::*;

/// Draw one named component verification scene.
pub fn draw_component_gallery(scene: &str, state: &GameState, data: Option<&GameData>) -> UiAction {
    draw_noir_city_background();
    match scene {
        "ui_core" => draw_core_vocabulary(),
        "ui_status" => draw_status_sample(state, data),
        "ui_passenger" => draw_passenger_sample(data),
        "ui_completion" => draw_completion_sample(data),
        _ => UiAction::None,
    }
}

fn gallery_heading(title: &str, subtitle: &str) {
    draw_ui_text(title, 34.0, 42.0, fonts::SIZE_XXL, colors::CAB_YELLOW);
    draw_small_caps(subtitle, 36.0, 68.0, fonts::SIZE_SM, colors::TEXT_SECONDARY);
}

fn draw_core_vocabulary() -> UiAction {
    gallery_heading(
        "SHARED UI VOCABULARY",
        "Type, semantic color, controls, state, icon, meter, badge and tooltip",
    );
    let margin = 34.0;
    let top = 92.0;
    let gap = 18.0;
    let left_w = ((screen_width() - margin * 2.0 - gap) * 0.48).max(360.0);
    let left = UiRect::new(margin, top, left_w, screen_height() - top - 28.0);
    let right = UiRect::new(
        left.right() + gap,
        top,
        screen_width() - left.right() - gap - margin,
        left.h,
    );
    draw_glass_panel(left, colors::BORDER);
    draw_glass_panel(right, colors::BORDER);

    let li = left.inset(18.0);
    draw_small_caps(
        "TYPE + SEMANTIC COLOR",
        li.x,
        li.y + 16.0,
        fonts::SIZE_SM,
        colors::TEXT_MUTED,
    );
    let semantic = [
        ("SAFE / PROTECTED", colors::FUEL_GOOD),
        ("WARNING / URGENT", colors::ACCENT_WARNING),
        ("DANGER / VIOLATION", colors::ACCENT_DANGER),
        ("OCCULT / INFERRED", colors::ACCENT_PRIMARY),
        ("MONEY / REWARD", colors::ACCENT_GOLD),
        ("UNKNOWN / MUTED", colors::TEXT_MUTED),
    ];
    let mut y = li.y + 48.0;
    for (label, color) in semantic {
        draw_rectangle(li.x, y - 13.0, 12.0, 12.0, color);
        draw_ui_text(label, li.x + 22.0, y, fonts::SIZE_SM, color);
        y += 27.0;
    }

    draw_small_caps(
        "BUTTON STATES",
        li.x,
        y + 6.0,
        fonts::SIZE_SM,
        colors::TEXT_MUTED,
    );
    y += 24.0;
    let button_w = (li.w - 10.0) / 2.0;
    let states = [
        ("DEFAULT", ButtonPreviewState::Default),
        ("HOVER", ButtonPreviewState::Hovered),
        ("FOCUS", ButtonPreviewState::Focused),
        ("DISABLED", ButtonPreviewState::Disabled),
        ("SELECTED", ButtonPreviewState::Selected),
        ("URGENT", ButtonPreviewState::Urgent),
    ];
    for (index, (label, state)) in states.into_iter().enumerate() {
        let col = index % 2;
        let row = index / 2;
        draw_glass_button_preview(
            UiRect::new(
                li.x + col as f32 * (button_w + 10.0),
                y + row as f32 * 48.0,
                button_w,
                38.0,
            ),
            label,
            state,
        );
    }
    y += 154.0;
    draw_small_caps(
        "METER + BADGES",
        li.x,
        y,
        fonts::SIZE_SM,
        colors::TEXT_MUTED,
    );
    draw_ui_meter(
        UiRect::new(li.x, y + 14.0, li.w, 18.0),
        0.68,
        colors::ACCENT_WARNING,
        "NEED 68%",
    );
    draw_ui_badge(
        UiRect::new(li.x, y + 44.0, 92.0, 28.0),
        "KNOWN",
        colors::FUEL_GOOD,
    );
    draw_ui_badge(
        UiRect::new(li.x + 102.0, y + 44.0, 104.0, 28.0),
        "INFERRED",
        colors::ACCENT_PRIMARY,
    );
    draw_ui_badge(
        UiRect::new(li.x + 216.0, y + 44.0, 102.0, 28.0),
        "UNKNOWN",
        colors::TEXT_MUTED,
    );

    let ri = right.inset(18.0);
    draw_small_caps(
        "DRAWN ICON ATLAS",
        ri.x,
        ri.y + 16.0,
        fonts::SIZE_SM,
        colors::TEXT_MUTED,
    );
    let icons = [
        (UiIcon::Fuel, "FUEL"),
        (UiIcon::Time, "TIME"),
        (UiIcon::Fare, "FARE"),
        (UiIcon::Risk, "RISK"),
        (UiIcon::Weather, "WEATHER"),
        (UiIcon::Rules, "RULES"),
        (UiIcon::Inventory, "ITEMS"),
        (UiIcon::Wards, "WARDS"),
        (UiIcon::Lore, "LORE"),
        (UiIcon::Cab, "CAB"),
    ];
    let icon_cols = if right.w >= 440.0 { 2 } else { 1 };
    let icon_w = ri.w / icon_cols as f32;
    for (index, (icon, label)) in icons.into_iter().enumerate() {
        let col = index % icon_cols;
        let row = index / icon_cols;
        let x = ri.x + col as f32 * icon_w;
        let iy = ri.y + 55.0 + row as f32 * 44.0;
        draw_ui_icon(icon, x + 16.0, iy, 24.0, colors::TEXT_PRIMARY);
        draw_small_caps(
            label,
            x + 36.0,
            iy + 5.0,
            fonts::SIZE_XS,
            colors::TEXT_SECONDARY,
        );
    }
    let tooltip_y = ri.y + if icon_cols == 2 { 292.0 } else { 490.0 };
    draw_ui_tooltip(
        UiRect::new(ri.x, tooltip_y, ri.w, 72.0),
        "QUOTE FLOOR",
        "Fuel is charged from the displayed ceiling; the fare cannot fall below this amount.",
    );
    UiAction::None
}

fn draw_status_sample(state: &GameState, data: Option<&GameData>) -> UiAction {
    let action = if let Some(data) = data {
        StatusBar::draw(state, &data.constants, Some(data))
    } else {
        UiAction::None
    };
    draw_ui_text(
        "STATUS BAR",
        34.0,
        122.0,
        fonts::SIZE_XXL,
        colors::CAB_YELLOW,
    );
    draw_small_caps(
        "Critical resources, weather, protection and persistent actions",
        36.0,
        148.0,
        fonts::SIZE_SM,
        colors::TEXT_SECONDARY,
    );
    action
}

fn draw_passenger_sample(data: Option<&GameData>) -> UiAction {
    gallery_heading(
        "PASSENGER CARD",
        "Portrait-led dossier with route, rarity, fare and voice",
    );
    if let Some(passenger) = data.and_then(|data| data.passengers.first()) {
        let w = (screen_width() * 0.42).clamp(360.0, 560.0);
        let rect = UiRect::new((screen_width() - w) / 2.0, 92.0, w, screen_height() - 122.0);
        PassengerCard::draw(passenger, rect, passenger.dialogue.first());
    }
    UiAction::None
}

fn draw_completion_sample(data: Option<&GameData>) -> UiAction {
    gallery_heading(
        "COMPLETION SUMMARY",
        "Itemized consequences and explicitly prevented harm",
    );
    if let Some(passenger) = data.and_then(|data| data.passengers.first()).cloned() {
        let completion = RideCompletion {
            passenger,
            fare_earned: 86,
            items_received: Vec::new(),
            backstory_unlocked: Some((
                "Case file".to_string(),
                "A final detail surfaced because the driver listened instead of merely surviving."
                    .to_string(),
            )),
            impact: RideImpact {
                fuel_spent: 9,
                time_spent: 27,
                need_delta: -14,
                rules_violated: 0,
                comfort_relief: 8,
                normal_route_relief: 6,
                ward_interventions: 1,
                brink_saves: 0,
            },
        };
        let rect = UiRect::new(34.0, 92.0, screen_width() - 68.0, screen_height() - 122.0);
        CompletionSummary::draw(&completion, rect, data);
    }
    UiAction::None
}

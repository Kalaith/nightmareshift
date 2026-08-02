use macroquad::prelude::*;

use super::Game;
use crate::data::WeatherType;
use crate::engine::{
    draw_danger_overlay, draw_fog_overlay, draw_glitch_effect, draw_tension_vignette, Overlay,
};
use crate::screens::{game_screens, menu_screens, meta_screens, Screen};
use crate::ui::StatusBar;
use crate::ui::*;
use macroquad_toolkit::ui::{draw_ui_text, format_clock};

impl Game {
    pub fn draw(&mut self) -> UiAction {
        // The shake displaces visuals only: hit-testing stays in unshaken
        // screen space, so buttons drift up to max_offset px for the shake's
        // duration. Zoom mirrors VirtualUi::camera() — positive Y, no
        // viewport — which reproduces macroquad's default screen mapping.
        let shake_offset = self.screen_shake.offset();
        let shaking = shake_offset != Vec2::ZERO;
        if shaking {
            set_camera(&Camera2D {
                target: vec2(
                    screen_width() * 0.5 - shake_offset.x,
                    screen_height() * 0.5 - shake_offset.y,
                ),
                zoom: vec2(2.0 / screen_width(), 2.0 / screen_height()),
                ..Default::default()
            });
        }
        let action = self.draw_frame();
        if shaking {
            set_default_camera();
        }
        action
    }

    fn draw_frame(&mut self) -> UiAction {
        clear_background(Color::from_hex(0x1a1a2e));

        let action = match self.screen {
            Screen::Loading => menu_screens::draw_loading(self.game_data.as_ref()),
            Screen::MainMenu => menu_screens::draw_main_menu(
                &self.player_stats,
                self.game_data.as_ref(),
                self.delete_armed_until.is_some(),
            ),
            Screen::Briefing => menu_screens::draw_briefing(
                &self.game_state,
                self.game_data.as_ref(),
                &self.player_stats,
            ),
            Screen::Game => self.draw_game_phase(),
            Screen::GameOver => {
                menu_screens::draw_game_over(&self.game_state, self.game_data.as_ref())
            }
            Screen::Success => {
                menu_screens::draw_success(&self.game_state, self.game_data.as_ref())
            }
            Screen::SkillTree => meta_screens::draw_skill_tree(
                &self.player_stats,
                self.game_data.as_ref(),
                &mut self.skill_tree_scroll,
            ),
            Screen::Almanac => meta_screens::draw_almanac(
                &self.player_stats,
                self.game_data.as_ref(),
                &mut self.almanac_scroll,
                &mut self.almanac_selected,
            ),
            Screen::Leaderboard => {
                meta_screens::draw_leaderboard(&self.player_stats, self.game_data.as_ref())
            }
        };

        if self.screen == Screen::Game {
            let game_data_ref = self.game_data.as_ref();
            if self.show_rules {
                let rules_action = game_screens::draw_rules_panel(&self.game_state, game_data_ref);
                if rules_action != UiAction::None {
                    return rules_action;
                }
            }
            if self.show_inventory {
                let inventory_action =
                    game_screens::draw_inventory_modal(&self.game_state, game_data_ref);
                if inventory_action != UiAction::None {
                    return inventory_action;
                }
            }
        }

        self.particles.draw();

        if self.screen == Screen::Game {
            if self.game_state.current_weather.weather_type == WeatherType::Fog {
                draw_fog_overlay(0.12);
            }

            let route_danger = self
                .game_state
                .route_history
                .iter()
                .rev()
                .take(3)
                .map(|route| route.risk_level as f32)
                .sum::<f32>()
                / 15.0;

            let passenger_danger = self
                .game_state
                .current_passenger_need_state
                .as_ref()
                .map(|need_state| (1.0 - need_state.stability) * 0.5)
                .unwrap_or(0.0);

            let total_danger = (route_danger + passenger_danger).clamp(0.0, 1.0);
            if total_danger > 0.1 {
                draw_danger_overlay(total_danger);
            }

            let tension = self
                .game_state
                .current_passenger_need_state
                .as_ref()
                .map(|need_state| need_state.level as f32 / 100.0)
                .unwrap_or(0.0);

            if tension > 0.3 {
                draw_tension_vignette((tension - 0.3) * 1.5);
            }
        }

        if self.screen == Screen::GameOver {
            let glitch_intensity = (get_time() % 2.0) as f32 / 2.0;
            draw_glitch_effect(glitch_intensity * 0.5);
        }

        if self.show_pause_menu && self.screen == Screen::Game {
            if let Some(pause_action) = self.draw_pause_menu() {
                return pause_action;
            }
        }

        self.transition.draw();

        // An open modal swallows the screen beneath it: the underlying
        // screen still drew (and computed) its action, but dispatching it
        // would let a click behind the rules panel pick a route or refuel.
        if self.active_overlay() != Overlay::None {
            return UiAction::None;
        }

        action
    }

    fn draw_game_phase(&self) -> UiAction {
        let phase_action = game_screens::draw_game(
            &self.game_state,
            self.game_data.as_ref(),
            &self.player_stats,
        );

        let status_action = if let Some(ref data) = self.game_data {
            StatusBar::draw(&self.game_state, &data.constants, self.game_data.as_ref())
        } else {
            UiAction::None
        };

        if status_action != UiAction::None {
            return status_action;
        }
        phase_action
    }

    fn draw_pause_menu(&self) -> Option<UiAction> {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.72),
        );
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.02, 0.025, 0.030, 0.18),
        );

        let panel = UiRect::centered_x(
            screen_width(),
            (screen_height() - 400.0) / 2.0,
            screen_width().min(520.0),
            400.0,
        );
        draw_glass_panel(panel, colors::BORDER);
        let inner = panel.inset(spacing::PADDING_LG);

        draw_ui_text(
            "PAUSED",
            inner.x,
            inner.y + 36.0,
            fonts::SIZE_XXL,
            colors::CAB_YELLOW,
        );
        draw_small_caps(
            "Shift suspended. The meter is still waiting.",
            inner.x,
            inner.y + 66.0,
            fonts::SIZE_SM,
            colors::TEXT_MUTED,
        );

        let stat_y = inner.y + 104.0;
        let stat_gap = 10.0;
        let stat_w = (inner.w - stat_gap * 2.0) / 3.0;
        let mins = self.game_state.time_remaining % 60;
        let hours = self.game_state.time_remaining / 60;
        let stats = [
            (
                "Fuel",
                format!("{:.0}%", self.game_state.fuel),
                self.game_data
                    .as_ref()
                    .map(|data| get_fuel_color(self.game_state.fuel, &data.constants.fuel))
                    .unwrap_or(colors::TEXT_PRIMARY),
            ),
            (
                "Earned",
                format!("${}", self.game_state.earnings),
                colors::ACCENT_GOLD,
            ),
            ("Time", format_clock(hours, mins), colors::TEXT_SECONDARY),
        ];
        for (idx, (label, value, color)) in stats.iter().enumerate() {
            let rect = UiRect::new(
                inner.x + idx as f32 * (stat_w + stat_gap),
                stat_y,
                stat_w,
                72.0,
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
            draw_ui_text(value, rect.x + 14.0, rect.y + 30.0, fonts::SIZE_LG, *color);
            draw_small_caps(
                label,
                rect.x + 14.0,
                rect.y + 54.0,
                fonts::SIZE_XS,
                colors::TEXT_MUTED,
            );
        }

        let action_y = stat_y + 104.0;
        if draw_glass_button(
            UiRect::new(inner.x, action_y, inner.w, 48.0),
            "Resume (ESC)",
            colors::CAB_YELLOW,
            true,
        ) {
            return Some(UiAction::TogglePauseMenu);
        }

        // Walking out mid-shift calls `return_to_menu` and nothing else: no
        // `end_shift`, so the night pays no bank, no lore, no leaderboard
        // entry and no stats. That is a defensible way for abandoning a run
        // to work, but the button said only "Return to Menu" and the player
        // had no way to know what it cost.
        if draw_glass_button(
            UiRect::new(inner.x, action_y + 62.0, inner.w, 48.0),
            "Abandon Night",
            colors::ACCENT_DANGER,
            true,
        ) {
            return Some(UiAction::ReturnToMenu);
        }

        let forfeit = if self.game_state.earnings > 0 {
            format!(
                "Forfeits ${} and everything this night would have banked.",
                self.game_state.earnings
            )
        } else {
            "Nothing earned yet. The night is forfeit either way.".to_string()
        };
        draw_small_caps(
            &forfeit,
            inner.x,
            action_y + 132.0,
            fonts::SIZE_XS,
            colors::TEXT_MUTED,
        );

        draw_rectangle(
            panel.x,
            panel.bottom() - 1.0,
            panel.w,
            1.0,
            Color::new(0.95, 0.58, 0.08, 0.55),
        );

        None
    }
}

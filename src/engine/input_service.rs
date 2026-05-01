//! Input service for mapping user input to game actions.

use crate::screens::Screen;
use crate::state::GamePhase;
use crate::ui::UiAction;
use macroquad::prelude::*; // GamePhase is in state, need to ensure imports are correct in mod.rs

/// Input service structure
pub struct InputService;

impl InputService {
    /// Capture input and return a list of triggered UI actions
    pub fn capture_input(screen: Screen, game_phase: GamePhase) -> Vec<UiAction> {
        let mut actions = Vec::new();

        match screen {
            Screen::MainMenu => {
                if is_key_pressed(KeyCode::Space) {
                    actions.push(UiAction::StartGame);
                }
            }
            Screen::Briefing => {
                if is_key_pressed(KeyCode::Space) {
                    actions.push(UiAction::StartGame); // Mapped to StartShift in context
                }
            }
            Screen::Game => {
                // Global Game keys
                if is_key_pressed(KeyCode::R) {
                    actions.push(UiAction::ToggleRules);
                }
                if is_key_pressed(KeyCode::I) {
                    actions.push(UiAction::ToggleInventory);
                }
                // ESC for pause menu (except during RideRequest where ESC declines)
                if is_key_pressed(KeyCode::Escape) && game_phase != GamePhase::RideRequest {
                    actions.push(UiAction::TogglePauseMenu);
                }

                // Phase specific input
                match game_phase {
                    GamePhase::Waiting => {
                        if is_key_pressed(KeyCode::Space) {
                            actions.push(UiAction::Continue); // Spawn passenger
                        }
                    }
                    GamePhase::RideRequest => {
                        if is_key_pressed(KeyCode::Space) {
                            actions.push(UiAction::AcceptRide);
                        }
                        if is_key_pressed(KeyCode::Escape) {
                            actions.push(UiAction::DeclineRide);
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    GamePhase::Driving => {
                        if is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::Kp1) {
                            actions.push(UiAction::SelectRoute(0)); // Normal
                        }
                        if is_key_pressed(KeyCode::Key2) || is_key_pressed(KeyCode::Kp2) {
                            actions.push(UiAction::SelectRoute(1)); // Shortcut
                        }
                        if is_key_pressed(KeyCode::Key3) || is_key_pressed(KeyCode::Kp3) {
                            actions.push(UiAction::SelectRoute(2)); // Scenic
                        }
                        if is_key_pressed(KeyCode::Key4) || is_key_pressed(KeyCode::Kp4) {
                            actions.push(UiAction::SelectRoute(3)); // Police
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    GamePhase::Interaction => {
                        if is_key_pressed(KeyCode::Space) {
                            actions.push(UiAction::Continue);
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    GamePhase::DropOff => {
                        if is_key_pressed(KeyCode::Space) {
                            actions.push(UiAction::Continue);
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    // Guideline decisions are usually button clicks, but we could map keys here too
                    _ => {}
                }
            }
            Screen::GameOver | Screen::Success => {
                if is_key_pressed(KeyCode::Space) {
                    // TryAgain or ReturnToMenu is handled by context, but usually Space on generic screens means "Proceed"
                    // In handle_input previously: Screen::GameOver | Screen::Success -> return_to_menu (UiAction::ReturnToMenu)
                    // But draw_game_over uses UiAction::TryAgain.
                    // Let's settle on ReturnToMenu for Space, matching previous handle_input logic
                    actions.push(UiAction::ReturnToMenu);
                }
            }
            Screen::SkillTree | Screen::Almanac | Screen::Leaderboard => {
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::ReturnToMenu);
                }
            }
            _ => {}
        }

        actions
    }

    fn capture_cab_controls(actions: &mut Vec<UiAction>) {
        if is_key_pressed(KeyCode::E) {
            actions.push(UiAction::PerformRuleAction("eye_contact".to_string()));
        }
        if is_key_pressed(KeyCode::M) {
            actions.push(UiAction::PerformRuleAction("play_music".to_string()));
        }
        if is_key_pressed(KeyCode::T) {
            actions.push(UiAction::PerformRuleAction("accept_tip".to_string()));
        }
        if is_key_pressed(KeyCode::W) {
            actions.push(UiAction::PerformRuleAction("open_window".to_string()));
        }
        if is_key_pressed(KeyCode::Y) {
            actions.push(UiAction::PerformRuleAction("use_wipers".to_string()));
        }
        if is_key_pressed(KeyCode::H) {
            actions.push(UiAction::PerformRuleAction("drive_dark".to_string()));
        }
        if is_key_pressed(KeyCode::A) {
            actions.push(UiAction::PerformRuleAction("use_ac".to_string()));
        }
        if is_key_pressed(KeyCode::S) {
            actions.push(UiAction::PerformRuleAction("stop_vehicle".to_string()));
        }
    }
}

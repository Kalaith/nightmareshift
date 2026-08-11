//! Input service for mapping user input to game actions.

use crate::screens::Screen;
use crate::state::GamePhase;
use crate::ui::UiAction;
use macroquad::prelude::*; // GamePhase is in state, need to ensure imports are correct in mod.rs

/// Which modal overlay is eating input this frame. While one is open, the
/// only keys read are the ones that dismiss it (or trade it for the pause
/// menu) — gameplay keys must not act on a screen the player cannot see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    /// The rules panel or the inventory modal.
    Panel,
    Pause,
}

/// Input service structure
pub struct InputService;

impl InputService {
    /// Capture input and return a list of triggered UI actions
    pub fn capture_input(screen: Screen, game_phase: GamePhase, overlay: Overlay) -> Vec<UiAction> {
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
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::ReturnToMenu);
                }
            }
            Screen::Game if overlay == Overlay::Pause => {
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::TogglePauseMenu);
                }
            }
            Screen::Game if overlay == Overlay::Panel => {
                if is_key_pressed(KeyCode::R) {
                    actions.push(UiAction::ToggleRules);
                }
                if is_key_pressed(KeyCode::I) {
                    actions.push(UiAction::ToggleInventory);
                }
                // ESC over a panel opens the pause menu, which closes the
                // panels.
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::TogglePauseMenu);
                }
                // The rules panel draws the cab controls as key-labelled
                // buttons, so the keys it advertises keep working under it.
                Self::capture_cab_controls(&mut actions);
            }
            Screen::Game => {
                // Global Game keys
                if is_key_pressed(KeyCode::R) {
                    actions.push(UiAction::ToggleRules);
                }
                if is_key_pressed(KeyCode::I) {
                    actions.push(UiAction::ToggleInventory);
                }
                // ESC pauses in every phase. It used to decline during
                // RideRequest, leaving that the one phase the shift could
                // not be paused from.
                if is_key_pressed(KeyCode::Escape) {
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
                        // D matches the button label; ESC pauses now.
                        if is_key_pressed(KeyCode::D) {
                            actions.push(UiAction::DeclineRide);
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    GamePhase::Driving => {
                        for (index, key) in Self::number_keys().enumerate() {
                            if is_key_pressed(key.0) || is_key_pressed(key.1) {
                                actions.push(UiAction::SelectRoute(index));
                            }
                        }
                        Self::capture_cab_controls(&mut actions);
                    }
                    GamePhase::Interaction => {
                        // The mid-ride event draws its options as [1], [2],
                        // [3] and read no number keys, so the brackets were a
                        // promise the screen could not keep. The same digits
                        // pick a route one phase earlier, which is where a
                        // player would have learned to expect them.
                        for (index, key) in Self::number_keys().enumerate() {
                            if is_key_pressed(key.0) || is_key_pressed(key.1) {
                                actions.push(UiAction::SelectEventChoice(index));
                            }
                        }
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
                    // The guideline decision is the one timed choice in the
                    // game and was the only phase with no keys at all, so a
                    // thirty-second deadline had to be met with the mouse.
                    GamePhase::GuidelineDecision => {
                        if is_key_pressed(KeyCode::F) {
                            actions.push(UiAction::FollowGuideline);
                        }
                        if is_key_pressed(KeyCode::B) {
                            actions.push(UiAction::BreakGuideline);
                        }
                        // The rules panel's cab-action buttons stay live in
                        // this phase, so the keys they advertise do too.
                        Self::capture_cab_controls(&mut actions);
                    }
                    _ => {}
                }
            }
            Screen::GameOver | Screen::Success => {
                // These screens label their buttons "Try Again (SPACE)",
                // "Next Night (SPACE)" and "Back to Menu (ESC)". Space used to
                // emit `ReturnToMenu`, so on a lost shift it went to the menu
                // while the button under the player's cursor promised another
                // attempt, and Escape was not read at all despite being
                // advertised. `TryAgain` resolves to the right thing for an
                // interim night or a finished run.
                if is_key_pressed(KeyCode::Space) {
                    actions.push(UiAction::TryAgain);
                }
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::ReturnToMenu);
                }
            }
            Screen::SkillTree | Screen::Almanac | Screen::Leaderboard
                if is_key_pressed(KeyCode::Escape) =>
            {
                actions.push(UiAction::ReturnToMenu);
            }
            Screen::HelpOptions => {
                if is_key_pressed(KeyCode::Escape) {
                    actions.push(UiAction::ReturnToMenu);
                }
                if is_key_pressed(KeyCode::T) {
                    actions.push(UiAction::CycleTextScale);
                }
                if is_key_pressed(KeyCode::H) {
                    actions.push(UiAction::ToggleHighContrast);
                }
                if is_key_pressed(KeyCode::R) {
                    actions.push(UiAction::ToggleReducedMotion);
                }
                if is_key_pressed(KeyCode::B) {
                    actions.push(UiAction::CycleBrightness);
                }
                if is_key_pressed(KeyCode::C) {
                    actions.push(UiAction::ToggleCaptions);
                }
                if is_key_pressed(KeyCode::F) {
                    actions.push(UiAction::ToggleFullscreen);
                }
                for (action, (top, keypad)) in [
                    UiAction::CycleMasterVolume,
                    UiAction::CycleAmbienceVolume,
                    UiAction::CycleMusicVolume,
                    UiAction::CycleEffectsVolume,
                ]
                .into_iter()
                .zip(Self::number_keys())
                {
                    if is_key_pressed(top) || is_key_pressed(keypad) {
                        actions.push(action);
                    }
                }
            }
            _ => {}
        }

        actions
    }

    /// The digits 1-4, top row and keypad, in the order the screens number
    /// their options.
    fn number_keys() -> impl Iterator<Item = (KeyCode, KeyCode)> {
        [
            (KeyCode::Key1, KeyCode::Kp1),
            (KeyCode::Key2, KeyCode::Kp2),
            (KeyCode::Key3, KeyCode::Kp3),
            (KeyCode::Key4, KeyCode::Kp4),
        ]
        .into_iter()
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

#[cfg(test)]
mod tests;

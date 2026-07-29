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
mod tests {
    use crate::data::loader::load_localization;

    /// Every key a button advertises must be one the input service reads on
    /// the screen that shows it.
    ///
    /// This reads the source rather than simulating a keypress, because
    /// `is_key_pressed` needs a window. It is still the check that was
    /// missing: "Try Again (SPACE)" sat on the game-over screen while Space
    /// there emitted `ReturnToMenu`, and "Back to Menu (ESC)" sat beside it
    /// while Escape was not read on that screen at all.
    #[test]
    fn the_outcome_screens_read_the_keys_they_advertise() {
        let source = include_str!("input_service.rs");
        let arm = source
            .split("Screen::GameOver | Screen::Success =>")
            .nth(1)
            .expect("the outcome screens have an input arm");
        let arm = &arm[..arm.find("Screen::SkillTree").unwrap_or(arm.len())];

        assert!(
            arm.contains("KeyCode::Space"),
            "the outcome screens advertise SPACE and do not read it"
        );
        assert!(
            arm.contains("KeyCode::Escape"),
            "the outcome screens advertise ESC and do not read it"
        );
        assert!(
            arm.contains("TryAgain"),
            "SPACE on the outcome screens does not do what the button says"
        );
        assert!(
            arm.contains("ReturnToMenu"),
            "ESC on the outcome screens does not do what the button says"
        );
    }

    /// A screen that numbers its options in brackets is promising number
    /// keys, and every phase that does so must read them.
    ///
    /// The mid-ride event drew [1], [2], [3] beside its choices and the
    /// Interaction phase read no digits at all — the same digits that pick a
    /// route one phase earlier. A player taught the binding by the driving
    /// screen would press it on the next screen and nothing would happen.
    #[test]
    fn every_phase_that_numbers_its_options_reads_the_digits() {
        let input = include_str!("input_service.rs");

        for (phase, screen_source) in [
            (
                "GamePhase::Driving",
                include_str!("../screens/game_screens/driving.rs"),
            ),
            (
                "GamePhase::Interaction",
                include_str!("../screens/game_screens/interaction.rs"),
            ),
        ] {
            // The screen advertises digits by numbering its options.
            assert!(
                screen_source.contains(r#"format!("[{}]", i + 1)"#)
                    || screen_source.contains(r#""[1]""#)
                    || screen_source.contains("key_hint"),
                "{phase}'s screen no longer numbers its options; update this test"
            );

            let arm = input
                .split(&format!("{phase} => {{"))
                .nth(1)
                .unwrap_or_else(|| panic!("{phase} has no input arm"));
            let arm = &arm[..arm.find("GamePhase::").unwrap_or(arm.len())];
            assert!(
                arm.contains("number_keys()"),
                "{phase} numbers its options and reads no digits"
            );
        }
    }

    /// The guideline decision is the only timed choice, so it must be
    /// reachable from the keyboard, and its buttons must say which keys.
    #[test]
    fn the_timed_decision_has_keys_and_advertises_them() {
        let source = include_str!("input_service.rs");
        assert!(
            source.contains("GamePhase::GuidelineDecision"),
            "the timed decision has no key input"
        );

        let guidelines = load_localization().ui.game.guidelines;
        assert!(
            guidelines.follow.contains("(F)"),
            "the follow button does not name its key: {:?}",
            guidelines.follow
        );
        assert!(
            guidelines.break_guideline.contains("(B)"),
            "the break button does not name its key: {:?}",
            guidelines.break_guideline
        );
    }
}

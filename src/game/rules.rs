use macroquad::prelude::get_time;
use macroquad_toolkit::rng;

use super::Game;
use crate::data::{Consequence, ConsequenceType, RouteType};
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;

impl Game {
    pub(super) fn perform_rule_action(&mut self, action_key: String) {
        if !self.can_perform_cab_action(&action_key) {
            self.game_state.current_dialogue = Some(CurrentDialogue {
                text: format!(
                    "{} is not useful right now.",
                    Self::cab_action_label(&action_key)
                ),
                speaker: DialogueSpeaker::Driver,
                timestamp: get_time(),
            });
            return;
        }

        let current_time = get_time();
        let visible = GameEngine::check_rule_violation(
            &self.game_state.current_rules,
            &action_key,
            self.game_state.current_passenger.as_ref(),
            self.game_state.current_passenger_need_state.as_ref(),
        );

        if visible.violation || visible.rule.is_some() {
            self.resolve_cab_rule_action(visible, false, &action_key, current_time);
            return;
        }

        let hidden = GameEngine::check_rule_violation(
            &self.game_state.hidden_rules,
            &action_key,
            self.game_state.current_passenger.as_ref(),
            self.game_state.current_passenger_need_state.as_ref(),
        );

        if hidden.violation || hidden.rule.is_some() {
            self.resolve_cab_rule_action(hidden, true, &action_key, current_time);
            return;
        }

        self.game_state.adjust_player_trust(0.01);
        self.game_state.current_dialogue = Some(CurrentDialogue {
            text: format!(
                "You {}. Nothing answers.",
                Self::cab_action_phrase(&action_key)
            ),
            speaker: DialogueSpeaker::Driver,
            timestamp: current_time,
        });
    }

    fn resolve_cab_rule_action(
        &mut self,
        result: RuleEvaluationResult,
        hidden: bool,
        action_key: &str,
        current_time: f64,
    ) {
        if hidden {
            if let Some(rule) = &result.rule {
                self.game_state.reveal_hidden_rule(rule.id);
            }
        }

        self.apply_rule_need_adjustment(&result, current_time);

        let rule_title = result
            .rule
            .as_ref()
            .map(|rule| rule.title.clone())
            .unwrap_or_else(|| "Rule".to_string());

        if result.violation {
            self.game_state.rules_violated += 1;
            self.game_state.adjust_player_trust(-0.1);

            let message = result
                .message
                .clone()
                .unwrap_or_else(|| "You violated a rule.".to_string());

            if self.game_state.rule_immunity_charges > 0 {
                self.game_state.rule_immunity_charges -= 1;
                self.game_state.current_dialogue = Some(CurrentDialogue {
                    text: format!("A ward absorbs the {} violation. {}", rule_title, message),
                    speaker: DialogueSpeaker::Narrator,
                    timestamp: current_time,
                });
                return;
            }

            if self.game_state.rides_completed == 0 {
                self.game_state.current_dialogue = Some(CurrentDialogue {
                    text: if hidden {
                        format!("Hidden rule revealed: {}. {}", rule_title, message)
                    } else {
                        format!("Rule pressure spikes: {}. {}", rule_title, message)
                    },
                    speaker: DialogueSpeaker::Narrator,
                    timestamp: current_time,
                });
                return;
            }

            if let Some(rule) = &result.rule {
                self.apply_rule_consequences(&rule.break_consequences, current_time);
            }

            self.game_state.game_over_reason = Some(message);
            self.end_shift(false);
        } else {
            self.game_state.adjust_player_trust(0.05);
            if let Some(rule) = &result.rule {
                self.apply_rule_consequences(&rule.exception_rewards, current_time);
            }
            self.game_state.current_dialogue = Some(CurrentDialogue {
                text: format!(
                    "{} was dangerous on paper, but the passenger needed it.",
                    Self::cab_action_label(action_key)
                ),
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
        }
    }

    fn apply_rule_need_adjustment(&mut self, result: &RuleEvaluationResult, current_time: f64) {
        if let (Some(mut need_state), Some(passenger)) = (
            self.game_state.current_passenger_need_state.clone(),
            self.game_state.current_passenger.clone(),
        ) {
            let triggered = PassengerStateMachine::apply_rule_outcome(
                &mut need_state,
                &passenger,
                result,
                current_time,
            );
            self.game_state.current_passenger_need_state = Some(need_state);
            PassengerStateMachine::merge_detected_tells(
                &mut self.game_state.detected_tells,
                triggered,
                passenger.id,
                current_time,
            );
        }
    }

    fn apply_rule_consequences(&mut self, consequences: &[Consequence], current_time: f64) {
        for consequence in consequences {
            if rng::rand() > consequence.probability.clamp(0.0, 1.0) {
                continue;
            }

            match consequence.consequence_type {
                ConsequenceType::Death => {}
                ConsequenceType::Survival => {}
                ConsequenceType::Reputation => {
                    if let Some(passenger_id) = self
                        .game_state
                        .current_passenger
                        .as_ref()
                        .map(|passenger| passenger.id)
                    {
                        if let Some(rep) =
                            self.game_state.passenger_reputation.get_mut(&passenger_id)
                        {
                            if consequence.value >= 0 {
                                rep.positive_choices += consequence.value as u32;
                            } else {
                                rep.negative_choices += consequence.value.unsigned_abs();
                            }
                            rep.interactions += 1;
                            rep.last_encounter = current_time;
                        }
                    }
                }
                ConsequenceType::Money => {
                    if consequence.value >= 0 {
                        self.game_state.earnings += consequence.value as u32;
                    } else {
                        self.game_state.earnings = self
                            .game_state
                            .earnings
                            .saturating_sub(consequence.value.unsigned_abs());
                    }
                }
                ConsequenceType::Fuel => {
                    self.game_state.fuel = (self.game_state.fuel + consequence.value as f32)
                        .clamp(0.0, self.game_state.max_fuel);
                }
                ConsequenceType::Time => {
                    if consequence.value >= 0 {
                        self.game_state.time_remaining += consequence.value as u32;
                    } else {
                        self.game_state.time_remaining = self
                            .game_state
                            .time_remaining
                            .saturating_sub(consequence.value.unsigned_abs());
                    }
                }
                ConsequenceType::Item => {
                    if let Some(source) = self
                        .game_state
                        .current_passenger
                        .as_ref()
                        .map(|passenger| passenger.name.clone())
                    {
                        self.game_state
                            .inventory
                            .push(crate::data::ItemDatabase::create_item(
                                "Crumpled Note",
                                &source,
                                current_time,
                            ));
                    }
                }
                ConsequenceType::StoryUnlock => {
                    if let Some(passenger_id) = self
                        .game_state
                        .current_passenger
                        .as_ref()
                        .map(|passenger| passenger.id)
                    {
                        self.player_stats.mark_passenger_encountered(passenger_id);
                    }
                }
            }
        }
    }

    fn can_perform_cab_action(&self, action_key: &str) -> bool {
        if self.screen != Screen::Game || self.game_state.current_passenger.is_none() {
            return false;
        }

        match action_key {
            "accept_tip" => self.game_state.game_phase == GamePhase::DropOff,
            "stop_vehicle" => matches!(
                self.game_state.game_phase,
                GamePhase::Driving | GamePhase::Interaction
            ),
            _ => matches!(
                self.game_state.game_phase,
                GamePhase::RideRequest
                    | GamePhase::Driving
                    | GamePhase::Interaction
                    | GamePhase::GuidelineDecision
                    | GamePhase::DropOff
            ),
        }
    }

    fn cab_action_label(action_key: &str) -> &'static str {
        match action_key {
            "eye_contact" => "Make Eye Contact",
            "play_music" => "Play Music",
            "accept_tip" => "Accept Tip",
            "open_window" => "Open Window",
            "use_wipers" => "Use Wipers",
            "drive_dark" => "Kill Headlights",
            "use_ac" => "Use AC",
            "stop_vehicle" => "Stop Cab",
            _ => "Cab Action",
        }
    }

    fn cab_action_phrase(action_key: &str) -> &'static str {
        match action_key {
            "eye_contact" => "meet the passenger's eyes",
            "play_music" => "turn on the radio",
            "accept_tip" => "accept the offered tip",
            "open_window" => "crack the window",
            "use_wipers" => "switch on the wipers",
            "drive_dark" => "kill the headlights",
            "use_ac" => "turn on the AC",
            "stop_vehicle" => "pull the cab over",
            _ => "try the control",
        }
    }

    pub(super) fn evaluate_guideline_decision(&mut self, action: GuidelineAction) {
        let current_time = get_time();

        if let (Some(guideline), Some(passenger)) = (
            self.game_state.active_guideline.clone(),
            self.game_state.current_passenger.clone(),
        ) {
            let result = GuidelineEngine::evaluate_guideline_choice(
                &guideline,
                action,
                &passenger,
                &self.game_state,
            );

            let tells_present: Vec<_> = self
                .game_state
                .detected_tells
                .iter()
                .filter(|tell| tell.related_guideline == Some(guideline.id))
                .map(|tell| tell.tell.clone())
                .collect();

            self.game_state.decision_history.push(GuidelineDecision {
                guideline_id: guideline.id,
                passenger_id: passenger.id,
                action,
                was_correct: result.is_safe,
                tells_present,
                timestamp: current_time,
            });

            if result.is_safe {
                self.game_state.adjust_player_trust(0.08);
            } else {
                self.game_state.adjust_player_trust(-0.12);
            }

            for consequence in &result.consequences {
                match consequence.consequence_type {
                    ConsequenceType::Death => {
                        if rng::rand() < consequence.probability.clamp(0.0, 1.0) {
                            self.end_shift(false);
                            self.game_state.game_over_reason = Some(result.message.clone());
                            return;
                        }
                    }
                    ConsequenceType::Survival => {
                        self.game_state.player_trust =
                            (self.game_state.player_trust + 0.1).min(1.0);
                    }
                    ConsequenceType::Reputation => {
                        let rep_change = consequence.value;
                        if let Some(rep) =
                            self.game_state.passenger_reputation.get_mut(&passenger.id)
                        {
                            if rep_change > 0 {
                                rep.positive_choices += rep_change.unsigned_abs();
                            } else {
                                rep.negative_choices += rep_change.unsigned_abs();
                            }
                        }
                    }
                    ConsequenceType::Item => {}
                    _ => {}
                }
            }

            self.game_state.active_guideline = None;
            self.game_state.guideline_decision_start_time = None;
            self.game_state.detected_tells.clear();

            let completion_route = self
                .game_state
                .current_ride
                .as_ref()
                .and_then(|ride| ride.route_type)
                .or_else(|| {
                    self.game_state
                        .route_history
                        .last()
                        .map(|entry| entry.route_type)
                })
                .unwrap_or(RouteType::Normal);
            self.complete_ride(completion_route);
        }
    }
}

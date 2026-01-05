//! Passenger state machine for need level progression.

use crate::data::*;
use crate::state::*;
use crate::engine::RuleEvaluationResult;

/// Triggered tell with context
#[derive(Debug, Clone)]
pub struct TriggeredTell {
    pub tell: PassengerTell,
    pub exception_id: Option<String>,
    pub related_guideline_id: Option<u32>,
}

/// Passenger state machine for tracking need levels
pub struct PassengerStateMachine;

impl PassengerStateMachine {
    /// Initialize need state from passenger profile
    pub fn initialize(passenger: &Passenger, current_time: f64) -> Option<PassengerNeedState> {
        PassengerNeedState::from_passenger(passenger, current_time)
    }

    /// Apply route choice effects to need state
    pub fn apply_route_choice(
        state: &mut PassengerNeedState,
        passenger: &Passenger,
        route: RouteType,
        rule_outcome: Option<&RuleEvaluationResult>,
        current_time: f64,
    ) -> Vec<TriggeredTell> {
        let profile = &state.profile;
        let previous_stage = state.stage;

        // Apply passive need change
        let mut level = state.level as i32 + profile.need_change.passive;

        // Route-specific changes
        if route == RouteType::Shortcut {
            level += profile.need_change.break_rule;
        } else {
            level += profile.need_change.obey;
        }

        // Rule outcome adjustment
        if let Some(outcome) = rule_outcome {
            level += outcome.need_adjustment;
        }

        // Clamp level
        state.level = level.clamp(0, 100) as u32;

        // Calculate new stage
        let new_stage = PassengerNeedState::calculate_stage(state.level, &profile.thresholds);

        // Collect tells if stage changed and not already revealed
        let triggered_tells = if new_stage != previous_stage && !state.revealed_stages.contains_key(&new_stage) {
            state.revealed_stages.insert(new_stage, true);
            Self::collect_tells(passenger, new_stage, rule_outcome)
        } else {
            Vec::new()
        };

        // Update state
        state.stage = new_stage;
        state.stability = 1.0 - (state.level as f32 / 100.0);
        state.last_updated = current_time;

        triggered_tells
    }

    /// Collect tells appropriate for the current stage
    fn collect_tells(
        passenger: &Passenger,
        stage: NeedStage,
        rule_outcome: Option<&RuleEvaluationResult>,
    ) -> Vec<TriggeredTell> {
        let intensities = Self::get_stage_intensities(stage);

        passenger.tells.iter()
            .filter(|tell| intensities.contains(&tell.intensity))
            .map(|tell| TriggeredTell {
                tell: tell.clone(),
                exception_id: rule_outcome.and_then(|r| r.triggered_exception.as_ref().map(|e| e.id.clone())),
                related_guideline_id: rule_outcome.and_then(|r| r.rule.as_ref().and_then(|ru| ru.related_guideline_id)),
            })
            .collect()
    }

    /// Get expected tell intensities for a stage
    fn get_stage_intensities(stage: NeedStage) -> Vec<TellIntensity> {
        match stage {
            NeedStage::Calm => vec![TellIntensity::Subtle],
            NeedStage::Warning => vec![TellIntensity::Moderate],
            NeedStage::Critical => vec![TellIntensity::Obvious],
            NeedStage::Meltdown => vec![TellIntensity::Obvious],
        }
    }

    /// Get stage-specific dialogue
    pub fn get_dialogue_for_stage(_passenger: &Passenger, state: &PassengerNeedState) -> Option<String> {
        let stage_key = match state.stage {
            NeedStage::Calm => "calm",
            NeedStage::Warning => "warning",
            NeedStage::Critical => "critical",
            NeedStage::Meltdown => "meltdown",
        };

        state.profile.dialogue_by_stage.as_ref()
            .and_then(|map| map.get(stage_key))
            .and_then(|lines| {
                use rand::seq::SliceRandom;
                lines.choose(&mut rand::thread_rng()).cloned()
            })
    }

    /// Merge triggered tells into detected tells list
    pub fn merge_detected_tells(
        existing: &mut Vec<DetectedTell>,
        triggered: Vec<TriggeredTell>,
        passenger_id: u32,
        current_time: f64,
    ) {
        for trigger in triggered {
            existing.push(DetectedTell {
                tell: trigger.tell,
                passenger_id,
                detection_time: current_time,
                player_noticed: false,
                related_guideline: trigger.related_guideline_id,
                exception_id: trigger.exception_id,
            });
        }
    }

    /// Check if exception is active based on need state
    pub fn is_exception_active(state: &PassengerNeedState, exception_id: &str) -> bool {
        if let Some(ref profile_exception) = state.profile.exception_id {
            profile_exception == exception_id && state.stage >= NeedStage::Warning
        } else {
            false
        }
    }

    /// Check if passenger is in meltdown
    pub fn is_meltdown(state: &PassengerNeedState) -> bool {
        state.stage == NeedStage::Meltdown
    }

    /// Check if passenger needs immediate attention
    pub fn is_critical(state: &PassengerNeedState) -> bool {
        state.stage >= NeedStage::Critical
    }

    /// Get stability as percentage (0-100)
    pub fn get_stability_percent(state: &PassengerNeedState) -> u32 {
        (state.stability * 100.0) as u32
    }
}

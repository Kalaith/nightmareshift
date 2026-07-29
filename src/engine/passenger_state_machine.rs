//! Passenger state machine for need level progression.

use crate::data::*;
use crate::engine::RuleEvaluationResult;
use crate::state::*;

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
        route_preference_stress_scale: f32,
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

        if let Some(preference) = passenger.get_route_preference(route) {
            level += (preference.stress_modifier * route_preference_stress_scale).round() as i32;
        }

        // Rule outcome adjustment
        if let Some(outcome) = rule_outcome {
            level += outcome.need_adjustment;
        }

        Self::set_level_and_collect_tells(
            state,
            passenger,
            level,
            previous_stage,
            rule_outcome,
            current_time,
        )
    }

    /// Apply a rule outcome directly to the passenger need state.
    pub fn apply_rule_outcome(
        state: &mut PassengerNeedState,
        passenger: &Passenger,
        rule_outcome: &RuleEvaluationResult,
        current_time: f64,
    ) -> Vec<TriggeredTell> {
        if rule_outcome.need_adjustment == 0 {
            state.last_updated = current_time;
            return Vec::new();
        }

        let previous_stage = state.stage;
        let level = state.level as i32 + rule_outcome.need_adjustment;
        Self::set_level_and_collect_tells(
            state,
            passenger,
            level,
            previous_stage,
            Some(rule_outcome),
            current_time,
        )
    }

    /// Apply a raw stress delta and keep stage/stability in sync.
    pub fn apply_stress_delta(
        state: &mut PassengerNeedState,
        passenger: &Passenger,
        delta: i32,
        current_time: f64,
    ) -> Vec<TriggeredTell> {
        if delta == 0 {
            state.last_updated = current_time;
            return Vec::new();
        }

        let previous_stage = state.stage;
        let level = state.level as i32 + delta;
        Self::set_level_and_collect_tells(
            state,
            passenger,
            level,
            previous_stage,
            None,
            current_time,
        )
    }

    fn set_level_and_collect_tells(
        state: &mut PassengerNeedState,
        passenger: &Passenger,
        level: i32,
        previous_stage: NeedStage,
        rule_outcome: Option<&RuleEvaluationResult>,
        current_time: f64,
    ) -> Vec<TriggeredTell> {
        state.level = level.clamp(0, 100) as u32;
        let new_stage = PassengerNeedState::calculate_stage(state.level, &state.profile.thresholds);

        let triggered_tells =
            if new_stage != previous_stage && !state.revealed_stages.contains_key(&new_stage) {
                state.revealed_stages.insert(new_stage, true);
                Self::collect_tells(
                    passenger,
                    new_stage,
                    rule_outcome,
                    state.profile.exception_id.as_deref(),
                    state.profile.tell_intensities.as_ref(),
                )
            } else {
                Vec::new()
            };

        state.stage = new_stage;
        state.stability = 1.0 - (state.level as f32 / 100.0);
        state.last_updated = current_time;

        triggered_tells
    }

    /// Collect tells appropriate for the current stage
    ///
    /// `profile_exception` is the passenger's own `stateProfile.exceptionId`.
    /// A tell raised by the need rising is *about* that exception, so it is
    /// used whenever the rule outcome does not name one; without the fallback
    /// a need-driven tell carries no exception and no guideline, and
    /// `check_guideline_triggers` can never match it.
    fn collect_tells(
        passenger: &Passenger,
        stage: NeedStage,
        rule_outcome: Option<&RuleEvaluationResult>,
        profile_exception: Option<&str>,
        authored: Option<&TellIntensityMap>,
    ) -> Vec<TriggeredTell> {
        let intensities = Self::stage_intensities(stage, authored);

        passenger
            .tells
            .iter()
            .filter(|tell| intensities.contains(&tell.intensity))
            .map(|tell| TriggeredTell {
                tell: tell.clone(),
                exception_id: rule_outcome
                    .and_then(|r| r.triggered_exception.as_ref().map(|e| e.id.clone()))
                    .or_else(|| profile_exception.map(str::to_string)),
                related_guideline_id: rule_outcome
                    .and_then(|r| r.rule.as_ref().and_then(|ru| ru.related_guideline_id)),
            })
            .collect()
    }

    /// Which tell intensities show at a stage.
    ///
    /// A passenger's `stateProfile.tellIntensities` overrides the defaults for
    /// the stages it names, so one passenger can start giving obvious signs at
    /// Warning while another stays subtle right up to Critical. Every profile
    /// authors this map and it was read by nothing, so every passenger on the
    /// roster tipped their hand at exactly the same point.
    fn stage_intensities(
        stage: NeedStage,
        authored: Option<&TellIntensityMap>,
    ) -> Vec<TellIntensity> {
        if let Some(map) = authored {
            if let Some(names) = map.get(Self::stage_key(stage)) {
                let parsed: Vec<TellIntensity> = names
                    .iter()
                    .filter_map(|n| Self::parse_intensity(n))
                    .collect();
                if !parsed.is_empty() {
                    return parsed;
                }
            }
        }
        match stage {
            NeedStage::Calm => vec![TellIntensity::Subtle],
            NeedStage::Warning => vec![TellIntensity::Moderate],
            NeedStage::Critical => vec![TellIntensity::Obvious],
            NeedStage::Meltdown => vec![TellIntensity::Obvious],
        }
    }

    /// The JSON key a stage is authored under in `tellIntensities` and
    /// `dialogueByStage`.
    fn stage_key(stage: NeedStage) -> &'static str {
        match stage {
            NeedStage::Calm => "calm",
            NeedStage::Warning => "warning",
            NeedStage::Critical => "critical",
            NeedStage::Meltdown => "meltdown",
        }
    }

    /// Parse an authored intensity name, ignoring anything unrecognised so a
    /// typo degrades to the default rather than silencing the passenger.
    fn parse_intensity(name: &str) -> Option<TellIntensity> {
        match name.to_lowercase().as_str() {
            "subtle" => Some(TellIntensity::Subtle),
            "moderate" => Some(TellIntensity::Moderate),
            "obvious" => Some(TellIntensity::Obvious),
            _ => None,
        }
    }

    /// The line a passenger actually speaks when a verbal tell fires.
    ///
    /// `PassengerTell` authors a `tellType` and, for verbal ones, a
    /// `triggerPhrase` — the words the player is supposed to catch. Both were
    /// read by nothing, so every tell surfaced only as a description on the
    /// guideline screen and a verbal tell sounded exactly like a behavioural
    /// one. A spoken tell now takes precedence over the generic stage line,
    /// because it is the more specific signal.
    pub fn spoken_tell(triggered: &[TriggeredTell]) -> Option<String> {
        triggered
            .iter()
            .find(|t| t.tell.tell_type == TellType::Verbal && t.tell.trigger_phrase.is_some())
            .and_then(|t| t.tell.trigger_phrase.clone())
            .map(|phrase| format!("\"{}...\"", phrase))
    }

    /// Get stage-specific dialogue
    pub fn get_dialogue_for_stage(
        _passenger: &Passenger,
        state: &PassengerNeedState,
    ) -> Option<String> {
        let stage_key = Self::stage_key(state.stage);

        state
            .profile
            .dialogue_by_stage
            .as_ref()
            .and_then(|map| map.get(stage_key))
            .and_then(|lines| macroquad_toolkit::rng::choose(lines).cloned())
    }

    /// Merge triggered tells into detected tells list.
    ///
    /// A tell that names an exception but no guideline is resolved against
    /// `guidelines` to find the one that owns that exception. The guideline
    /// decision only triggers on tells that carry a `related_guideline`, so
    /// this is what connects a rising need to the decision it should provoke.
    pub fn merge_detected_tells(
        existing: &mut Vec<DetectedTell>,
        triggered: Vec<TriggeredTell>,
        passenger_id: u32,
        current_time: f64,
        guidelines: &[Guideline],
    ) {
        for trigger in triggered {
            let related_guideline = trigger.related_guideline_id.or_else(|| {
                let exception_id = trigger.exception_id.as_deref()?;
                guidelines
                    .iter()
                    .find(|g| g.exceptions.iter().any(|e| e.id == exception_id))
                    .map(|g| g.id)
            });
            existing.push(DetectedTell {
                tell: trigger.tell,
                passenger_id,
                detection_time: current_time,
                player_noticed: false,
                related_guideline,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_passengers;

    /// Every stage a profile authors intensities for must parse to at least
    /// one real intensity, or the passenger silently falls back to the
    /// defaults and the authored tuning does nothing.
    #[test]
    fn authored_tell_intensities_all_parse() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let Some(map) = &profile.tell_intensities else {
                continue;
            };
            for (stage, names) in map {
                let parsed = names
                    .iter()
                    .filter(|n| PassengerStateMachine::parse_intensity(n).is_some())
                    .count();
                assert_eq!(
                    parsed,
                    names.len(),
                    "{} authors an unrecognised intensity for {stage:?}: {names:?}",
                    passenger.name
                );
            }
        }
    }

    /// Every stage key a profile authors must be one the state machine looks
    /// up, otherwise the entry is never read.
    #[test]
    fn authored_stage_keys_are_recognised() {
        let known = [
            PassengerStateMachine::stage_key(NeedStage::Calm),
            PassengerStateMachine::stage_key(NeedStage::Warning),
            PassengerStateMachine::stage_key(NeedStage::Critical),
            PassengerStateMachine::stage_key(NeedStage::Meltdown),
        ];
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            if let Some(map) = &profile.tell_intensities {
                for stage in map.keys() {
                    assert!(
                        known.contains(&stage.as_str()),
                        "{} authors tellIntensities for unknown stage {stage:?}",
                        passenger.name
                    );
                }
            }
            if let Some(dialogue) = &profile.dialogue_by_stage {
                for stage in dialogue.keys() {
                    assert!(
                        known.contains(&stage.as_str()),
                        "{} authors dialogue for unknown stage {stage:?}",
                        passenger.name
                    );
                }
            }
        }
    }

    /// An authored intensity map must actually change which tells surface,
    /// or wiring it bought nothing over the hardcoded defaults.
    #[test]
    fn authored_intensities_override_the_defaults() {
        let mut map = TellIntensityMap::new();
        map.insert("warning".to_string(), vec!["obvious".to_string()]);
        assert_eq!(
            PassengerStateMachine::stage_intensities(NeedStage::Warning, Some(&map)),
            vec![TellIntensity::Obvious],
        );
        // Unnamed stages keep the default.
        assert_eq!(
            PassengerStateMachine::stage_intensities(NeedStage::Critical, Some(&map)),
            vec![TellIntensity::Obvious],
        );
        assert_eq!(
            PassengerStateMachine::stage_intensities(NeedStage::Calm, Some(&map)),
            vec![TellIntensity::Subtle],
        );
    }

    /// Every passenger must surface a tell at *both* warning and critical.
    /// Authoring an intensity the passenger owns no tell of leaves that stage
    /// silent, which is how six of the roster used to reach one of the two
    /// stages with nothing to show for it.
    #[test]
    fn every_passenger_surfaces_a_tell_at_each_stage() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            for stage in [NeedStage::Warning, NeedStage::Critical] {
                let surfaces = PassengerStateMachine::stage_intensities(
                    stage,
                    profile.tell_intensities.as_ref(),
                )
                .into_iter()
                .any(|intensity| passenger.tells.iter().any(|t| t.intensity == intensity));
                assert!(surfaces, "{} surfaces no tell at {stage:?}", passenger.name);
            }
        }
    }

    /// The reveal must escalate: whatever shows at critical must be at least
    /// as strong as what showed at warning, or the passenger gets quieter as
    /// they get closer to breaking.
    #[test]
    fn tells_escalate_from_warning_to_critical() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let authored = profile.tell_intensities.as_ref();
            let warning = PassengerStateMachine::stage_intensities(NeedStage::Warning, authored);
            let critical = PassengerStateMachine::stage_intensities(NeedStage::Critical, authored);
            let (Some(faintest_warning), Some(strongest_critical)) =
                (warning.iter().min(), critical.iter().max())
            else {
                continue;
            };
            assert!(
                strongest_critical >= faintest_warning,
                "{} goes quieter at critical ({critical:?}) than at warning ({warning:?})",
                passenger.name
            );
        }
    }

    /// A verbal tell with a trigger phrase is what the passenger says; a
    /// behavioural one is not spoken.
    #[test]
    fn only_verbal_tells_are_spoken() {
        let behavioural = TriggeredTell {
            tell: PassengerTell {
                tell_type: TellType::Behavioral,
                intensity: TellIntensity::Obvious,
                description: "Fidgets".to_string(),
                trigger_phrase: Some("should not be said".to_string()),
                animation_cue: None,
                audio_cue: None,
                reliability: 1.0,
            },
            exception_id: None,
            related_guideline_id: None,
        };
        assert!(PassengerStateMachine::spoken_tell(&[behavioural]).is_none());

        let verbal = TriggeredTell {
            tell: PassengerTell {
                tell_type: TellType::Verbal,
                intensity: TellIntensity::Moderate,
                description: "Asks about the tide".to_string(),
                trigger_phrase: Some("Tide is wrong".to_string()),
                animation_cue: None,
                audio_cue: None,
                reliability: 1.0,
            },
            exception_id: None,
            related_guideline_id: None,
        };
        let spoken = PassengerStateMachine::spoken_tell(&[verbal]).expect("verbal tell speaks");
        assert!(spoken.contains("Tide is wrong"));
    }
}

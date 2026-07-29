//! Route selection: cost validation, rule checks, transit effects, and the
//! phase transition that follows a chosen route.

use crate::data::*;
use crate::engine::{
    GameEngine, PassengerStateMachine, ProtectionService, RouteCosts, RouteService,
    RuleEvaluationResult, SkillModifiers,
};
use crate::state::*;

use super::{RideService, RouteOutcome};

impl RideService {
    /// Choose a route and determine the outcome
    pub fn choose_route(
        state: &mut GameState,
        data: &GameData,
        stats: &mut PlayerStats,
        route: RouteType,
        current_time: f64,
    ) -> RouteOutcome {
        // Calculate route costs
        let mut passenger_risk = state
            .current_passenger
            .as_ref()
            .and_then(|p| data.get_location(&p.pickup).map(|l| l.risk_level))
            .unwrap_or(1);
        if let Some(passenger) = state.current_passenger.as_ref() {
            if let Some(reputation) = state.passenger_reputation.get(&passenger.id) {
                let adjusted =
                    passenger_risk as i32 + reputation.risk_modifier(&data.constants.reputation);
                passenger_risk = adjusted.clamp(0, 5) as u32;
            }
        }

        let route_mastery = stats.route_mastery_map();
        let skill_mods = SkillModifiers::from_unlocked(&data.skills, &stats.unlocked_skills);
        let mut costs = RouteService::calculate_route_costs(
            route,
            &data.constants,
            passenger_risk,
            Some(&state.current_weather),
            Some(&state.time_of_day),
            &state.environmental_hazards,
            &route_mastery,
            state.current_passenger.as_ref(),
            &skill_mods,
        );
        Self::apply_curse_route_pressure(state, &mut costs, &data.constants);

        // 1. Check resources
        if let Some(outcome) = Self::validate_resources(state, &costs) {
            return outcome;
        }

        // 2. Check rule violations
        if let Some(outcome) = Self::check_route_rules(state, route) {
            return outcome;
        }

        // 3. Apply costs and record history
        Self::apply_transit_effects(state, &costs, route, current_time);
        stats.record_route_usage(route);

        // 3b. A risky leg can cost more than its stated price.
        let encounter = Self::apply_risk_encounters(state, &costs, &data.constants, current_time);
        if let Some(encounter) = encounter {
            state.current_dialogue = Some(CurrentDialogue {
                text: encounter.message,
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
        }

        // 4. Update passenger state machine
        Self::update_passenger_state(
            state,
            route,
            current_time,
            data.constants.game_constants.route_preference_stress_scale,
        );
        if Self::is_passenger_meltdown(state)
            && !Self::absorb_meltdown_with_protection(state, current_time)
        {
            return RouteOutcome::GameOver(
                "The passenger's need became uncontrollable.".to_string(),
            );
        }

        // 5. Check guidelines
        if let Some(outcome) = Self::check_guideline_triggers(state, current_time) {
            return outcome;
        }

        // 6. Progress phase
        Self::transition_driving_phase(state, data, stats, route, current_time)
    }

    /// Check if player has enough resources for the route
    fn validate_resources(state: &GameState, costs: &RouteCosts) -> Option<RouteOutcome> {
        if (state.fuel as u32) < costs.fuel {
            return Some(RouteOutcome::GameOver(
                "Not enough fuel for this route.".to_string(),
            ));
        }

        if state.time_remaining < costs.time {
            return Some(RouteOutcome::GameOver(
                "Not enough time for this route.".to_string(),
            ));
        }

        None
    }

    /// Check for rule violations specific to the route
    fn check_route_rules(state: &mut GameState, route: RouteType) -> Option<RouteOutcome> {
        if route == RouteType::Shortcut {
            // Check visible rules
            let violation = GameEngine::check_rule_violation(
                &state.current_rules,
                "take_shortcut",
                state.current_passenger.as_ref(),
                state.current_passenger_need_state.as_ref(),
            );

            if violation.violation {
                return Self::resolve_rule_violation(state, violation, false);
            }

            // Check hidden rules
            let hidden_violation = GameEngine::check_rule_violation(
                &state.hidden_rules,
                "take_shortcut",
                state.current_passenger.as_ref(),
                state.current_passenger_need_state.as_ref(),
            );

            if hidden_violation.violation {
                return Self::resolve_rule_violation(state, hidden_violation, true);
            }
        }

        let weather_violation = GameEngine::check_weather_route_violation(
            &state.current_rules,
            route,
            &state.current_weather,
            &state.time_of_day,
        );
        if weather_violation.violation {
            return Self::resolve_rule_violation(state, weather_violation, false);
        }

        let hidden_weather_violation = GameEngine::check_weather_route_violation(
            &state.hidden_rules,
            route,
            &state.current_weather,
            &state.time_of_day,
        );
        if hidden_weather_violation.violation {
            return Self::resolve_rule_violation(state, hidden_weather_violation, true);
        }

        None
    }

    fn resolve_rule_violation(
        state: &mut GameState,
        violation: RuleEvaluationResult,
        hidden: bool,
    ) -> Option<RouteOutcome> {
        let rule_title = violation
            .rule
            .as_ref()
            .map(|rule| rule.title.clone())
            .unwrap_or_else(|| "Rule".to_string());

        if hidden {
            if let Some(rule) = &violation.rule {
                state.reveal_hidden_rule(rule.id);
            }
        }

        Self::apply_rule_need_adjustment(state, &violation, macroquad::prelude::get_time());
        state.rules_violated += 1;
        state.adjust_player_trust(-0.08);

        let message = violation
            .message
            .unwrap_or_else(|| "Rule violation".to_string());

        let passenger_id = state.current_passenger.as_ref().map(|p| p.id);
        let forgiven_by = if state.rule_immunity_charges > 0 {
            state.rule_immunity_charges -= 1;
            Some(None)
        } else {
            ProtectionService::consume_ward(
                &mut state.inventory,
                ProtectionType::RuleForgiveness,
                passenger_id,
            )
            .map(|ward| Some(ward.describe()))
        };

        if let Some(ward_name) = forgiven_by {
            let ward_label = ward_name.unwrap_or_else(|| "ward".to_string());
            state.current_dialogue = Some(CurrentDialogue {
                text: format!(
                    "The {} absorbs the {} violation. {}",
                    ward_label, rule_title, message
                ),
                speaker: DialogueSpeaker::Narrator,
                timestamp: macroquad::prelude::get_time(),
            });
            return None;
        }

        if state.rides_completed == 0 {
            state.current_dialogue = Some(CurrentDialogue {
                text: if hidden {
                    format!("Hidden rule revealed: {}. {}", rule_title, message)
                } else {
                    format!("Rule pressure spikes: {}. {}", rule_title, message)
                },
                speaker: DialogueSpeaker::Narrator,
                timestamp: macroquad::prelude::get_time(),
            });
            return None;
        }

        Some(RouteOutcome::GameOver(if hidden {
            format!("Hidden Rule Violated!\n{}", message)
        } else {
            message
        }))
    }

    fn apply_rule_need_adjustment(
        state: &mut GameState,
        outcome: &RuleEvaluationResult,
        current_time: f64,
    ) {
        if let (Some(mut need_state), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let triggered = PassengerStateMachine::apply_rule_outcome(
                &mut need_state,
                &passenger,
                outcome,
                current_time,
            );
            state.current_passenger_need_state = Some(need_state);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                passenger.id,
                current_time,
                &state.current_guidelines,
            );
        }
    }

    fn apply_curse_route_pressure(
        state: &mut GameState,
        costs: &mut RouteCosts,
        constants: &ConstantsData,
    ) {
        if state.curse_danger_bonus == 0 {
            return;
        }

        let bonus = state.curse_danger_bonus.min(3);
        costs.risk = (costs.risk + bonus).min(constants.risk.max_risk_level);
        costs.time += bonus * 2;
        state.curse_danger_bonus = state.curse_danger_bonus.saturating_sub(bonus);
    }

    /// Apply costs, tracking, and history
    fn apply_transit_effects(
        state: &mut GameState,
        costs: &RouteCosts,
        route: RouteType,
        current_time: f64,
    ) {
        // Deduct resources
        state.fuel -= costs.fuel as f32;
        state.time_remaining = state.time_remaining.saturating_sub(costs.time);

        // Update route tracking
        state.update_route_streak(route);
        let driving_phase = state.driving_phase.unwrap_or(DrivingPhase::Pickup);

        if let Some(current_ride) = &mut state.current_ride {
            current_ride.route_type = Some(route);
            current_ride.driving_phase = driving_phase;
        }

        // Record history
        let passenger_id = state.current_passenger.as_ref().map(|p| p.id);

        state.route_history.push(RouteHistoryEntry {
            route_type: route,
            driving_phase,
            fuel_cost: costs.fuel,
            time_cost: costs.time,
            risk_level: costs.risk,
            passenger_id,
            timestamp: current_time,
        });
    }

    /// Update passenger stress/state based on route
    fn update_passenger_state(
        state: &mut GameState,
        route: RouteType,
        current_time: f64,
        route_preference_stress_scale: f32,
    ) {
        if let (Some(mut need_state), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let mut triggered_tells = PassengerStateMachine::apply_route_choice(
                &mut need_state,
                &passenger,
                route,
                None, // No rule evaluation result passed here contextually
                route_preference_stress_scale,
                current_time,
            );

            let weather_stress: i32 = state
                .current_weather
                .effects
                .iter()
                .filter(|effect| effect.effect_type == WeatherEffectType::PassengerBehavior)
                .map(|effect| (effect.value.max(0) / 5).max(1))
                .sum();
            if weather_stress > 0 {
                let weather_tells = PassengerStateMachine::apply_stress_delta(
                    &mut need_state,
                    &passenger,
                    weather_stress,
                    current_time,
                );
                triggered_tells.extend(weather_tells);
            }

            // A verbal tell that just fired is the most specific thing the
            // passenger is doing, so it outranks both the route reaction and
            // the generic stage line.
            let spoken_tell = PassengerStateMachine::spoken_tell(&triggered_tells);
            let route_dialogue = Self::route_reaction_dialogue(&passenger, route);
            let stage_dialogue =
                PassengerStateMachine::get_dialogue_for_stage(&passenger, &need_state);
            let dialogue = spoken_tell.or(route_dialogue).or(stage_dialogue);

            state.current_passenger_need_state = Some(need_state);

            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered_tells,
                passenger.id,
                current_time,
                &state.current_guidelines,
            );

            if let Some(dialogue) = dialogue {
                state.current_dialogue = Some(CurrentDialogue {
                    text: dialogue.clone(),
                    speaker: DialogueSpeaker::Passenger,
                    timestamp: current_time,
                });
                state.current_passenger_dialogue = Some(dialogue);
            }
        }
    }

    fn route_reaction_dialogue(passenger: &Passenger, route: RouteType) -> Option<String> {
        let preference = passenger.get_route_preference(route)?;
        if preference.preference == PreferenceLevel::Neutral {
            return None;
        }

        let line = preference.special_dialogue.as_ref()?;
        let trigger_chance = preference.trigger_chance.unwrap_or(1.0).clamp(0.0, 1.0);
        if macroquad_toolkit::rng::gen_range(0.0, 1.0) <= trigger_chance {
            Some(line.clone())
        } else {
            None
        }
    }

    fn is_passenger_meltdown(state: &GameState) -> bool {
        state
            .current_passenger_need_state
            .as_ref()
            .map(PassengerStateMachine::is_meltdown)
            .unwrap_or(false)
    }

    /// Try to take a meltdown on the chin using whatever protection is
    /// carried: first the `supernatural_protection` counter granted by item
    /// effects and skills, then an actual warding item from the inventory.
    fn absorb_meltdown_with_protection(state: &mut GameState, current_time: f64) -> bool {
        if !Self::is_passenger_meltdown(state) {
            return false;
        }

        let should_protect = state
            .current_passenger
            .as_ref()
            .map(|passenger| passenger.is_supernatural)
            .unwrap_or(false)
            || state
                .current_weather
                .effects
                .iter()
                .any(|effect| effect.effect_type == WeatherEffectType::SupernaturalAttraction);

        if !should_protect {
            return false;
        }

        // Spend the cheapest thing that will cover it: the counter first, so
        // a carried ward survives while a skill charge is available.
        let passenger_id = state.current_passenger.as_ref().map(|p| p.id);
        // A carried ward pulls the passenger back by its authored strength;
        // a bare counter charge is worth one step of it.
        const RELIEF_PER_STRENGTH: i32 = 18;
        let (absorbed_by, relief) = if state.supernatural_protection > 0 {
            state.supernatural_protection -= 1;
            (None, RELIEF_PER_STRENGTH * 2)
        } else {
            match ProtectionService::consume_ward(
                &mut state.inventory,
                ProtectionType::SupernaturalImmunity,
                passenger_id,
            ) {
                Some(ward) => (
                    Some(ward.describe()),
                    RELIEF_PER_STRENGTH * ward.strength.max(1) as i32,
                ),
                None => return false,
            }
        };

        if let (Some(mut need_state), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let triggered = PassengerStateMachine::apply_stress_delta(
                &mut need_state,
                &passenger,
                -relief,
                current_time,
            );
            state.current_passenger_need_state = Some(need_state);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                passenger.id,
                current_time,
                &state.current_guidelines,
            );
            state.current_dialogue = Some(CurrentDialogue {
                text: match absorbed_by {
                    Some(name) => format!(
                        "The {} flares and pulls the passenger back from the edge.",
                        name
                    ),
                    None => "A protective charm flares and pulls the passenger back from the edge."
                        .to_string(),
                },
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
        }

        !Self::is_passenger_meltdown(state)
    }

    /// Check if guideline decision should be triggered.
    ///
    /// Fires on the ride's final leg, because resolving the decision completes
    /// the ride (`evaluate_guideline_decision` calls `complete_ride`). This
    /// used to gate on `DrivingPhase::Destination`, a phase the mid-ride-event
    /// flow never enters — `transition_driving_phase` even marks its
    /// `Destination` arm "should not happen with new flow" — so the decision,
    /// and with it every authored guideline exception, was unreachable.
    fn check_guideline_triggers(state: &mut GameState, current_time: f64) -> Option<RouteOutcome> {
        // `apply_transit_effects` has already recorded this leg, so a count
        // above one means we are on the second (final) leg of the ride.
        let passenger_id = state.current_passenger.as_ref().map(|p| p.id);
        let final_leg = state
            .route_history
            .iter()
            .filter(|r| r.passenger_id == passenger_id)
            .count()
            > 1;

        let should_check =
            !state.current_guidelines.is_empty() && !state.detected_tells.is_empty() && final_leg;

        if should_check {
            // Find a guideline that has detected tells
            if let Some(guideline) = state
                .current_guidelines
                .iter()
                .find(|g| {
                    state.detected_tells.iter().any(|t| {
                        if t.related_guideline != Some(g.id) {
                            return false;
                        }
                        match (&state.current_passenger_need_state, &t.exception_id) {
                            (Some(need_state), Some(exception_id)) => {
                                PassengerStateMachine::is_exception_active(need_state, exception_id)
                            }
                            _ => true,
                        }
                    })
                })
                .cloned()
            {
                // Enter guideline decision phase
                state.active_guideline = Some(guideline);
                state.guideline_decision_start_time = Some(current_time);
                state.guideline_time_remaining = 30.0;
                state.game_phase = GamePhase::GuidelineDecision;
                return Some(RouteOutcome::GuidelineTriggered);
            }
        }
        None
    }

    /// Advance game phase
    fn transition_driving_phase(
        state: &mut GameState,
        data: &GameData,
        stats: &mut PlayerStats,
        route: RouteType,
        current_time: f64,
    ) -> RouteOutcome {
        match state.driving_phase {
            Some(DrivingPhase::Pickup) => {
                // Check how many routes we've taken with this passenger
                let pid = state.current_passenger.as_ref().map(|p| p.id);
                let ride_legs = state
                    .route_history
                    .iter()
                    .filter(|r| r.passenger_id == pid)
                    .count();

                if ride_legs <= 1 {
                    // First leg -> Mid-Ride Event
                    state.current_event =
                        Some(Self::generate_mid_ride_event(state, data, stats, route));
                    state.game_phase = GamePhase::Interaction;
                    RouteOutcome::Success
                } else {
                    // Second leg -> Complete Ride (DropOff)
                    Self::complete_ride(state, data, stats, route, current_time);
                    RouteOutcome::RideCompleted
                }
            }
            Some(DrivingPhase::Destination) => {
                // Should not happen with new flow, but safe fallback
                Self::complete_ride(state, data, stats, route, current_time);
                RouteOutcome::RideCompleted
            }
            None => RouteOutcome::Success,
        }
    }
}

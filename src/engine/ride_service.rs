//! Service for managing the ride lifecycle (spawn, accept, decline, complete).

use crate::data::event::{EventChoice, EventConsequence, MidRideEvent, RiskTag};
use crate::data::*;
use crate::engine::{
    GameEngine, ItemService, PassengerSelectionContext, PassengerService, PassengerStateMachine,
    RouteCosts, RouteService, RuleEvaluationResult,
};
use crate::state::*;
use crate::ui::layout; // For constants like MINIMUM_FUEL_FOR_RIDE

/// Outcome of a route choice
#[derive(Debug, PartialEq, Eq)]
pub enum RouteOutcome {
    Success,
    GameOver(String),
    GuidelineTriggered,
    RideCompleted,
}

pub struct RideService;

impl RideService {
    /// Spawn a new passenger. Returns true if a passenger was found, false otherwise.
    pub fn spawn_passenger(state: &mut GameState, data: &GameData, current_time: f64) -> bool {
        let context = PassengerSelectionContext {
            difficulty_level: state.difficulty_level,
            weather: &state.current_weather,
            time_of_day: &state.time_of_day,
            season: &state.season,
            constants: &data.constants,
        };

        let passenger = PassengerService::select_weather_aware_passenger(
            &data.passengers,
            &state.used_passengers,
            &context,
        );

        if let Some(p) = passenger {
            state.used_passengers.push(p.id);
            state.current_passenger_need_state =
                PassengerStateMachine::initialize(&p, current_time);
            // Select dialogue once and store it
            state.current_passenger_dialogue = p.random_dialogue().map(|s| s.to_string());
            state.current_passenger = Some(p);
            state.game_phase = GamePhase::RideRequest;
            true
        } else {
            false
        }
    }

    /// Accept the current ride request.
    /// Returns Ok if successful, Err(reason) if failed (e.g. not enough fuel).
    pub fn accept_ride(state: &mut GameState) -> Result<(), String> {
        if state.fuel < layout::MINIMUM_FUEL_FOR_RIDE {
            return Err("You ran out of fuel with a passenger in the car.".to_string());
        }

        let current_time = macroquad::prelude::get_time();
        if let Some(passenger) = state.current_passenger.clone() {
            state.current_ride = Some(CurrentRide {
                pickup_location: passenger.pickup.clone(),
                destination_location: passenger.destination.clone(),
                passenger,
                route_type: None,
                driving_phase: DrivingPhase::Pickup,
                start_time: current_time,
            });
        }

        state.game_phase = GamePhase::Driving;
        state.driving_phase = Some(DrivingPhase::Pickup);
        state.current_dialogue = Some(CurrentDialogue {
            text: "Ride accepted. Choose a route to the pickup point.".to_string(),
            speaker: DialogueSpeaker::Driver,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Decline the current ride request.
    /// Clears passenger state and returns to Waiting phase (caller should trigger spawn again if needed).
    pub fn decline_ride(state: &mut GameState) {
        state.current_passenger = None;
        state.current_passenger_dialogue = None;
        state.current_passenger_need_state = None;
        state.current_ride = None;
        state.current_event = None;
        state.game_phase = GamePhase::Waiting;
    }

    /// Complete the current ride logic.
    pub fn complete_ride(
        state: &mut GameState,
        data: &GameData,
        stats: &mut PlayerStats,
        route: RouteType,
        current_time: f64,
    ) {
        if let Some(ref passenger) = state.current_passenger.clone() {
            // Calculate fare
            let reputation = state.passenger_reputation.get(&passenger.id);
            let destination_fare_modifier = data
                .get_location(&passenger.destination)
                .map(|l| l.fare_modifier)
                .unwrap_or(1.0);
            let fare = GameEngine::calculate_fare(
                passenger.fare,
                route,
                passenger,
                state.consecutive_route_streak.as_ref(),
                reputation,
                &data.constants,
                destination_fare_modifier,
            );

            // Add earnings
            state.earnings += fare;
            state.rides_completed += 1;

            // Check backstory unlock
            let backstory_unlocked =
                if PassengerService::check_backstory_unlock(passenger.id, stats, &data.constants) {
                    stats.unlock_backstory(passenger.id);
                    Some((passenger.name.clone(), passenger.backstory_details.clone()))
                } else {
                    None
                };

            // Record encounter
            stats.record_passenger_encounter(passenger.id);

            // Update reputation
            let is_positive = passenger
                .get_route_preference(route)
                .map(|p| {
                    matches!(
                        p.preference,
                        PreferenceLevel::Loves | PreferenceLevel::Likes
                    )
                })
                .unwrap_or(false);

            state.get_passenger_reputation(passenger.id).update(
                is_positive,
                current_time,
                &data.constants.reputation,
            );

            // Generate item drop
            let mut items_received = Vec::new();
            if let Some(drop) = ItemService::generate_drop(
                passenger,
                route,
                backstory_unlocked.is_some(),
                current_time,
                &data.constants,
                &data.item_pools,
            ) {
                items_received.push(drop.item.clone());
                state.inventory.push(drop.item);
            }

            // Check for trade offer
            if let Some(trade) = ItemService::check_trade_offer(
                passenger,
                &state.inventory,
                &data.constants,
                current_time,
                &data.item_pools,
            ) {
                state.pending_trade =
                    Some((trade.passenger_name.clone(), trade.offered_item.clone()));
            }

            // Create completion data
            state.last_ride_completion = Some(RideCompletion {
                passenger: passenger.clone(),
                fare_earned: fare,
                items_received,
                backstory_unlocked,
            });

            state.game_phase = GamePhase::DropOff;
        }
    }

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
        let mut costs = RouteService::calculate_route_costs(
            route,
            &data.constants,
            passenger_risk,
            Some(&state.current_weather),
            Some(&state.time_of_day),
            &state.environmental_hazards,
            &route_mastery,
            state.current_passenger.as_ref(),
        );
        Self::apply_curse_route_pressure(state, &mut costs);

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

        if state.rule_immunity_charges > 0 {
            state.rule_immunity_charges -= 1;
            state.current_dialogue = Some(CurrentDialogue {
                text: format!("The ward absorbs the {} violation. {}", rule_title, message),
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
            );
        }
    }

    fn apply_curse_route_pressure(state: &mut GameState, costs: &mut RouteCosts) {
        if state.curse_danger_bonus == 0 {
            return;
        }

        let bonus = state.curse_danger_bonus.min(3);
        costs.risk = (costs.risk + bonus).min(5);
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

            let route_dialogue = Self::route_reaction_dialogue(&passenger, route);
            let stage_dialogue =
                PassengerStateMachine::get_dialogue_for_stage(&passenger, &need_state);
            let dialogue = route_dialogue.or(stage_dialogue);

            state.current_passenger_need_state = Some(need_state);

            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered_tells,
                passenger.id,
                current_time,
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

    fn absorb_meltdown_with_protection(state: &mut GameState, current_time: f64) -> bool {
        if state.supernatural_protection == 0 || !Self::is_passenger_meltdown(state) {
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

        if let (Some(mut need_state), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            state.supernatural_protection -= 1;
            let triggered = PassengerStateMachine::apply_stress_delta(
                &mut need_state,
                &passenger,
                -35,
                current_time,
            );
            state.current_passenger_need_state = Some(need_state);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                passenger.id,
                current_time,
            );
            state.current_dialogue = Some(CurrentDialogue {
                text: "A protective charm flares and pulls the passenger back from the edge."
                    .to_string(),
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
        }

        !Self::is_passenger_meltdown(state)
    }

    /// Check if guideline decision should be triggered
    fn check_guideline_triggers(state: &mut GameState, current_time: f64) -> Option<RouteOutcome> {
        let should_check = !state.current_guidelines.is_empty()
            && !state.detected_tells.is_empty()
            && state.driving_phase == Some(DrivingPhase::Destination);

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

    /// Generate a mid-ride event by drawing from the authored event deck.
    ///
    /// Picks a route-eligible template (weighted), then optionally appends a
    /// passenger-specific "use your ability" choice when the player has both the
    /// almanac knowledge and the matching skill unlocked, and shuffles.
    fn generate_mid_ride_event(
        state: &GameState,
        data: &GameData,
        stats: &PlayerStats,
        route: RouteType,
    ) -> MidRideEvent {
        let eligible: Vec<&EventTemplate> = data
            .events
            .iter()
            .filter(|e| e.eligible_for(route))
            .collect();

        let (title, description, mut choices) = match Self::pick_weighted_event(&eligible) {
            Some(tpl) => (
                tpl.title.clone(),
                tpl.description.clone(),
                tpl.choices.clone(),
            ),
            None => Self::fallback_event(),
        };

        // Passenger ability choice: strongest option, gated behind almanac + skill.
        if let Some(p) = &state.current_passenger {
            let almanac_unlocked = stats.get_almanac_entry(p.id).knowledge_level >= 1;
            if let Some(trait_name) = p.traits.first() {
                let skill_id = trait_name.to_lowercase().replace(' ', "_");
                if almanac_unlocked && stats.is_skill_unlocked(&skill_id) {
                    choices.push(EventChoice {
                        description: format!("Use your {} to steady the moment", trait_name),
                        risk_type: RiskTag::SpiritualDisturbance,
                        consequence: EventConsequence::Stress(-10),
                        required_trait: Some(trait_name.clone()),
                    });
                }
            }
        }

        macroquad_toolkit::rng::shuffle(&mut choices);

        MidRideEvent {
            title,
            description,
            choices,
        }
    }

    /// Pick one event weighted by its `weight` field.
    fn pick_weighted_event<'a>(events: &[&'a EventTemplate]) -> Option<&'a EventTemplate> {
        if events.is_empty() {
            return None;
        }
        let total: f32 = events.iter().map(|e| e.weight.max(0.0)).sum();
        if total <= 0.0 {
            return macroquad_toolkit::rng::choose(events).copied();
        }
        let mut roll = macroquad_toolkit::rng::rand() * total;
        for e in events {
            roll -= e.weight.max(0.0);
            if roll <= 0.0 {
                return Some(*e);
            }
        }
        events.last().copied()
    }

    /// Generic event used only if the deck is empty (e.g. data failed to load).
    fn fallback_event() -> (String, String, Vec<EventChoice>) {
        (
            "Unsettling Atmosphere".to_string(),
            "The air grows heavy and silent.".to_string(),
            vec![
                EventChoice {
                    description: "Proceed carefully".to_string(),
                    risk_type: RiskTag::DenseFog,
                    consequence: EventConsequence::Fuel(3.0),
                    required_trait: None,
                },
                EventChoice {
                    description: "Speed through it".to_string(),
                    risk_type: RiskTag::StrangeNoises,
                    consequence: EventConsequence::Risk(2),
                    required_trait: None,
                },
                EventChoice {
                    description: "Take a detour".to_string(),
                    risk_type: RiskTag::RoadConstruction,
                    consequence: EventConsequence::Time(8),
                    required_trait: None,
                },
            ],
        )
    }

    /// Resolve a mid-ride event choice, applying its consequence and returning
    /// to route selection for the second leg of the journey.
    pub fn resolve_event_choice(state: &mut GameState, choice_index: usize) {
        if let Some(event) = &state.current_event {
            if let Some(choice) = event.choices.get(choice_index) {
                match choice.consequence {
                    EventConsequence::Fuel(amount) => {
                        state.fuel = (state.fuel - amount).max(0.0);
                    }
                    EventConsequence::Time(amount) => {
                        state.time_remaining = state.time_remaining.saturating_sub(amount);
                    }
                    EventConsequence::Risk(amount) => {
                        Self::apply_event_stress(state, amount * 8);
                    }
                    EventConsequence::Stress(amount) => {
                        Self::apply_event_stress(state, amount);
                    }
                    EventConsequence::None => {}
                }
            }
        }

        // Return to Driving (Pickup) to present route options for the second leg.
        // transition_driving_phase completes the ride after the second leg.
        state.game_phase = GamePhase::Driving;
        state.driving_phase = Some(DrivingPhase::Pickup);
    }

    /// Apply a stress delta from an event choice to the current passenger's need
    /// state, merging any tells the change triggers.
    fn apply_event_stress(state: &mut GameState, amount: i32) {
        if let (Some(mut need), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let now = macroquad::prelude::get_time();
            let triggered =
                PassengerStateMachine::apply_stress_delta(&mut need, &passenger, amount, now);
            state.current_passenger_need_state = Some(need);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                passenger.id,
                now,
            );
        }
    }
}

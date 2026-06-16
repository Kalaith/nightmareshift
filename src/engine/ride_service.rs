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
    pub fn accept_ride(state: &mut GameState, data: &GameData) -> Result<(), String> {
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

        // Print route options for the first leg
        Self::debug_route_options(state, data);
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
            let fare = GameEngine::calculate_fare(
                passenger.fare,
                route,
                passenger,
                state.consecutive_route_streak.as_ref(),
                reputation,
                &data.constants,
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
                    state.current_event = Some(Self::generate_mid_ride_event(state, stats, route));
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

    /// Generate a random mid-ride event
    fn generate_mid_ride_event(
        state: &GameState,
        stats: &PlayerStats,
        route: RouteType,
    ) -> MidRideEvent {
        // let mut rng = macroquad_toolkit::rng::rand();

        // 1. Determine potential risks based on route costs (which has the risk tags)
        // We need to re-calculate or retrieve the risk tags for this route.
        // For simplicity, we regenerate them here, but ideally we'd pass them through.
        // It's stateless so it's consistent if inputs are same, but Passenger/Weather might be.
        // Wait, route costs are calculated in choose_route.
        // We should probably store the selected route costs or just regenerate tags.
        let risk_tags = RouteService::generate_risk_tags(
            route,
            Some(&state.current_weather),
            Some(&state.time_of_day),
            state.current_passenger.as_ref(),
            None,
        );

        // 2. Base Event Templates (can be moved to a JSON file later)
        let templates = [
            (
                "Strange Obstruction",
                "Something is blocking the path ahead.",
            ),
            ("Unsettling Atmosphere", "The air grows heavy and silent."),
            ("Mechanical Noise", "The car makes a worrying sound."),
            ("Passenger Request", "The passenger leans forward abruptly."),
        ];

        // Pick a template
        let (title, desc) = *macroquad_toolkit::rng::choose(&templates).unwrap();

        // 3. Generate Choices
        let mut choices = Vec::new();

        // Strategy: Create 4 choices.
        // 1 Safe/Standard
        // 1 Risky but Fast
        // 1 Passenger Trait Related (if applicable)
        // 1 Random Risk Mitigation

        // Helper to make a choice
        let make_choice =
            |desc: &str, risk: RiskTag, cons: EventConsequence, trait_req: Option<String>| {
                EventChoice {
                    description: desc.to_string(),
                    risk_type: risk,
                    consequence: cons,
                    required_trait: trait_req,
                }
            };

        // Standard choice - safe but costs fuel (extra caution)
        let risk1 = risk_tags.first().copied().unwrap_or(RiskTag::HighTraffic);
        choices.push(make_choice(
            "Proceed carefully",
            risk1,
            EventConsequence::Fuel(3.0),
            None,
        ));

        // Risky choice - fast but increases danger
        let risk2 = risk_tags.get(1).copied().unwrap_or(RiskTag::Potholes);
        choices.push(make_choice(
            "Speed through it",
            risk2,
            EventConsequence::Risk(2),
            None,
        ));

        // Trait Choice (if passenger has one AND player has the skill unlocked)
        if let Some(p) = &state.current_passenger {
            // Check both conditions:
            // 1. Almanac knowledge level >= 1 (know about the passenger's abilities)
            // 2. Player has the skill unlocked in skill tree (can use the ability)
            let almanac_unlocked = stats.get_almanac_entry(p.id).knowledge_level >= 1;

            if let Some(trait_name) = p.traits.first() {
                // Convert trait name to skill ID format (e.g., "Night Vision" -> "night_vision")
                let skill_id = trait_name.to_lowercase().replace(' ', "_");
                let skill_unlocked = stats.is_skill_unlocked(&skill_id);

                if almanac_unlocked && skill_unlocked {
                    // Player has both knowledge AND the skill - show the ability option (BEST choice)
                    println!(
                        "[DEBUG] Generating Trait Choice for trait: {} (skill '{}' is UNLOCKED)",
                        trait_name, skill_id
                    );
                    choices.push(make_choice(
                        &format!("Utilize {} ability", trait_name),
                        RiskTag::SpiritualDisturbance,
                        EventConsequence::Stress(-10), // Positive effect - reduces stress
                        Some(trait_name.clone()),
                    ));
                } else {
                    // Skill not unlocked - "Ignore it" causes passenger stress (unresolved tension)
                    if !skill_unlocked {
                        println!(
                            "[DEBUG] Trait '{}' requires skill '{}' which is NOT UNLOCKED.",
                            trait_name, skill_id
                        );
                    }
                    if !almanac_unlocked {
                        println!(
                            "[DEBUG] Almanac knowledge for passenger {} is too low.",
                            p.id
                        );
                    }
                    choices.push(make_choice(
                        "Ignore it",
                        RiskTag::StrangeNoises,
                        EventConsequence::Stress(5),
                        None,
                    ));
                }
            } else {
                // No traits on passenger - ignoring still causes mild stress
                choices.push(make_choice(
                    "Ignore it",
                    RiskTag::StrangeNoises,
                    EventConsequence::Stress(5),
                    None,
                ));
            }
        } else {
            choices.push(make_choice(
                "Ignore it",
                RiskTag::StrangeNoises,
                EventConsequence::Stress(5),
                None,
            ));
        }

        // 4th choice - safe detour but costs time
        let risk3 = risk_tags.get(2).copied().unwrap_or(RiskTag::DenseFog);
        choices.push(make_choice(
            "Take a detour",
            risk3,
            EventConsequence::Time(8),
            None,
        ));

        // Shuffle choices so the trait one isn't always 3rd
        macroquad_toolkit::rng::shuffle(&mut choices);

        println!("[DEBUG] --- EVENT GENERATED: {} ---", title);
        println!("[DEBUG] Description: {}", desc);
        for (i, c) in choices.iter().enumerate() {
            println!(
                "[DEBUG] Option {}: \"{}\" -> Consequence: {:?}",
                i + 1,
                c.description,
                c.consequence
            );
            if let Some(trait_req) = &c.required_trait {
                println!("[DEBUG]   (Requires Trait: {})", trait_req);
            }
        }
        println!("[DEBUG] -----------------------------------");

        MidRideEvent {
            title: title.to_string(),
            description: desc.to_string(),
            choices,
        }
    }

    /// Resolve a mid-ride event choice
    pub fn resolve_event_choice(state: &mut GameState, data: &GameData, choice_index: usize) {
        if let Some(event) = &state.current_event {
            if let Some(choice) = event.choices.get(choice_index) {
                // Apply consequence
                println!("[DEBUG] Selected Choice: '{}'", choice.description);
                println!("[DEBUG] Applying Consequence: {:?}", choice.consequence);

                match choice.consequence {
                    EventConsequence::Fuel(amount) => {
                        state.fuel = (state.fuel - amount).max(0.0);
                        println!(
                            "[DEBUG] Fuel reduced by {}. New Fuel: {}",
                            amount, state.fuel
                        );
                    }
                    EventConsequence::Time(amount) => {
                        state.time_remaining = state.time_remaining.saturating_sub(amount);
                        println!(
                            "[DEBUG] Time reduced by {}. Remaining: {}",
                            amount, state.time_remaining
                        );
                    }
                    EventConsequence::Risk(amount) => {
                        if let (Some(mut need), Some(passenger)) = (
                            state.current_passenger_need_state.clone(),
                            state.current_passenger.clone(),
                        ) {
                            let old_level = need.level;
                            let triggered = PassengerStateMachine::apply_stress_delta(
                                &mut need,
                                &passenger,
                                amount * 8,
                                macroquad::prelude::get_time(),
                            );
                            state.current_passenger_need_state = Some(need);
                            PassengerStateMachine::merge_detected_tells(
                                &mut state.detected_tells,
                                triggered,
                                passenger.id,
                                macroquad::prelude::get_time(),
                            );
                            if let Some(updated) = &state.current_passenger_need_state {
                                println!(
                                    "[DEBUG] Risk Consequence: Level changed from {} to {}",
                                    old_level, updated.level
                                );
                            }
                        }
                    }
                    EventConsequence::Stress(amount) => {
                        if let (Some(mut need), Some(passenger)) = (
                            state.current_passenger_need_state.clone(),
                            state.current_passenger.clone(),
                        ) {
                            let old_level = need.level;
                            let triggered = PassengerStateMachine::apply_stress_delta(
                                &mut need,
                                &passenger,
                                amount,
                                macroquad::prelude::get_time(),
                            );
                            state.current_passenger_need_state = Some(need);
                            PassengerStateMachine::merge_detected_tells(
                                &mut state.detected_tells,
                                triggered,
                                passenger.id,
                                macroquad::prelude::get_time(),
                            );
                            if let Some(updated) = &state.current_passenger_need_state {
                                println!("[DEBUG] Stress Consequence: Level changed from {} to {} (Amount: {})", old_level, updated.level, amount);
                            }
                        }
                    }
                    EventConsequence::None => {
                        println!("[DEBUG] No consequence applied.");
                    }
                }
            }
        }

        // Transition back to Driving (Pickup) for "2nd Leg" of the journey
        // This will present route options again.
        state.game_phase = GamePhase::Driving;
        state.driving_phase = Some(DrivingPhase::Pickup);

        // Print route options for the second leg
        Self::debug_route_options(state, data);

        // NOTE: If we wanted to check if this was the last event, we could logic here,
        // but our logic in transition_driving_phase handles the "Stop after 2nd leg" part.
        // So here we ALWAYS assume "Back to Route Selection" after an event.
        // If we want multiple events, this loops correctly.

        // Clear event?
        // state.current_event = None; // Keep for history/ref? UI will show route selection so it's hidden.

        // Clear event? Or keep it for log?
        // state.current_event = None; // Keep it maybe for history, but UI might show it again if we don't clear or check phase.
        // Actually, game_phase change hides it. Keeping it allows reference.
    }

    /// Helper to print debug info about available routes
    fn debug_route_options(state: &GameState, data: &GameData) {
        println!("[DEBUG] --- AVAILABLE ROUTES ANALYSIS ---");
        let routes = [
            RouteType::Normal,
            RouteType::Shortcut,
            RouteType::Scenic,
            RouteType::Police,
        ];
        let no_mastery = std::collections::HashMap::new();
        for route in routes {
            let passenger_risk = state
                .current_passenger
                .as_ref()
                .and_then(|p| data.get_location(&p.pickup).map(|l| l.risk_level))
                .unwrap_or(1);

            let seed = state.rides_completed as u64
                + state
                    .current_passenger
                    .as_ref()
                    .map(|p| p.id as u64)
                    .unwrap_or(0)
                + (route as u64);
            let costs = RouteService::calculate_route_costs(
                route,
                &data.constants,
                passenger_risk,
                Some(&state.current_weather),
                Some(&state.time_of_day),
                &state.environmental_hazards,
                &no_mastery,
                state.current_passenger.as_ref(), // Pass passenger
            );
            let risk_tags = RouteService::generate_risk_tags(
                route,
                Some(&state.current_weather),
                Some(&state.time_of_day),
                state.current_passenger.as_ref(),
                Some(seed),
            );

            // Base Fare Modifier (approximate)
            let fare_mod = match route {
                RouteType::Scenic => "1.3x",
                RouteType::Shortcut => "1.0x (Speed)",
                RouteType::Police => "1.0x (Safe)",
                RouteType::Normal => "1.0x",
            };

            println!(
                "[DEBUG] Route {:?}: FuelCost={:.1}, TimeCost={:.1}, RiskScore={:.1}, FareMod={}, Risks={:?}, CostRisks={:?}",
                route, costs.fuel, costs.time, costs.risk, fare_mod, risk_tags, costs.risk_tags
            );
        }
        println!("[DEBUG] ---------------------------------");
    }
}

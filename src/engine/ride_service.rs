//! Service for managing the ride lifecycle (spawn, accept, decline, complete).

use crate::data::*;
use crate::state::*;
use crate::engine::{PassengerService, PassengerSelectionContext, GameEngine, ItemService, RouteService, PassengerStateMachine, RouteCosts};
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
    pub fn spawn_passenger(
        state: &mut GameState,
        data: &GameData,
        current_time: f64
    ) -> bool {
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
                PassengerNeedState::from_passenger(&p, current_time);
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

        state.game_phase = GamePhase::Driving;
        state.driving_phase = Some(DrivingPhase::Pickup);
        Ok(())
    }

    /// Decline the current ride request.
    /// Clears passenger state and returns to Waiting phase (caller should trigger spawn again if needed).
    pub fn decline_ride(state: &mut GameState) {
        state.current_passenger = None;
        state.current_passenger_dialogue = None;
        state.current_passenger_need_state = None;
        state.game_phase = GamePhase::Waiting;
    }

    /// Complete the current ride logic.
    pub fn complete_ride(
        state: &mut GameState,
        data: &GameData,
        stats: &mut PlayerStats,
        route: RouteType,
        current_time: f64
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
            let backstory_unlocked = if PassengerService::check_backstory_unlock(
                passenger.id,
                stats,
                &data.constants,
            ) {
                stats.unlock_backstory(passenger.id);
                Some((passenger.name.clone(), passenger.backstory_details.clone()))
            } else {
                None
            };

            // Record encounter
            stats.record_passenger_encounter(passenger.id);

            // Update reputation
            let is_positive = passenger.get_route_preference(route)
                .map(|p| matches!(p.preference, PreferenceLevel::Loves | PreferenceLevel::Likes))
                .unwrap_or(false);
            
            state.get_passenger_reputation(passenger.id)
                .update(is_positive, current_time, &data.constants.reputation);

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
                current_time
            ) {
                state.pending_trade = Some((trade.passenger_name.clone(), trade.offered_item.clone()));
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
        current_time: f64
    ) -> RouteOutcome {
        // Calculate route costs
        let passenger_risk = state.current_passenger.as_ref()
            .and_then(|p| data.get_location(&p.pickup).map(|l| l.risk_level))
            .unwrap_or(1);
        
        let costs = RouteService::calculate_route_costs(
            route,
            &data.constants,
            passenger_risk,
            Some(&state.current_weather),
            Some(&state.time_of_day),
            &state.environmental_hazards,
            &state.route_mastery,
            state.current_passenger.as_ref(),
        );

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

        // 4. Update passenger state machine
        Self::update_passenger_state(state, route, current_time);

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
            return Some(RouteOutcome::GameOver("Not enough fuel for this route.".to_string()));
        }

        if state.time_remaining < costs.time {
             return Some(RouteOutcome::GameOver("Not enough time for this route.".to_string()));
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
                return Some(RouteOutcome::GameOver(
                    violation.message.unwrap_or_else(|| "Rule violation".to_string())
                ));
            }

            // Check hidden rules
            let hidden_violation = GameEngine::check_rule_violation(
                &state.hidden_rules,
                "take_shortcut",
                state.current_passenger.as_ref(),
                state.current_passenger_need_state.as_ref(),
            );

            if hidden_violation.violation {
                // Reveal the hidden rule
                if let Some(violated_rule) = state.hidden_rules.iter()
                    .find(|r| r.forbids_action("take_shortcut"))
                    .cloned()
                {
                    state.revealed_hidden_rules.push(violated_rule.clone());
                    state.hidden_rules.retain(|r| r.id != violated_rule.id);
                    state.current_rules.push(violated_rule);
                }

                // Apply penalty
                state.rules_violated += 1;
                return Some(RouteOutcome::GameOver(format!(
                    "Hidden Rule Violated!\n{}",
                    hidden_violation.message.unwrap_or_else(|| "You broke an unknown rule...".to_string())
                )));
            }
        }
        None
    }

    /// Apply costs, tracking, and history
    fn apply_transit_effects(
        state: &mut GameState,
        costs: &RouteCosts,
        route: RouteType,
        current_time: f64
    ) {
        // Deduct resources
        state.fuel -= costs.fuel as f32;
        state.time_remaining = state.time_remaining.saturating_sub(costs.time);

        // Update route tracking
        state.increment_route_mastery(route);
        state.update_route_streak(route);

        // Record history
        let driving_phase = state.driving_phase.unwrap_or(DrivingPhase::Pickup);
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
    fn update_passenger_state(state: &mut GameState, route: RouteType, current_time: f64) {
        if let (Some(mut need_state), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone()
        ) {
            let triggered_tells = PassengerStateMachine::apply_route_choice(
                &mut need_state,
                &passenger,
                route,
                None, // No rule evaluation result passed here contextually
                current_time,
            );
            
            // Update state with result
            state.current_passenger_need_state = Some(need_state);

            // Store triggered tells
            for triggered in triggered_tells {
                state.detected_tells.push(DetectedTell {
                    tell: triggered.tell,
                    passenger_id: passenger.id,
                    detection_time: current_time,
                    player_noticed: false,
                    related_guideline: triggered.related_guideline_id,
                    exception_id: triggered.exception_id,
                });
            }
        }
    }

    /// Check if guideline decision should be triggered
    fn check_guideline_triggers(state: &mut GameState, current_time: f64) -> Option<RouteOutcome> {
        let should_check = !state.current_guidelines.is_empty()
            && !state.detected_tells.is_empty()
            && state.driving_phase == Some(DrivingPhase::Destination);

        if should_check {
            // Find a guideline that has detected tells
            if let Some(guideline) = state.current_guidelines.iter()
                .find(|g| state.detected_tells.iter()
                    .any(|t| t.related_guideline == Some(g.id)))
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
        current_time: f64
    ) -> RouteOutcome {
        match state.driving_phase {
            Some(DrivingPhase::Pickup) => {
                state.game_phase = GamePhase::Interaction;
                RouteOutcome::Success
            }
            Some(DrivingPhase::Destination) => {
                Self::complete_ride(state, data, stats, route, current_time);
                RouteOutcome::RideCompleted
            }
            None => RouteOutcome::Success
        }
    }
}

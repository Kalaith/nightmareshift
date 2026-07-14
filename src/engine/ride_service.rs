//! Service for managing the ride lifecycle (spawn, accept, decline, complete).
//!
//! Route selection lives in `route_choice`; the mid-ride event deck lives in
//! `events`. Both extend `RideService` with further `impl` blocks.

mod events;
mod route_choice;

use crate::data::*;
use crate::engine::{
    GameEngine, ItemService, PassengerSelectionContext, PassengerService, PassengerStateMachine,
    SkillModifiers,
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
            // Calculate fare. The destination's fareModifier and the player's
            // fare-boosting skills (Silver Tongue) both scale the payout.
            let reputation = state.passenger_reputation.get(&passenger.id);
            let skill_mods = SkillModifiers::from_unlocked(&data.skills, &stats.unlocked_skills);
            let destination_fare_modifier = data
                .get_location(&passenger.destination)
                .map(|l| l.fare_modifier)
                .unwrap_or(1.0)
                * skill_mods.fare_mult;
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
}

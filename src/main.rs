//! Nightmare Shift - A horror-themed taxi driving survival game.
//!
//! Drive supernatural passengers through the night, follow mysterious rules,
//! and try to survive until dawn.

mod data;
mod engine;
mod screens;
mod state;
mod ui;

use macroquad::prelude::*;

use data::{GameData, RouteType, PreferenceLevel, Rarity, TimePhase};
use engine::*;
use state::*;
use ui::*; // Import everything including UiAction and toolkit
use ui::{StatusBar, PassengerCard, CompletionSummary};

/// Main screen enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Loading,
    MainMenu,
    Briefing,
    Game,
    GameOver,
    Success,
    SkillTree,
    Almanac,
    Leaderboard,
}

/// Main game structure
pub struct Game {
    screen: Screen,
    game_data: Option<GameData>,
    game_state: GameState,
    player_stats: PlayerStats,
    show_rules: bool,
    show_inventory: bool,
    transition: ScreenTransition,
    particles: ParticleSystem,
    screen_shake: ScreenShake,
    last_frame_time: f64,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let current_time = get_time();
        
        // Load game data first (embedded at compile time, always succeeds)
        let game_data = GameData::load();
        
        // Try to load saved player stats
        let mut player_stats = Persistence::load().unwrap_or_else(|_| PlayerStats::new());
        player_stats.init_achievements();
        
        // Create game state using constants from loaded data
        let game_state = GameState::new(current_time, &game_data.constants.game_constants);
        
        Self {
            screen: Screen::MainMenu, // Skip loading since data is embedded
            game_data: Some(game_data),
            game_state,
            player_stats,
            show_rules: false,
            show_inventory: false,
            transition: ScreenTransition::new(),
            particles: ParticleSystem::new(),
            screen_shake: ScreenShake::new(),
            last_frame_time: current_time,
        }
    }

    /// Save player stats
    fn save_stats(&self) {
        if let Err(e) = Persistence::save(&self.player_stats) {
            eprintln!("Failed to save: {}", e);
        }
    }

    /// Start a new game
    fn start_game(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();

            // Reset game state using constants from data
            self.game_state.reset_for_new_shift(current_time, &data.constants.game_constants);

            // Generate rules
            let shift_rules = GameEngine::generate_shift_rules(
                self.player_stats.total_shifts_completed,
                &data.rules,
                &data.constants,
            );
            self.game_state.current_rules = shift_rules.visible_rules;
            self.game_state.hidden_rules = shift_rules.hidden_rules;
            self.game_state.difficulty_level = shift_rules.difficulty_level;

            // Load guidelines for tell detection
            self.game_state.current_guidelines = data.guidelines.clone();

            // Initialize weather
            let month = 10; // October by default
            self.game_state.season = WeatherService::get_current_season(month);
            self.game_state.current_weather = WeatherService::generate_initial_weather(
                &self.game_state.season,
                current_time,
            );
            self.game_state.time_of_day = WeatherService::get_time_of_day(20); // 8 PM

            self.transition.fade_in();
            self.screen = Screen::Briefing;
        }
    }

    /// Start the shift after briefing
    fn start_shift(&mut self) {
        self.game_state.game_phase = GamePhase::Waiting;
        self.transition.fade_in();
        self.screen = Screen::Game;
        // Don't auto-spawn - let player refuel first
    }

    /// Spawn a new passenger
    fn spawn_passenger(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            
            // Select passenger
            let passenger = PassengerService::select_weather_aware_passenger(
                &data.passengers,
                &self.game_state.used_passengers,
                self.game_state.difficulty_level,
                &self.game_state.current_weather,
                &self.game_state.time_of_day,
                &self.game_state.season,
                &data.constants,
            );
            
            if let Some(p) = passenger {
                self.game_state.used_passengers.push(p.id);
                self.game_state.current_passenger_need_state =
                    PassengerNeedState::from_passenger(&p, current_time);
                // Select dialogue once and store it
                self.game_state.current_passenger_dialogue = p.random_dialogue().map(|s| s.to_string());
                self.game_state.current_passenger = Some(p);
                self.game_state.game_phase = GamePhase::RideRequest;
            } else {
                // End shift if no passengers
                self.end_shift(true);
            }
        }
    }

    /// Accept current ride
    fn accept_ride(&mut self) {
        if self.game_state.fuel < 5.0 {
            self.end_shift(false);
            self.game_state.game_over_reason = Some(
                "You ran out of fuel with a passenger in the car.".to_string()
            );
            return;
        }

        self.game_state.game_phase = GamePhase::Driving;
        self.game_state.driving_phase = Some(DrivingPhase::Pickup);
    }

    /// Decline current ride
    fn decline_ride(&mut self) {
        self.game_state.current_passenger = None;
        self.game_state.current_passenger_dialogue = None;
        self.game_state.current_passenger_need_state = None;
        self.game_state.game_phase = GamePhase::Waiting;
        self.spawn_passenger();
    }

    /// Choose a route
    fn choose_route(&mut self, route: RouteType) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            
            // Extract passenger info we need before mutating state
            let passenger_id = self.game_state.current_passenger.as_ref().map(|p| p.id);
            let passenger_risk = self.game_state.current_passenger.as_ref()
                .and_then(|p| data.get_location(&p.pickup).map(|l| l.risk_level))
                .unwrap_or(1);
            
            // Calculate route costs
            let costs = RouteService::calculate_route_costs(
                route,
                &data.constants,
                passenger_risk,
                Some(&self.game_state.current_weather),
                Some(&self.game_state.time_of_day),
                &self.game_state.environmental_hazards,
                &self.game_state.route_mastery,
                self.game_state.current_passenger.as_ref(),
            );

            // Check resources
            if (self.game_state.fuel as u32) < costs.fuel {
                self.end_shift(false);
                self.game_state.game_over_reason = Some(
                    "Not enough fuel for this route.".to_string()
                );
                return;
            }

            if self.game_state.time_remaining < costs.time {
                self.end_shift(false);
                self.game_state.game_over_reason = Some(
                    "Not enough time for this route.".to_string()
                );
                return;
            }

            // Check rule violations for shortcuts
            if route == RouteType::Shortcut {
                // First check visible rules
                let violation = GameEngine::check_rule_violation(
                    &self.game_state.current_rules,
                    "take_shortcut",
                    self.game_state.current_passenger.as_ref(),
                    self.game_state.current_passenger_need_state.as_ref(),
                );

                if violation.violation {
                    self.end_shift(false);
                    self.game_state.game_over_reason = violation.message;
                    return;
                }

                // Check hidden rules - these get revealed on violation
                let hidden_violation = GameEngine::check_rule_violation(
                    &self.game_state.hidden_rules,
                    "take_shortcut",
                    self.game_state.current_passenger.as_ref(),
                    self.game_state.current_passenger_need_state.as_ref(),
                );

                if hidden_violation.violation {
                    // Reveal the hidden rule
                    if let Some(violated_rule) = self.game_state.hidden_rules.iter()
                        .find(|r| r.forbids_action("take_shortcut"))
                        .cloned()
                    {
                        self.game_state.revealed_hidden_rules.push(violated_rule.clone());
                        self.game_state.hidden_rules.retain(|r| r.id != violated_rule.id);
                        self.game_state.current_rules.push(violated_rule);
                    }

                    // Apply penalty
                    self.game_state.rules_violated += 1;
                    self.end_shift(false);
                    self.game_state.game_over_reason = Some(format!(
                        "Hidden Rule Violated!\n{}",
                        hidden_violation.message.unwrap_or_else(|| "You broke an unknown rule...".to_string())
                    ));
                    return;
                }
            }

            // Apply costs
            self.game_state.fuel -= costs.fuel as f32;
            self.game_state.time_remaining = self.game_state.time_remaining.saturating_sub(costs.time);

            // Update route tracking
            self.game_state.increment_route_mastery(route);
            self.game_state.update_route_streak(route);

            // Record history
            let driving_phase = self.game_state.driving_phase.unwrap_or(DrivingPhase::Pickup);
            self.game_state.route_history.push(RouteHistoryEntry {
                route_type: route,
                driving_phase,
                fuel_cost: costs.fuel,
                time_cost: costs.time,
                risk_level: costs.risk,
                passenger_id,
                timestamp: current_time,
            });

            // Update passenger state machine
            if let (Some(state), Some(passenger)) = (
                self.game_state.current_passenger_need_state.as_mut(),
                self.game_state.current_passenger.as_ref()
            ) {
                use crate::engine::PassengerStateMachine;
                let triggered_tells = PassengerStateMachine::apply_route_choice(
                    state,
                    passenger,
                    route,
                    None, // No rule evaluation result for now
                    current_time,
                );

                // Store triggered tells
                for triggered in triggered_tells {
                    self.game_state.detected_tells.push(DetectedTell {
                        tell: triggered.tell,
                        passenger_id: passenger.id,
                        detection_time: current_time,
                        player_noticed: false,
                        related_guideline: triggered.related_guideline_id,
                        exception_id: triggered.exception_id,
                    });
                }
            }

            // Check if we should trigger a guideline decision
            let should_check_guidelines = !self.game_state.current_guidelines.is_empty()
                && !self.game_state.detected_tells.is_empty()
                && self.game_state.driving_phase == Some(DrivingPhase::Destination);

            if should_check_guidelines {
                // Find a guideline that has detected tells
                if let Some(guideline) = self.game_state.current_guidelines.iter()
                    .find(|g| self.game_state.detected_tells.iter()
                        .any(|t| t.related_guideline == Some(g.id)))
                    .cloned()
                {
                    // Enter guideline decision phase
                    self.game_state.active_guideline = Some(guideline);
                    self.game_state.guideline_decision_start_time = Some(current_time);
                    self.game_state.guideline_time_remaining = 30.0;
                    self.game_state.game_phase = GamePhase::GuidelineDecision;
                    return;
                }
            }

            // Progress to next phase
            match self.game_state.driving_phase {
                Some(DrivingPhase::Pickup) => {
                    self.game_state.game_phase = GamePhase::Interaction;
                }
                Some(DrivingPhase::Destination) => {
                    self.complete_ride(route);
                }
                None => {}
            }
        }
    }

    /// Continue after interaction
    fn continue_to_destination(&mut self) {
        self.game_state.game_phase = GamePhase::Driving;
        self.game_state.driving_phase = Some(DrivingPhase::Destination);
    }

    /// Complete the current ride
    fn complete_ride(&mut self, route: RouteType) {
        if let Some(ref data) = self.game_data {
            if let Some(ref passenger) = self.game_state.current_passenger.clone() {
                let current_time = get_time();

                // Calculate fare
                let reputation = self.game_state.passenger_reputation.get(&passenger.id);
                let fare = GameEngine::calculate_fare(
                    passenger.fare,
                    route,
                    passenger,
                    self.game_state.consecutive_route_streak.as_ref(),
                    reputation,
                    &data.constants,
                );

                // Add earnings
                self.game_state.earnings += fare;
                self.game_state.rides_completed += 1;

                // Check backstory unlock
                let backstory_unlocked = if PassengerService::check_backstory_unlock(
                    passenger.id,
                    &self.player_stats,
                    &data.constants,
                ) {
                    self.player_stats.unlock_backstory(passenger.id);
                    Some((passenger.name.clone(), passenger.backstory_details.clone()))
                } else {
                    None
                };

                // Record encounter
                self.player_stats.record_passenger_encounter(passenger.id);

                // Update reputation
                let is_positive = passenger.get_route_preference(route)
                    .map(|p| matches!(p.preference, PreferenceLevel::Loves | PreferenceLevel::Likes))
                    .unwrap_or(false);
                self.game_state.get_passenger_reputation(passenger.id)
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
                    self.game_state.inventory.push(drop.item);
                }

                // Check for trade offer
                if let Some(trade) = ItemService::check_trade_offer(
                    passenger,
                    &self.game_state.inventory,
                    &data.constants,
                ) {
                    self.game_state.pending_trade = Some((trade.passenger_name.clone(), trade.offered_item.clone()));
                }

                // Create completion data
                self.game_state.last_ride_completion = Some(RideCompletion {
                    passenger: passenger.clone(),
                    fare_earned: fare,
                    items_received,
                    backstory_unlocked,
                });

                self.game_state.game_phase = GamePhase::DropOff;
            }
        }
    }

    /// Continue after drop off
    fn continue_from_dropoff(&mut self) {
        self.game_state.current_passenger = None;
        self.game_state.current_passenger_dialogue = None;
        self.game_state.current_passenger_need_state = None;
        self.game_state.last_ride_completion = None;
        self.game_state.driving_phase = None;

        // Check end conditions
        if self.game_state.should_end_shift() {
            self.end_shift(self.game_state.earnings >= self.game_state.minimum_earnings);
        } else {
            // Go to Waiting phase to allow refueling
            self.game_state.game_phase = GamePhase::Waiting;
            // Don't spawn passenger immediately - let player refuel first
        }
    }

    /// Refuel to full capacity
    fn refuel_full(&mut self) {
        if let Some(ref data) = self.game_data {
            let fuel_needed = 100.0 - self.game_state.fuel;
            let cost = (fuel_needed * data.constants.fuel.cost_per_percent) as u32;

            if self.game_state.earnings >= cost {
                self.game_state.fuel = 100.0;
                self.game_state.earnings -= cost;
            }
        }
    }

    /// Refuel by 25%
    fn refuel_partial(&mut self) {
        if let Some(ref data) = self.game_data {
            let fuel_needed = 100.0 - self.game_state.fuel;
            let amount = 25.0_f32.min(fuel_needed);
            let cost = (amount * data.constants.fuel.cost_per_percent) as u32;

            if self.game_state.earnings >= cost {
                self.game_state.fuel = (self.game_state.fuel + amount).min(100.0);
                self.game_state.earnings -= cost;
            }
        }
    }

    /// Use an item from inventory
    fn use_item(&mut self, idx: usize) {
        if idx >= self.game_state.inventory.len() {
            return;
        }

        let item = &self.game_state.inventory[idx];
        if !item.can_use {
            return;
        }

        // Apply item effects
        use crate::data::ItemEffectType;
        for effect in &item.effects {
            match effect.effect_type {
                ItemEffectType::FuelBonus => {
                    let bonus = effect.value as f32;
                    self.game_state.fuel = (self.game_state.fuel + bonus).min(100.0);
                }
                ItemEffectType::TimeBonus => {
                    self.game_state.time_remaining += effect.value as u32;
                }
                ItemEffectType::RuleImmunity => {
                    self.game_state.rule_immunity_charges += effect.value as u32;
                }
                ItemEffectType::SupernaturalProtection => {
                    self.game_state.supernatural_protection += effect.value as u32;
                }
                ItemEffectType::FuelDrain => {
                    let drain = effect.value as f32;
                    self.game_state.fuel = (self.game_state.fuel - drain).max(0.0);
                }
                ItemEffectType::TimePenalty => {
                    let penalty = effect.value as u32;
                    self.game_state.time_remaining = self.game_state.time_remaining.saturating_sub(penalty);
                }
                ItemEffectType::ReputationModifier => {
                    // Applied to current passenger if exists
                    if let Some(passenger_id) = self.game_state.current_passenger.as_ref().map(|p| p.id) {
                        if let Some(rep) = self.game_state.passenger_reputation.get_mut(&passenger_id) {
                            if effect.value > 0 {
                                rep.positive_choices += effect.value.abs() as u32;
                            } else {
                                rep.negative_choices += effect.value.abs() as u32;
                            }
                        }
                    }
                }
                ItemEffectType::RuleTrigger => {
                    // Rule triggering effects (TODO: implement if needed)
                }
            }
        }

        // Remove consumable items
        if item.item_type == crate::data::ItemType::Consumable {
            self.game_state.inventory.remove(idx);
        } else {
            // Decrease durability for other usable items
            if let Some(item) = self.game_state.inventory.get_mut(idx) {
                if let Some(durability) = item.durability {
                    if durability > 0 {
                        item.durability = Some(durability - 1);
                        if durability <= 1 {
                            // Item breaks
                            self.game_state.inventory.remove(idx);
                            return; // Exit early since we removed the item
                        }
                    }
                }
            }
        }

        // Close inventory after use
        self.show_inventory = false;
    }

    /// Evaluate a guideline decision
    fn evaluate_guideline_decision(&mut self, action: GuidelineAction) {
        let current_time = get_time();

        if let (Some(guideline), Some(passenger)) = (
            self.game_state.active_guideline.clone(),
            self.game_state.current_passenger.clone()
        ) {
            // Evaluate the choice using the guideline engine
            let result = GuidelineEngine::evaluate_guideline_choice(
                &guideline,
                action,
                &passenger,
                &self.game_state,
            );

            // Record the decision
            let tells_present: Vec<_> = self.game_state.detected_tells.iter()
                .filter(|t| t.related_guideline == Some(guideline.id))
                .map(|t| t.tell.clone())
                .collect();

            self.game_state.decision_history.push(GuidelineDecision {
                guideline_id: guideline.id,
                passenger_id: passenger.id,
                action,
                was_correct: result.is_safe,
                tells_present,
                timestamp: current_time,
            });

            // Apply consequences
            for consequence in &result.consequences {
                use data::ConsequenceType;
                match consequence.consequence_type {
                    ConsequenceType::Death => {
                        use macroquad::rand::gen_range;
                        if gen_range(0.0, 1.0) < consequence.probability {
                            self.end_shift(false);
                            self.game_state.game_over_reason = Some(result.message.clone());
                            return;
                        }
                    }
                    ConsequenceType::Survival => {
                        // Player made the right choice - increase trust
                        self.game_state.player_trust = (self.game_state.player_trust + 0.1).min(1.0);
                    }
                    ConsequenceType::Reputation => {
                        // Update passenger reputation
                        let rep_change = consequence.value;
                        if let Some(rep) = self.game_state.passenger_reputation.get_mut(&passenger.id) {
                            if rep_change > 0 {
                                rep.positive_choices += rep_change.abs() as u32;
                            } else {
                                rep.negative_choices += rep_change.abs() as u32;
                            }
                        }
                    }
                    ConsequenceType::Item => {
                        // Could add item drops here in the future
                    }
                    _ => {}
                }
            }

            // Clear guideline state and continue to completion
            self.game_state.active_guideline = None;
            self.game_state.guideline_decision_start_time = None;
            self.game_state.detected_tells.clear();

            // Complete the ride
            if let Some(ride) = self.game_state.current_ride.as_ref() {
                let route = ride.route_type.unwrap_or(RouteType::Normal);
                self.complete_ride(route);
            }
        }
    }

    /// End the shift
    fn end_shift(&mut self, success: bool) {
        let earned_enough = self.game_state.earnings >= self.game_state.minimum_earnings;
        let actually_successful = success && earned_enough;

        if actually_successful {
            // Add survival bonus
            if let Some(ref data) = self.game_data {
                self.game_state.earnings += data.constants.game_constants.survival_bonus;
            }
            self.transition.fade_in();
            self.screen = Screen::Success;
        } else {
            if self.game_state.game_over_reason.is_none() {
                self.game_state.game_over_reason = Some(
                    if !earned_enough {
                        format!(
                            "You only earned ${} but needed ${}.",
                            self.game_state.earnings,
                            self.game_state.minimum_earnings
                        )
                    } else {
                        "The night shift has ended...".to_string()
                    }
                );
            }
            // Add screen shake for dramatic effect
            self.screen_shake.shake(15.0, 0.5);
            self.transition.fade_in();
            self.screen = Screen::GameOver;
        }

        // Record stats
        self.player_stats.record_shift_completion(
            self.game_state.earnings,
            self.game_state.rides_completed,
            actually_successful,
            480 - self.game_state.time_remaining,
        );

        // Generate bank balance from earnings (50% of earnings goes to bank)
        let bank_earnings = self.game_state.earnings / 2;
        self.player_stats.bank_balance += bank_earnings;

        // Generate lore fragments
        // Base: 1 per completed ride
        let mut lore_earned = self.game_state.rides_completed;
        // Bonus: 2 per unlocked backstory this shift
        let backstories_unlocked = self.game_state.used_passengers.iter()
            .filter(|id| self.player_stats.is_backstory_unlocked(**id))
            .count() as u32;
        lore_earned += backstories_unlocked * 2;
        // Difficulty bonus
        lore_earned += self.game_state.difficulty_level;
        self.player_stats.lore_fragments += lore_earned;

        // Mark all encountered passengers in almanac
        for passenger_id in &self.game_state.used_passengers {
            self.player_stats.mark_passenger_encountered(*passenger_id);
        }

        // Add leaderboard entry
        if let Some(ref data) = self.game_data {
            use chrono::Local;
            let score = self.game_state.calculate_score(&data.constants);
            let entry = LeaderboardEntry {
                score,
                date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                survived: actually_successful,
                passengers_transported: self.game_state.rides_completed,
                difficulty_level: self.game_state.difficulty_level,
                rules_violated: self.game_state.rules_violated,
            };
            self.player_stats.add_leaderboard_entry(entry);
        }

        // Check and unlock achievements
        self.player_stats.check_achievements(
            self.game_state.earnings,
            actually_successful,
            self.game_state.rules_violated,
        );

        // Auto-save after shift
        self.save_stats();
    }

    /// Return to main menu
    fn return_to_menu(&mut self) {
        self.transition.fade_in();
        self.screen = Screen::MainMenu;
        self.particles.clear();
    }

    /// Change screen with transition
    fn change_screen(&mut self, new_screen: Screen) {
        self.transition.fade_in();
        self.screen = new_screen;
        if new_screen != Screen::Game {
            self.particles.clear();
        }
    }

    /// Update game logic
    fn update(&mut self) {
        let current_time = get_time();
        let dt = (current_time - self.last_frame_time) as f32;
        self.last_frame_time = current_time;

        // Update effects
        self.transition.update(dt);
        self.screen_shake.update(dt);
        self.particles.update(dt);

        // Spawn weather particles during game
        if self.screen == Screen::Game {
            use crate::data::WeatherType;
            match self.game_state.current_weather.weather_type {
                WeatherType::Rain | WeatherType::Thunderstorm => {
                    if self.particles.count() < 100 {
                        self.particles.spawn_rain(5);
                    }
                }
                WeatherType::Snow => {
                    if self.particles.count() < 80 {
                        self.particles.spawn_snow(3);
                    }
                }
                WeatherType::Fog => {
                    if self.particles.count() < 30 {
                        self.particles.spawn_fog(1);
                    }
                }
                _ => {}
            }

            // Update guideline decision timer
            if self.game_state.game_phase == GamePhase::GuidelineDecision {
                if let Some(start_time) = self.game_state.guideline_decision_start_time {
                    let elapsed = (current_time - start_time) as f32;
                    self.game_state.guideline_time_remaining = (30.0 - elapsed).max(0.0);

                    // Time's up - force a decision (default to following the guideline)
                    if self.game_state.guideline_time_remaining <= 0.0 {
                        // We can't modify state here, so we'll handle this in the handle_action
                    }
                }
            }

            // Proactive tell detection during rides
            if matches!(self.game_state.game_phase, GamePhase::Driving | GamePhase::Interaction) {
                if let Some(passenger) = self.game_state.current_passenger.as_ref() {
                    // Analyze passenger for tells every update
                    let mut new_tells = GuidelineEngine::analyze_passenger(
                        passenger,
                        &self.game_state,
                        &self.game_state.current_guidelines,
                        current_time
                    );

                    // Introduce false tells for experienced players
                    if GuidelineEngine::should_introduce_false_tells(&self.game_state) {
                        // Generate a false tell (inverted truth)
                        if let Some(real_tell) = new_tells.first().cloned() {
                            let false_tell = DetectedTell {
                                tell: real_tell.tell.clone(),
                                passenger_id: real_tell.passenger_id,
                                detection_time: current_time,
                                player_noticed: false,
                                related_guideline: real_tell.related_guideline,
                                exception_id: real_tell.exception_id,
                            };
                            new_tells.push(false_tell);
                        }
                    }

                    // Merge new tells with existing ones (avoid duplicates)
                    for tell in new_tells {
                        if !self.game_state.detected_tells.iter().any(|t|
                            t.tell.description == tell.tell.description && t.passenger_id == tell.passenger_id
                        ) {
                            self.game_state.detected_tells.push(tell);
                        }
                    }
                }
            }

            // Dynamic weather updates
            if let Some(shift_start) = self.game_state.shift_start_time {
                self.game_state.current_weather = WeatherService::update_weather(
                    &self.game_state.current_weather,
                    &self.game_state.season,
                    current_time
                );

                // Update time of day based on elapsed time
                self.game_state.time_of_day = WeatherService::update_time_of_day(
                    shift_start,
                    current_time
                );
            }

            // Apply curse penalties periodically (every 5 seconds of real time)
            let curse_check_interval = 5.0;
            if current_time - self.last_frame_time as f64 >= curse_check_interval {
                // Clone inventory to avoid borrow checker conflict
                let inventory_snapshot = self.game_state.inventory.clone();
                ItemService::apply_curse_penalties(
                    &inventory_snapshot,
                    &mut self.game_state,
                    current_time
                );
            }

            // Apply item deterioration
            for item in &mut self.game_state.inventory {
                item.apply_deterioration(current_time);
            }

            // Remove broken items
            self.game_state.inventory.retain(|item| !item.is_broken());
        }
    }

    /// Draw the current screen
    fn draw(&self) -> UiAction {
        clear_background(Color::from_hex(0x1a1a2e));

        // Apply screen shake offset
        let (_shake_x, _shake_y) = self.screen_shake.get_offset();

        // Draw main content
        let action = match self.screen {
            Screen::Loading => self.draw_loading(),
            Screen::MainMenu => self.draw_main_menu(),
            Screen::Briefing => self.draw_briefing(),
            Screen::Game => self.draw_game(),
            Screen::GameOver => self.draw_game_over(),
            Screen::Success => self.draw_success(),
            Screen::SkillTree => self.draw_skill_tree(),
            Screen::Almanac => self.draw_almanac(),
            Screen::Leaderboard => self.draw_leaderboard(),
        };

        // Draw overlays if toggled on during game
        if self.screen == Screen::Game {
            if self.show_rules {
                self.draw_rules_panel();
            }
            if self.show_inventory {
                self.draw_inventory_modal();
            }
        }

        // Draw weather particles
        self.particles.draw();

        // Draw fog overlay for foggy weather
        if self.screen == Screen::Game {
            use crate::data::WeatherType;
            if self.game_state.current_weather.weather_type == WeatherType::Fog {
                draw_fog_overlay(0.12);
            }
        }

        // Draw glitch effect on game over
        if self.screen == Screen::GameOver {
            let glitch_intensity = (get_time() % 2.0) as f32 / 2.0;
            draw_glitch_effect(glitch_intensity * 0.5);
        }

        // Draw transition overlay (always on top)
        self.transition.draw();

        action
    }

    fn draw_loading(&self) -> UiAction {
        let text = "Loading...";
        let font_size = 32.0;
        let text_width = measure_text(text, None, font_size as u16, 1.0).width;
        draw_text(
            text,
            screen_width() / 2.0 - text_width / 2.0,
            screen_height() / 2.0,
            font_size,
            WHITE,
        );
        UiAction::None
    }

    fn draw_main_menu(&self) -> UiAction {
        // Title
        let title = "🚕 NIGHTMARE SHIFT";
        let font_size = 48.0;
        let text_width = measure_text(title, None, font_size as u16, 1.0).width;
        draw_text(
            title,
            screen_width() / 2.0 - text_width / 2.0,
            150.0,
            font_size,
            Color::from_hex(0xff6b6b),
        );

        // Subtitle
        let subtitle = "Survive the night. Follow the rules. Maybe.";
        let sub_size = 20.0;
        let sub_width = measure_text(subtitle, None, sub_size as u16, 1.0).width;
        draw_text(
            subtitle,
            screen_width() / 2.0 - sub_width / 2.0,
            190.0,
            sub_size,
            Color::from_hex(0xaaaaaa),
        );

        // Stats
        let stats_y = 250.0;
        let stats = format!(
            "Shifts Completed: {} | Total Earnings: ${} | Rides: {}",
            self.player_stats.total_shifts_completed,
            self.player_stats.total_earnings,
            self.player_stats.total_rides_completed,
        );
        let stats_width = measure_text(&stats, None, 16, 1.0).width;
        draw_text(
            &stats,
            screen_width() / 2.0 - stats_width / 2.0,
            stats_y,
            16.0,
            Color::from_hex(0x888888),
        );

        // Achievements
        let unlocked_count = self.player_stats.achievements.iter().filter(|a| a.unlocked).count();
        let total_count = self.player_stats.achievements.len();
        let achievements_text = format!("🏆 Achievements: {}/{}", unlocked_count, total_count);
        let achievements_width = measure_text(&achievements_text, None, 16, 1.0).width;
        draw_text(
            &achievements_text,
            screen_width() / 2.0 - achievements_width / 2.0,
            stats_y + 25.0,
            16.0,
            Color::from_hex(0xffd700),
        );

        // Start button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() / 2.0 - 50.0,
            300.0,
            50.0,
            "Start Shift (SPACE)"
        ) {
            return UiAction::StartGame;
        }

        // Meta-progression buttons
        let button_y = screen_height() / 2.0 + 20.0;
        let button_spacing = 60.0;

        // Skill Tree button
        if button(
            screen_width() / 2.0 - 150.0,
            button_y,
            300.0,
            50.0,
            &format!("🌳 Skill Tree (Bank: ${})", self.player_stats.bank_balance)
        ) {
            return UiAction::OpenSkillTree;
        }

        // Almanac button
        if button(
            screen_width() / 2.0 - 150.0,
            button_y + button_spacing,
            300.0,
            50.0,
            &format!("📖 Almanac (Lore: {})", self.player_stats.lore_fragments)
        ) {
            return UiAction::OpenAlmanac;
        }

        // Leaderboard button
        if button(
            screen_width() / 2.0 - 150.0,
            button_y + button_spacing * 2.0,
            300.0,
            50.0,
            "🏆 Leaderboard"
        ) {
            return UiAction::OpenLeaderboard;
        }

        UiAction::None
    }

    fn draw_briefing(&self) -> UiAction {
        // Title
        let title = "📋 SHIFT BRIEFING";
        let font_size = 36.0;
        draw_text(title, 50.0, 60.0, font_size, Color::from_hex(0xf39c12));

        // Rules
        draw_text("Tonight's Rules:", 50.0, 120.0, 24.0, WHITE);

        let mut y = 160.0;
        for (i, rule) in self.game_state.current_rules.iter().enumerate() {
            let rule_text = format!("{}. {} - {}", i + 1, rule.title, rule.description);
            draw_text(&rule_text, 70.0, y, 18.0, Color::from_hex(0xcccccc));
            y += 30.0;
        }

        // Weather
        y += 20.0;
        let weather_text = format!(
            "Weather: {} {} - {}",
            self.game_state.current_weather.icon,
            format!("{:?}", self.game_state.current_weather.weather_type),
            self.game_state.current_weather.description
        );
        draw_text(&weather_text, 50.0, y, 18.0, Color::from_hex(0x87ceeb));

        // Start hint/button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() - 80.0,
            300.0,
            50.0,
            "Begin Shift (SPACE)"
        ) {
            return UiAction::StartGame; // Reuse StartGame action for starting from briefing?
            // Actually main menu -> briefing -> game.
            // Briefing uses SPACE to start shift.
            // Let's assume StartGame maps to whatever start logic takes place.
        }
        
        UiAction::None
    }

    fn draw_game(&self) -> UiAction {
        // Status bar
        self.draw_status_bar();

        // Game content based on phase
        match self.game_state.game_phase {
            GamePhase::Waiting => self.draw_waiting(),
            GamePhase::RideRequest => self.draw_ride_request(),
            GamePhase::Driving => self.draw_driving(),
            GamePhase::Interaction => self.draw_interaction(),
            GamePhase::GuidelineDecision => self.draw_guideline_decision(),
            GamePhase::DropOff => self.draw_dropoff(),
            _ => UiAction::None,
        }
    }

    fn draw_status_bar(&self) {
        if let Some(ref data) = self.game_data {
            StatusBar::draw(&self.game_state, &data.constants);
        }
    }

    fn draw_inventory_modal(&self) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 200));

        // Panel
        let panel_w = 700.0;
        let panel_h = 550.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;
        let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

        draw_panel_bordered(panel_rect, colors::PANEL_BG, colors::ACCENT_SKY, 3.0);

        let inner = panel_rect.inset(spacing::PADDING_LG);
        let mut y = inner.y;

        // Title
        draw_text("INVENTORY", inner.x, y, fonts::SIZE_XL, colors::ACCENT_SKY);
        y += 40.0;

        // Help text
        draw_text("Press I to close", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 30.0;

        // Item count
        let count_text = format!("Items: {}", self.game_state.inventory.len());
        draw_text(&count_text, inner.x, y, fonts::SIZE_MD, colors::TEXT_PRIMARY);
        y += 35.0;

        // Draw items
        if self.game_state.inventory.is_empty() {
            draw_text("No items collected yet.", inner.x, y, fonts::SIZE_MD, colors::TEXT_MUTED);
        } else {
            for (i, item) in self.game_state.inventory.iter().enumerate() {
                // Item background
                let item_bg = if i % 2 == 0 {
                    Color::from_rgba(45, 45, 68, 255)
                } else {
                    Color::from_rgba(37, 37, 56, 255)
                };
                draw_rectangle(inner.x, y - 5.0, inner.w, 60.0, item_bg);

                // Rarity color
                let rarity_color = match item.rarity {
                    crate::data::Rarity::Common => colors::TEXT_SECONDARY,
                    crate::data::Rarity::Uncommon => colors::ACCENT_PRIMARY,
                    crate::data::Rarity::Rare => colors::ACCENT_SKY,
                    crate::data::Rarity::Legendary => colors::ACCENT_GOLD,
                };

                // Item name
                draw_text(&item.name, inner.x + 10.0, y + 15.0, fonts::SIZE_MD, rarity_color);

                // Rarity badge
                let rarity_text = format!("{:?}", item.rarity);
                draw_text(&rarity_text, inner.x + 10.0, y + 35.0, fonts::SIZE_XS, colors::TEXT_MUTED);

                // Source
                let source_text = format!("from {}", item.source);
                draw_text(&source_text, inner.x + 100.0, y + 35.0, fonts::SIZE_XS, colors::TEXT_MUTED);

                // Can use indicator
                if item.can_use {
                    let use_text = "[Click to use]";
                    draw_text(use_text, inner.x + inner.w - 120.0, y + 25.0, fonts::SIZE_SM, colors::ACCENT_PRIMARY);

                    // Clickable area for using item
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let (mx, my) = mouse_position();
                        if mx >= inner.x && mx <= inner.x + inner.w && my >= y - 5.0 && my <= y + 55.0 {
                            // Will be handled by use_item action on next frame
                        }
                    }
                }

                y += 65.0;

                // Check if we're running out of space
                if y > inner.y + panel_h - 80.0 {
                    draw_text("...", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                    break;
                }
            }
        }
    }

    fn draw_rules_panel(&self) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(0, 0, 0, 200));

        // Panel
        let panel_w = 600.0;
        let panel_h = 500.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;
        let panel_rect = UiRect::new(panel_x, panel_y, panel_w, panel_h);

        draw_panel_bordered(panel_rect, colors::PANEL_BG, colors::ACCENT_PRIMARY, 3.0);

        let inner = panel_rect.inset(spacing::PADDING_LG);
        let mut y = inner.y;

        // Title
        draw_text("CURRENT RULES", inner.x, y, fonts::SIZE_XL, colors::ACCENT_PRIMARY);
        y += 40.0;

        // Help text
        draw_text("Press R to close", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
        y += 30.0;

        // Draw rules
        for rule in &self.game_state.current_rules {
            // Rule title with difficulty color
            let difficulty_color = match rule.difficulty {
                crate::data::Difficulty::Easy => colors::FUEL_GOOD,
                crate::data::Difficulty::Medium => colors::ACCENT_WARNING,
                crate::data::Difficulty::Hard => colors::FUEL_LOW,
                crate::data::Difficulty::Expert => colors::FUEL_CRITICAL,
                crate::data::Difficulty::Nightmare => colors::ACCENT_DANGER,
            };

            draw_text(&rule.title, inner.x, y, fonts::SIZE_MD, difficulty_color);
            y += 25.0;

            // Rule description (wrapped if too long)
            let desc = if rule.description.len() > 70 {
                format!("{}...", &rule.description[..70])
            } else {
                rule.description.clone()
            };
            draw_text(&desc, inner.x + 10.0, y, fonts::SIZE_SM, colors::TEXT_SECONDARY);
            y += 30.0;

            // Check if we're running out of space
            if y > inner.y + panel_h - 80.0 {
                draw_text("...", inner.x, y, fonts::SIZE_SM, colors::TEXT_MUTED);
                break;
            }
        }
    }

    fn draw_waiting(&self) -> UiAction {
        if let Some(ref data) = self.game_data {
            let center_x = screen_width() / 2.0;
            let mut y = 120.0;

            // Title
            let text = "Looking for passengers...";
            let font_size = 24.0;
            let text_width = measure_text(text, None, font_size as u16, 1.0).width;
            draw_text(
                text,
                center_x - text_width / 2.0,
                y,
                font_size,
                colors::TEXT_SECONDARY,
            );
            y += 60.0;

            // Fuel status
            let fuel_pct = self.game_state.fuel;
            let fuel_color = get_fuel_color(fuel_pct);
            let fuel_status = if fuel_pct <= 10.0 {
                "CRITICAL"
            } else if fuel_pct <= 20.0 {
                "LOW"
            } else if fuel_pct <= 40.0 {
                "MEDIUM"
            } else {
                "GOOD"
            };

            let fuel_text = format!("⛽ Fuel: {:.0}% - {}", fuel_pct, fuel_status);
            let fuel_width = measure_text(&fuel_text, None, fonts::SIZE_LG as u16, 1.0).width;
            draw_text(&fuel_text, center_x - fuel_width / 2.0, y, fonts::SIZE_LG, fuel_color);
            y += 50.0;

            // Inventory hint
            let inv_hint = format!("Press I for Inventory ({}  items)", self.game_state.inventory.len());
            let inv_width = measure_text(&inv_hint, None, fonts::SIZE_SM as u16, 1.0).width;
            draw_text(&inv_hint, center_x - inv_width / 2.0, y, fonts::SIZE_SM, colors::TEXT_MUTED);
            y += 35.0;

            // Refuel options
            if fuel_pct < 100.0 {
                let refuel_text = "Refuel Options:";
                let refuel_width = measure_text(refuel_text, None, fonts::SIZE_MD as u16, 1.0).width;
                draw_text(refuel_text, center_x - refuel_width / 2.0, y, fonts::SIZE_MD, colors::TEXT_PRIMARY);
                y += 40.0;

                let btn_w = 200.0;
                let btn_h = 50.0;
                let spacing = 20.0;

                // Full refuel button
                let fuel_needed = 100.0 - fuel_pct;
                let full_cost = (fuel_needed * data.constants.fuel.cost_per_percent) as u32;
                let full_label = format!("Full Tank (${})", full_cost);

                let can_afford_full = self.game_state.earnings >= full_cost;
                let _full_btn_color = if can_afford_full {
                    colors::ACCENT_SKY
                } else {
                    colors::TEXT_MUTED
                };

                if can_afford_full && button(center_x - btn_w - spacing / 2.0, y, btn_w, btn_h, &full_label) {
                    return UiAction::RefuelFull;
                } else if !can_afford_full {
                    // Show disabled button
                    draw_rectangle(center_x - btn_w - spacing / 2.0, y, btn_w, btn_h, Color::from_rgba(60, 60, 80, 255));
                    let label_width = measure_text(&full_label, None, 18, 1.0).width;
                    draw_text(&full_label, center_x - btn_w - spacing / 2.0 + (btn_w - label_width) / 2.0, y + 30.0, 18.0, colors::TEXT_MUTED);
                }

                // Partial refuel button
                let partial_amount = 25.0_f32.min(fuel_needed);
                let partial_cost = (partial_amount * data.constants.fuel.cost_per_percent) as u32;
                let partial_label = format!("+25% (${})", partial_cost);

                let can_afford_partial = self.game_state.earnings >= partial_cost;

                if can_afford_partial && button(center_x + spacing / 2.0, y, btn_w, btn_h, &partial_label) {
                    return UiAction::RefuelPartial;
                } else if !can_afford_partial {
                    // Show disabled button
                    draw_rectangle(center_x + spacing / 2.0, y, btn_w, btn_h, Color::from_rgba(60, 60, 80, 255));
                    let label_width = measure_text(&partial_label, None, 18, 1.0).width;
                    draw_text(&partial_label, center_x + spacing / 2.0 + (btn_w - label_width) / 2.0, y + 30.0, 18.0, colors::TEXT_MUTED);
                }

                y += 70.0;
            }

            // Find passenger button
            y += 20.0;
            let find_label = "Find Passenger (SPACE)";
            let find_width = 300.0;
            let find_height = 60.0;
            if button(center_x - find_width / 2.0, y, find_width, find_height, find_label) {
                return UiAction::Continue; // We'll use Continue action to spawn passenger
            }

            UiAction::None
        } else {
            UiAction::None
        }
    }

    fn draw_ride_request(&self) -> UiAction {
        if let Some(ref passenger) = self.game_state.current_passenger {
            let rect = UiRect::centered_x(80.0, 420.0, 340.0);
            return PassengerCard::draw(passenger, rect, true, self.game_state.current_passenger_dialogue.as_ref());
        }
        UiAction::None
    }

    fn draw_driving(&self) -> UiAction {
        if let Some(ref _data) = self.game_data {
            let center_x = screen_width() / 2.0;
            let y = 80.0;

            // Phase indicator
            let phase_text = match self.game_state.driving_phase {
                Some(DrivingPhase::Pickup) => "Driving to pickup...",
                Some(DrivingPhase::Destination) => "Driving to destination...",
                None => "Driving...",
            };
            let phase_width = measure_text(phase_text, None, 24, 1.0).width;
            draw_text(
                phase_text,
                center_x - phase_width / 2.0,
                y,
                24.0,
                WHITE,
            );

            // Route options
            let routes = [
                (RouteType::Normal, "Normal Route", "Safe and reliable", "[1]"),
                (RouteType::Shortcut, "Shortcut", "Faster, riskier", "[2]"),
                (RouteType::Scenic, "Scenic Route", "+30% fare bonus", "[3]"),
                (RouteType::Police, "Police Route", "Safest option", "[4]"),
            ];

            // Get passenger for preferences
            let passenger_opt = self.game_state.current_passenger.as_ref();

            let mut route_y = y + 60.0;
            for (i, (route_type, name, desc, key)) in routes.iter().enumerate() {
                // Check if route is blocked by environmental hazards
                let is_blocked = self.game_state.environmental_hazards.iter()
                    .any(|h| h.blocks_route(*route_type));

                // Check weather warnings
                let mut weather_warning = String::new();
                if matches!(self.game_state.current_weather.intensity, data::WeatherIntensity::Heavy)
                    && *route_type == RouteType::Shortcut {
                    weather_warning = format!("⚠️ {:?} weather!", self.game_state.current_weather.weather_type);
                }
                if matches!(self.game_state.time_of_day.phase, TimePhase::Night | TimePhase::Latenight)
                    && *route_type == RouteType::Shortcut {
                    if !weather_warning.is_empty() {
                        weather_warning.push_str(" Night!");
                    } else {
                        weather_warning = "⚠️ Night driving!".to_string();
                    }
                }

                // Button logic for route selection (disabled if blocked)
                if !is_blocked && button(center_x - 200.0, route_y, 400.0, 100.0, "") {
                    return UiAction::SelectRoute(i);
                }

                // Get passenger preference for this route
                let preference = passenger_opt.and_then(|p| p.get_route_preference(*route_type));

                // Background color based on preference or blocked status
                let bg_color = if is_blocked {
                    Color::from_rgba(60, 30, 30, 255) // Dark red for blocked
                } else if let Some(pref) = preference {
                    match pref.preference {
                        PreferenceLevel::Loves => Color::from_rgba(50, 100, 50, 255), // Dark green
                        PreferenceLevel::Likes => Color::from_rgba(50, 80, 60, 255),  // Light green tint
                        PreferenceLevel::Neutral => if i % 2 == 0 {
                            Color::from_hex(0x2d2d44)
                        } else {
                            Color::from_hex(0x252538)
                        },
                        PreferenceLevel::Dislikes => Color::from_rgba(80, 60, 50, 255), // Orange tint
                        PreferenceLevel::Fears => Color::from_rgba(100, 50, 50, 255),   // Dark red
                    }
                } else {
                    if i % 2 == 0 {
                        Color::from_hex(0x2d2d44)
                    } else {
                        Color::from_hex(0x252538)
                    }
                };

                draw_rectangle(
                    center_x - 200.0,
                    route_y,
                    400.0,
                    100.0,
                    bg_color,
                );

                // Show blocked overlay if route is blocked
                if is_blocked {
                    draw_rectangle(
                        center_x - 200.0,
                        route_y,
                        400.0,
                        100.0,
                        Color::from_rgba(0, 0, 0, 180),
                    );
                    draw_text("🚫 BLOCKED", center_x - 180.0, route_y + 25.0, 18.0, colors::FUEL_CRITICAL);
                    if let Some(hazard) = self.game_state.environmental_hazards.iter()
                        .find(|h| h.blocks_route(*route_type)) {
                        draw_text(&hazard.description, center_x - 180.0, route_y + 50.0, 14.0, colors::ACCENT_WARNING);
                    }
                } else {
                    draw_text(key, center_x - 180.0, route_y + 25.0, 16.0, Color::from_hex(0x4ecdc4));
                    draw_text(name, center_x - 140.0, route_y + 25.0, 18.0, WHITE);
                    draw_text(desc, center_x - 140.0, route_y + 45.0, 14.0, Color::from_hex(0x888888));

                    // Show passenger preference
                    if let Some(pref) = preference {
                        let pref_text = match pref.preference {
                            PreferenceLevel::Loves => "❤️ Loves",
                            PreferenceLevel::Likes => "👍 Likes",
                            PreferenceLevel::Neutral => "➖ Neutral",
                            PreferenceLevel::Dislikes => "👎 Dislikes",
                            PreferenceLevel::Fears => "😱 Fears",
                        };
                        let pref_color = match pref.preference {
                            PreferenceLevel::Loves => colors::FUEL_GOOD,
                            PreferenceLevel::Likes => colors::ACCENT_SKY,
                            PreferenceLevel::Neutral => colors::TEXT_MUTED,
                            PreferenceLevel::Dislikes => colors::ACCENT_WARNING,
                            PreferenceLevel::Fears => colors::FUEL_CRITICAL,
                        };
                        draw_text(pref_text, center_x - 140.0, route_y + 65.0, fonts::SIZE_SM, pref_color);
                    }

                    // Show weather warning if applicable
                    if !weather_warning.is_empty() {
                        draw_text(&weather_warning, center_x - 140.0, route_y + 85.0, fonts::SIZE_SM, colors::ACCENT_WARNING);
                    }

                    // Show route mastery if available
                    let mastery = self.game_state.get_route_mastery(*route_type);
                    if mastery >= 10 {
                        let mastery_text = format!("⭐ Master ({})", mastery);
                        draw_text(&mastery_text, center_x + 80.0, route_y + 25.0, fonts::SIZE_SM, colors::ACCENT_SKY);

                        // Show fuel reduction from mastery
                        let fuel_reduction = (mastery / 5).min(4);
                        let time_reduction = (mastery / 3).min(6);
                        if fuel_reduction > 0 || time_reduction > 0 {
                            let bonus_text = format!("-{} fuel, -{} min", fuel_reduction, time_reduction);
                            draw_text(&bonus_text, center_x + 80.0, route_y + 45.0, fonts::SIZE_XS, colors::FUEL_GOOD);
                        }
                    } else if mastery > 0 {
                        let mastery_text = format!("Used {} times", mastery);
                        draw_text(&mastery_text, center_x + 80.0, route_y + 25.0, fonts::SIZE_XS, colors::TEXT_MUTED);
                    }
                }

                route_y += 110.0;
            }
        }
        UiAction::None
    }

    fn draw_interaction(&self) -> UiAction {
        if let Some(ref passenger) = self.game_state.current_passenger {
            let rect = UiRect::centered_x(150.0, 500.0, 150.0);
            draw_panel(rect, colors::PANEL_BG);

            let inner = rect.inset(spacing::PADDING_MD);

            // Name
            draw_text(
                &passenger.name,
                inner.x,
                inner.y + 24.0,
                fonts::SIZE_LG,
                colors::ACCENT_WARNING,
            );

            // Dialogue
            if let Some(ref dialogue) = self.game_state.current_passenger_dialogue {
                let preview = if dialogue.len() > 70 {
                    format!("\"{}...\"", &dialogue[..70])
                } else {
                    format!("\"{}\"", dialogue)
                };
                draw_text(&preview, inner.x, inner.y + 60.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);
            }

            // Continue Button
            if button(
                screen_width() / 2.0 - 100.0,
                rect.bottom() + 40.0,
                200.0,
                50.0,
                "Continue (SPACE)"
            ) {
                return UiAction::Continue;
            }
        }
        UiAction::None
    }

    fn draw_dropoff(&self) -> UiAction {
        if let Some(ref completion) = self.game_state.last_ride_completion {
            let rect = UiRect::centered_x(100.0, 400.0, 240.0);
            let completion_action = CompletionSummary::draw(completion, rect);

            // Show trade offer if available
            if let Some((ref passenger_name, ref offered_item)) = self.game_state.pending_trade {
                let trade_y = rect.bottom() + 40.0;
                let trade_rect = UiRect::centered_x(trade_y, 450.0, 200.0);
                draw_panel(trade_rect, Color::from_rgba(40, 40, 60, 240));

                let inner = trade_rect.inset(spacing::PADDING_MD);

                // Title
                draw_text(
                    "💱 TRADE OFFER",
                    inner.x,
                    inner.y + 24.0,
                    fonts::SIZE_LG,
                    colors::ACCENT_SKY,
                );

                // Message
                let msg = format!("{} wants to trade!", passenger_name);
                draw_text(&msg, inner.x, inner.y + 50.0, fonts::SIZE_MD, colors::TEXT_PRIMARY);

                // Offered item
                let rarity_color = match offered_item.rarity {
                    Rarity::Common => colors::TEXT_MUTED,
                    Rarity::Uncommon => colors::ACCENT_SKY,
                    Rarity::Rare => Color::from_hex(0x87CEEB),
                    Rarity::Legendary => colors::ACCENT_WARNING,
                };
                let offer_text = format!("Offering: {}", offered_item.name);
                draw_text(&offer_text, inner.x, inner.y + 80.0, fonts::SIZE_MD, rarity_color);

                // Show what they want if available (simplified - any tradeable item)
                draw_text(
                    "For any item from your inventory",
                    inner.x,
                    inner.y + 105.0,
                    fonts::SIZE_SM,
                    colors::TEXT_MUTED,
                );

                // Buttons
                let btn_y = inner.y + 140.0;
                let btn_w = 200.0;
                let btn_h = 40.0;
                let center_x = screen_width() / 2.0;

                // Show inventory items for selection
                if !self.game_state.inventory.is_empty() {
                    draw_text(
                        "Select an item to trade:",
                        inner.x,
                        btn_y - 20.0,
                        fonts::SIZE_SM,
                        colors::TEXT_PRIMARY,
                    );

                    for (i, item) in self.game_state.inventory.iter().take(3).enumerate() {
                        if !item.can_trade {
                            continue;
                        }
                        let item_btn_y = btn_y + (i as f32 * 45.0);
                        if button(center_x - btn_w / 2.0, item_btn_y, btn_w, 35.0, &item.name) {
                            return UiAction::AcceptTrade(i);
                        }
                    }

                    // Decline button
                    let decline_y = btn_y + (150.0);
                    if button(center_x - btn_w / 2.0, decline_y, btn_w, btn_h, "Decline Trade") {
                        return UiAction::DeclineTrade;
                    }
                } else {
                    // No items to trade
                    draw_text(
                        "You have nothing to trade",
                        inner.x,
                        btn_y,
                        fonts::SIZE_SM,
                        colors::ACCENT_WARNING,
                    );
                    if button(center_x - btn_w / 2.0, btn_y + 30.0, btn_w, btn_h, "Continue") {
                        return UiAction::DeclineTrade;
                    }
                }
            } else {
                return completion_action;
            }
        }
        UiAction::None
    }

    fn draw_guideline_decision(&self) -> UiAction {
        if let Some(ref guideline) = self.game_state.active_guideline {
            let center_x = screen_width() / 2.0;
            let rect = UiRect::centered_x(100.0, 500.0, 450.0);
            draw_panel(rect, Color::from_rgba(30, 30, 50, 250));

            let inner = rect.inset(spacing::PADDING_LG);
            let mut y = inner.y;

            // Title
            draw_text(
                "👁️ GUIDELINE DECISION",
                inner.x,
                y + 28.0,
                fonts::SIZE_XL,
                colors::ACCENT_WARNING,
            );
            y += 50.0;

            // Timer with color coding
            let time_left = self.game_state.guideline_time_remaining;
            let timer_color = if time_left <= 10.0 {
                colors::FUEL_CRITICAL
            } else if time_left <= 20.0 {
                colors::ACCENT_WARNING
            } else {
                colors::FUEL_GOOD
            };
            let timer_text = format!("⏱️ Time: {:.1}s", time_left);
            draw_text(&timer_text, inner.x, y + 20.0, fonts::SIZE_LG, timer_color);
            y += 50.0;

            // Guideline info
            draw_text("Guideline:", inner.x, y + 18.0, fonts::SIZE_MD, colors::TEXT_MUTED);
            y += 25.0;
            draw_text(&guideline.title, inner.x, y + 18.0, fonts::SIZE_LG, colors::ACCENT_SKY);
            y += 35.0;

            // Description (truncated)
            let desc_preview = if guideline.description.len() > 60 {
                format!("{}...", &guideline.description[..60])
            } else {
                guideline.description.clone()
            };
            draw_text(&desc_preview, inner.x, y + 16.0, fonts::SIZE_SM, colors::TEXT_PRIMARY);
            y += 50.0;

            // Detected tells
            draw_text("Detected Tells:", inner.x, y + 18.0, fonts::SIZE_MD, colors::TEXT_MUTED);
            y += 30.0;

            let relevant_tells: Vec<_> = self.game_state.detected_tells.iter()
                .filter(|t| t.related_guideline == Some(guideline.id))
                .collect();

            if relevant_tells.is_empty() {
                draw_text("No clear tells detected", inner.x + 20.0, y + 16.0, fonts::SIZE_SM, colors::TEXT_MUTED);
                y += 25.0;
            } else {
                for (_i, tell) in relevant_tells.iter().take(3).enumerate() {
                    let intensity_text = match tell.tell.intensity {
                        data::TellIntensity::Subtle => "Subtle",
                        data::TellIntensity::Moderate => "Moderate",
                        data::TellIntensity::Obvious => "Obvious",
                    };
                    let intensity_color = match tell.tell.intensity {
                        data::TellIntensity::Subtle => colors::TEXT_MUTED,
                        data::TellIntensity::Moderate => colors::ACCENT_WARNING,
                        data::TellIntensity::Obvious => colors::FUEL_CRITICAL,
                    };

                    let tell_text = format!("• [{}] {}", intensity_text, tell.tell.description);
                    draw_text(&tell_text, inner.x + 20.0, y + 16.0, fonts::SIZE_SM, intensity_color);
                    y += 25.0;
                }
            }

            y += 30.0;

            // Decision buttons
            let btn_w = 200.0;
            let btn_h = 50.0;
            let btn_spacing = 20.0;

            // Follow guideline button (left)
            if button(center_x - btn_w - btn_spacing / 2.0, y, btn_w, btn_h, "Follow Guideline") {
                return UiAction::FollowGuideline;
            }

            // Break guideline button (right)
            if button(center_x + btn_spacing / 2.0, y, btn_w, btn_h, "Break Guideline") {
                return UiAction::BreakGuideline;
            }

            // Auto-decide if time runs out
            if time_left <= 0.0 {
                return UiAction::FollowGuideline;
            }
        }

        UiAction::None
    }

    fn draw_game_over(&self) -> UiAction {
        let center_x = screen_width() / 2.0;

        // Title
        let title = "💀 GAME OVER";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            150.0,
            title_size,
            Color::from_hex(0xff4444),
        );

        // Reason
        if let Some(ref reason) = self.game_state.game_over_reason {
            let reason_width = measure_text(reason, None, 20, 1.0).width;
            draw_text(
                reason,
                center_x - reason_width / 2.0,
                220.0,
                20.0,
                Color::from_hex(0xaaaaaa),
            );
        }

        // Stats
        let score = if let Some(ref data) = self.game_data {
            self.game_state.calculate_score(&data.constants)
        } else {
            0
        };

        let stats = format!(
            "Earnings: ${} | Rides: {} | Score: {}",
            self.game_state.earnings,
            self.game_state.rides_completed,
            score,
        );
        let stats_width = measure_text(&stats, None, 18, 1.0).width;
        draw_text(
            &stats,
            center_x - stats_width / 2.0,
            280.0,
            18.0,
            WHITE,
        );

        // Try Again Button
        if button(
            screen_width() / 2.0 - 150.0,
            screen_height() - 120.0,
            300.0,
            50.0,
            "Try Again (SPACE)"
        ) {
            return UiAction::TryAgain;
        }
        
        UiAction::None
    }

    fn draw_success(&self) -> UiAction {
        let center_x = screen_width() / 2.0;

        // Title
        let title = "🌅 SHIFT COMPLETE!";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            150.0,
            title_size,
            Color::from_hex(0xffd700),
        );

        // Subtitle
        let subtitle = "You survived the night!";
        let sub_width = measure_text(subtitle, None, 24, 1.0).width;
        draw_text(
            subtitle,
            center_x - sub_width / 2.0,
            200.0,
            24.0,
            Color::from_hex(0x4ecdc4),
        );

        // Stats
        let y = 260.0;
        draw_text(
            &format!("Total Earnings: ${}", self.game_state.earnings),
            center_x - 150.0,
            y,
            20.0,
            Color::from_hex(0xffd700),
        );
        draw_text(
            &format!("Rides Completed: {}", self.game_state.rides_completed),
            center_x - 150.0,
            y + 30.0,
            20.0,
            WHITE,
        );

        if let Some(ref data) = self.game_data {
            draw_text(
                &format!("Survival Bonus: +${}", data.constants.scoring.survival_bonus),
                center_x - 150.0,
                y + 60.0,
                20.0,
                Color::from_hex(0x44ff44),
            );
            draw_text(
                &format!("Final Score: {}", self.game_state.calculate_score(&data.constants)),
                center_x - 150.0,
                y + 100.0,
                24.0,
                Color::from_hex(0xff6b6b),
            );
        }

        // Continue Button
        if button(
             center_x - 100.0,
             screen_height() - 100.0,
             200.0,
             40.0,
             "Continue"
        ) {
            return UiAction::ReturnToMenu;
        }
        
        UiAction::None
    }

    fn draw_skill_tree(&self) -> UiAction {
        let center_x = screen_width() / 2.0;

        // Title
        let title = "🌳 SKILL TREE";
        let title_size = 36.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            Color::from_hex(0x4ecdc4),
        );

        // Bank balance
        let balance = format!("Bank Balance: ${}", self.player_stats.bank_balance);
        let balance_width = measure_text(&balance, None, 20, 1.0).width;
        draw_text(
            &balance,
            center_x - balance_width / 2.0,
            100.0,
            20.0,
            Color::from_hex(0xffd700),
        );

        if let Some(ref data) = self.game_data {
            let mut y = 150.0;
            let categories = vec!["survival", "occult", "efficiency"];
            let category_names = vec!["🛡️ Survival", "👁️ Occult", "💰 Efficiency"];

            for (cat_idx, category) in categories.iter().enumerate() {
                // Category header
                draw_text(
                    category_names[cat_idx],
                    50.0,
                    y,
                    24.0,
                    Color::from_hex(0xf39c12),
                );
                y += 35.0;

                // Skills in category
                for skill in data.skills.iter().filter(|s| &s.category == category) {
                    let is_unlocked = self.player_stats.is_skill_unlocked(&skill.id);
                    let can_unlock = skill.can_unlock(&self.player_stats.unlocked_skills)
                        && !is_unlocked
                        && self.player_stats.bank_balance >= skill.cost;

                    let color = if is_unlocked {
                        Color::from_hex(0x44ff44) // Green - unlocked
                    } else if can_unlock {
                        Color::from_hex(0x4ecdc4) // Cyan - can afford
                    } else {
                        Color::from_hex(0x888888) // Gray - locked
                    };

                    let status = if is_unlocked {
                        "✓ UNLOCKED"
                    } else {
                        ""
                    };

                    let text = format!("{} {} - ${} {}", skill.icon, skill.name, skill.cost, status);
                    draw_text(&text, 70.0, y, 18.0, color);
                    y += 25.0;

                    // Description
                    draw_text(&skill.description, 90.0, y, 14.0, Color::from_hex(0xaaaaaa));
                    y += 25.0;

                    // Purchase button if can unlock
                    if can_unlock {
                        if button(90.0, y - 20.0, 150.0, 30.0, "Purchase") {
                            return UiAction::PurchaseSkill(skill.id.clone());
                        }
                    }
                }

                y += 15.0;
            }
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            "Back to Menu (ESC)"
        ) {
            return UiAction::ReturnToMenu;
        }

        UiAction::None
    }

    fn draw_almanac(&self) -> UiAction {
        let center_x = screen_width() / 2.0;

        // Title
        let title = "📖 ALMANAC";
        let title_size = 36.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            Color::from_hex(0x4ecdc4),
        );

        // Lore fragments
        let fragments = format!("Lore Fragments: {}", self.player_stats.lore_fragments);
        let fragments_width = measure_text(&fragments, None, 20, 1.0).width;
        draw_text(
            &fragments,
            center_x - fragments_width / 2.0,
            100.0,
            20.0,
            Color::from_hex(0xffd700),
        );

        if let Some(ref data) = self.game_data {
            let mut y = 150.0;

            for passenger in &data.passengers {
                let entry = self.player_stats.get_almanac_entry(passenger.id);
                let level_name = data.almanac.get_level(entry.knowledge_level)
                    .map(|l| l.name.as_str())
                    .unwrap_or("Unknown");

                let color = if entry.encountered {
                    Color::from_hex(0x4ecdc4)
                } else {
                    Color::from_hex(0x555555)
                };

                let text = format!(
                    "{} {} - Level {}: {}",
                    passenger.emoji,
                    passenger.name,
                    entry.knowledge_level,
                    level_name
                );
                draw_text(&text, 50.0, y, 18.0, color);
                y += 25.0;

                // Show upgrade button if encountered and not max level
                if entry.encountered && entry.knowledge_level < 3 {
                    let cost = data.almanac.get_upgrade_cost(entry.knowledge_level + 1);
                    let can_afford = self.player_stats.lore_fragments >= cost;

                    let button_text = format!("Upgrade (Cost: {} fragments)", cost);
                    let button_color = if can_afford {
                        Color::from_hex(0x44ff44)
                    } else {
                        Color::from_hex(0x888888)
                    };

                    if can_afford && button(70.0, y - 20.0, 200.0, 30.0, &button_text) {
                        return UiAction::UpgradeAlmanacKnowledge(passenger.id);
                    } else if !can_afford {
                        draw_text(&button_text, 70.0, y, 14.0, button_color);
                        y += 20.0;
                    }
                }

                y += 10.0;

                // Scroll handling - simple pagination
                if y > screen_height() - 100.0 {
                    break;
                }
            }
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            "Back to Menu (ESC)"
        ) {
            return UiAction::ReturnToMenu;
        }

        UiAction::None
    }

    fn draw_leaderboard(&self) -> UiAction {
        let center_x = screen_width() / 2.0;

        // Title
        let title = "🏆 LEADERBOARD & ACHIEVEMENTS";
        let title_size = 32.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            center_x - title_width / 2.0,
            60.0,
            title_size,
            Color::from_hex(0x4ecdc4),
        );

        // Leaderboard section
        let subtitle = "Top 10 Best Runs";
        draw_text(
            subtitle,
            50.0,
            110.0,
            20.0,
            Color::from_hex(0xf39c12),
        );

        let mut y = 140.0;

        if self.player_stats.leaderboard.is_empty() {
            let msg = "No completed shifts yet. Finish a shift to see your scores!";
            let msg_width = measure_text(msg, None, 18, 1.0).width;
            draw_text(
                msg,
                center_x - msg_width / 2.0,
                y + 50.0,
                18.0,
                Color::from_hex(0x888888),
            );
        } else {
            for (idx, entry) in self.player_stats.leaderboard.iter().enumerate() {
                let rank_color = match idx {
                    0 => Color::from_hex(0xffd700), // Gold
                    1 => Color::from_hex(0xc0c0c0), // Silver
                    2 => Color::from_hex(0xcd7f32), // Bronze
                    _ => WHITE,
                };

                let status_icon = if entry.survived { "✓" } else { "✗" };
                let status_color = if entry.survived {
                    Color::from_hex(0x44ff44)
                } else {
                    Color::from_hex(0xff4444)
                };

                // Rank and score
                let rank_text = format!("#{} Score: {}", idx + 1, entry.score);
                draw_text(&rank_text, 50.0, y, 20.0, rank_color);

                // Status
                draw_text(status_icon, 250.0, y, 20.0, status_color);

                y += 25.0;

                // Details
                let details = format!(
                    "  {} passengers | Difficulty {} | {} rule violations | {}",
                    entry.passengers_transported,
                    entry.difficulty_level,
                    entry.rules_violated,
                    entry.date
                );
                draw_text(&details, 70.0, y, 14.0, Color::from_hex(0xaaaaaa));

                y += 30.0;
            }
        }

        // Achievements section
        let achievements_x = screen_width() / 2.0 + 50.0;
        let mut achievements_y = 110.0;

        draw_text(
            "Achievements",
            achievements_x,
            achievements_y,
            20.0,
            Color::from_hex(0xf39c12),
        );
        achievements_y += 30.0;

        for achievement in &self.player_stats.achievements {
            let color = if achievement.unlocked {
                Color::from_hex(0x44ff44)
            } else {
                Color::from_hex(0x555555)
            };

            let status = if achievement.unlocked { "✓" } else { "✗" };
            let text = format!("{} {}", status, achievement.name);
            draw_text(&text, achievements_x, achievements_y, 16.0, color);
            achievements_y += 20.0;

            // Description
            draw_text(&achievement.description, achievements_x + 15.0, achievements_y, 12.0, Color::from_hex(0x888888));
            achievements_y += 25.0;
        }

        // Back button
        if button(
            center_x - 100.0,
            screen_height() - 60.0,
            200.0,
            40.0,
            "Back to Menu (ESC)"
        ) {
            return UiAction::ReturnToMenu;
        }

        UiAction::None
    }

    /// Handle input
    fn handle_input(&mut self) {
        match self.screen {
            Screen::MainMenu => {
                if is_key_pressed(KeyCode::Space) {
                    self.start_game();
                }
            }
            Screen::Briefing => {
                if is_key_pressed(KeyCode::Space) {
                    self.start_shift();
                }
            }
            Screen::Game => {
                // Toggle rules with R key (works in all game phases)
                if is_key_pressed(KeyCode::R) {
                    self.show_rules = !self.show_rules;
                }
                // Toggle inventory with I key
                if is_key_pressed(KeyCode::I) {
                    self.show_inventory = !self.show_inventory;
                }

                match self.game_state.game_phase {
                    GamePhase::Waiting => {
                        if is_key_pressed(KeyCode::Space) {
                            self.spawn_passenger();
                        }
                    }
                    GamePhase::RideRequest => {
                        if is_key_pressed(KeyCode::Space) {
                            self.accept_ride();
                        }
                        if is_key_pressed(KeyCode::Escape) {
                            self.decline_ride();
                        }
                    }
                    GamePhase::Driving => {
                        if is_key_pressed(KeyCode::Key1) {
                            self.choose_route(RouteType::Normal);
                        }
                        if is_key_pressed(KeyCode::Key2) {
                            self.choose_route(RouteType::Shortcut);
                        }
                        if is_key_pressed(KeyCode::Key3) {
                            self.choose_route(RouteType::Scenic);
                        }
                        if is_key_pressed(KeyCode::Key4) {
                            self.choose_route(RouteType::Police);
                        }
                    }
                    GamePhase::Interaction => {
                        if is_key_pressed(KeyCode::Space) {
                            self.continue_to_destination();
                        }
                    }
                    GamePhase::DropOff => {
                        if is_key_pressed(KeyCode::Space) {
                            self.continue_from_dropoff();
                        }
                    }
                    _ => {}
                }
            }
            Screen::GameOver | Screen::Success => {
                if is_key_pressed(KeyCode::Space) {
                    self.return_to_menu();
                }
            }
            Screen::SkillTree | Screen::Almanac | Screen::Leaderboard => {
                if is_key_pressed(KeyCode::Escape) {
                    self.return_to_menu();
                }
            }
            _ => {}
        }
    }

    /// Handle UI actions from draw phase
    fn handle_ui_action(&mut self, action: UiAction) {
        match action {
            UiAction::StartGame => {
                if self.screen == Screen::MainMenu {
                    self.start_game();
                } else if self.screen == Screen::Briefing {
                    self.start_shift();
                }
            }
            UiAction::AcceptRide => {
                if self.screen == Screen::Game { self.accept_ride(); }
            }
            UiAction::DeclineRide => {
                if self.screen == Screen::Game { self.decline_ride(); }
            }
            UiAction::SelectRoute(idx) => {
                 let route_type = match idx {
                     0 => RouteType::Normal,
                     1 => RouteType::Shortcut,
                     2 => RouteType::Scenic,
                     3 => RouteType::Police,
                     _ => RouteType::Normal,
                 };
                 if self.screen == Screen::Game { self.choose_route(route_type); }
            }
            UiAction::Continue => {
                if self.screen == Screen::Game {
                    match self.game_state.game_phase {
                         GamePhase::Waiting => self.spawn_passenger(),
                         GamePhase::Interaction => self.continue_to_destination(),
                         GamePhase::DropOff => self.continue_from_dropoff(),
                         _ => {},
                    }
                }
            }
            UiAction::ReturnToMenu => {
                 if self.screen == Screen::GameOver
                     || self.screen == Screen::Success
                     || self.screen == Screen::SkillTree
                     || self.screen == Screen::Almanac
                     || self.screen == Screen::Leaderboard {
                     self.return_to_menu();
                 }
            }
            UiAction::TryAgain => {
                 if self.screen == Screen::GameOver || self.screen == Screen::Success {
                     self.return_to_menu();
                     self.start_game();
                 }
            }
            UiAction::RefuelFull => {
                if self.screen == Screen::Game && self.game_state.game_phase == GamePhase::Waiting {
                    self.refuel_full();
                }
            }
            UiAction::RefuelPartial => {
                if self.screen == Screen::Game && self.game_state.game_phase == GamePhase::Waiting {
                    self.refuel_partial();
                }
            }
            UiAction::ToggleRules => {
                if self.screen == Screen::Game {
                    self.show_rules = !self.show_rules;
                }
            }
            UiAction::ToggleInventory => {
                if self.screen == Screen::Game {
                    self.show_inventory = !self.show_inventory;
                }
            }
            UiAction::UseItem(idx) => {
                if self.screen == Screen::Game && idx < self.game_state.inventory.len() {
                    self.use_item(idx);
                }
            }
            UiAction::AcceptTrade(item_idx) => {
                if let Some((_, offered_item)) = self.game_state.pending_trade.take() {
                    if item_idx < self.game_state.inventory.len() {
                        // Remove the given item
                        self.game_state.inventory.remove(item_idx);
                        // Add the received item
                        self.game_state.inventory.push(offered_item);
                    }
                }
            }
            UiAction::DeclineTrade => {
                // Clear pending trade
                self.game_state.pending_trade = None;
            }
            UiAction::FollowGuideline => {
                if self.game_state.game_phase == GamePhase::GuidelineDecision {
                    self.evaluate_guideline_decision(GuidelineAction::Follow);
                }
            }
            UiAction::BreakGuideline => {
                if self.game_state.game_phase == GamePhase::GuidelineDecision {
                    self.evaluate_guideline_decision(GuidelineAction::Break);
                }
            }
            UiAction::OpenSkillTree => {
                self.screen = Screen::SkillTree;
            }
            UiAction::OpenAlmanac => {
                self.screen = Screen::Almanac;
            }
            UiAction::OpenLeaderboard => {
                self.screen = Screen::Leaderboard;
            }
            UiAction::PurchaseSkill(skill_id) => {
                if let Some(ref data) = self.game_data {
                    if let Some(skill) = data.skills.iter().find(|s| s.id == skill_id) {
                        if self.player_stats.purchase_skill(&skill.id, skill.cost) {
                            self.save_stats();
                        }
                    }
                }
            }
            UiAction::UpgradeAlmanacKnowledge(passenger_id) => {
                if let Some(ref data) = self.game_data {
                    let current_level = self.player_stats.get_almanac_entry(passenger_id).knowledge_level;
                    let cost = data.almanac.get_upgrade_cost(current_level + 1);
                    if self.player_stats.upgrade_almanac_knowledge(passenger_id, cost) {
                        self.save_stats();
                    }
                }
            }
            UiAction::None => {}
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Nightmare Shift".to_string(),
        window_width: 800,
        window_height: 600,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        game.update();
        game.handle_input();
        let action = game.draw();
        game.handle_ui_action(action);
        next_frame().await;
    }
}

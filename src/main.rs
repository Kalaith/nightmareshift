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

use data::{GameData, RouteType, PreferenceLevel};
use engine::*;
use screens::{menu_screens, game_screens, meta_screens};
use state::*;
use ui::*;
use ui::StatusBar;

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
            self.game_state.season = WeatherService::get_current_season(layout::DEFAULT_MONTH);
            self.game_state.current_weather = WeatherService::generate_initial_weather(
                &self.game_state.season,
                current_time,
            );
            self.game_state.time_of_day = WeatherService::get_time_of_day(layout::DEFAULT_START_HOUR);

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
            let context = PassengerSelectionContext {
                difficulty_level: self.game_state.difficulty_level,
                weather: &self.game_state.current_weather,
                time_of_day: &self.game_state.time_of_day,
                season: &self.game_state.season,
                constants: &data.constants,
            };

            let passenger = PassengerService::select_weather_aware_passenger(
                &data.passengers,
                &self.game_state.used_passengers,
                &context,
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
        if self.game_state.fuel < layout::MINIMUM_FUEL_FOR_RIDE {
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

        // Draw main content - delegate to screen modules
        let action = match self.screen {
            Screen::Loading => menu_screens::draw_loading(),
            Screen::MainMenu => menu_screens::draw_main_menu(&self.player_stats),
            Screen::Briefing => menu_screens::draw_briefing(&self.game_state),
            Screen::Game => self.draw_game_phase(),
            Screen::GameOver => menu_screens::draw_game_over(&self.game_state, self.game_data.as_ref()),
            Screen::Success => menu_screens::draw_success(&self.game_state, self.game_data.as_ref()),
            Screen::SkillTree => meta_screens::draw_skill_tree(&self.player_stats, self.game_data.as_ref()),
            Screen::Almanac => meta_screens::draw_almanac(&self.player_stats, self.game_data.as_ref()),
            Screen::Leaderboard => meta_screens::draw_leaderboard(&self.player_stats),
        };

        // Draw overlays if toggled on during game
        if self.screen == Screen::Game {
            if self.show_rules {
                game_screens::draw_rules_panel(&self.game_state);
            }
            if self.show_inventory {
                game_screens::draw_inventory_modal(&self.game_state);
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

    /// Draw the current game phase (during active gameplay)
    fn draw_game_phase(&self) -> UiAction {
        // Draw status bar
        if let Some(ref data) = self.game_data {
            StatusBar::draw(&self.game_state, &data.constants);
        }
        
        // Delegate to game_screens module
        game_screens::draw_game(&self.game_state, self.game_data.as_ref())
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

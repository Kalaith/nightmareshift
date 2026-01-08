use macroquad::prelude::*;

use crate::data::{self, GameData, RouteType, PreferenceLevel, ConsequenceType};
use crate::engine::*;
use crate::screens::{menu_screens, game_screens, meta_screens, Screen};
use crate::state::*;
use crate::ui::*;
use crate::ui::{StatusBar, layout}; // Import layout explicitly just in case, or rely on ui::*

/// Main game structure
pub struct Game {
    screen: Screen,
    game_data: Option<GameData>,
    game_state: GameState,
    player_stats: PlayerStats,
    show_rules: bool,
    show_inventory: bool,
    show_pause_menu: bool,
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
            show_pause_menu: false,
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
    pub fn start_game(&mut self) {
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
    pub fn start_shift(&mut self) {
        self.game_state.game_phase = GamePhase::Waiting;
        self.transition.fade_in();
        self.screen = Screen::Game;
        // Don't auto-spawn - let player refuel first
    }

    /// Spawn a new passenger
    fn spawn_passenger(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            if !RideService::spawn_passenger(&mut self.game_state, data, current_time) {
                self.end_shift(true);
            }
        }
    }

    /// Accept current ride
    /// Accept current ride
    fn accept_ride(&mut self) {
        if let Some(ref data) = self.game_data {
            if let Err(reason) = RideService::accept_ride(&mut self.game_state, data) {
                self.end_shift(false);
                self.game_state.game_over_reason = Some(reason);
            }
        }
    }

    /// Decline current ride
    fn decline_ride(&mut self) {
        RideService::decline_ride(&mut self.game_state);
        self.spawn_passenger();
    }

    /// Choose a route
    fn choose_route(&mut self, route: RouteType) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            
            match RideService::choose_route(
                &mut self.game_state,
                data,
                &mut self.player_stats,
                route,
                current_time
            ) {
                RouteOutcome::GameOver(reason) => {
                    self.end_shift(false);
                    self.game_state.game_over_reason = Some(reason);
                }
                _ => {}
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
            let current_time = get_time();
            RideService::complete_ride(
                &mut self.game_state,
                data,
                &mut self.player_stats,
                route,
                current_time
            );
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
        if ItemService::use_item(&mut self.game_state, idx) {
            self.show_inventory = false;
        }
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
                // use data::ConsequenceType; // already imported
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
            #[cfg(not(target_arch = "wasm32"))]
            let date_str = {
                use chrono::Local;
                Local::now().format("%Y-%m-%d %H:%M").to_string()
            };
            #[cfg(target_arch = "wasm32")]
            let date_str = "Session".to_string(); // Simple fallback for WASM

            let score = self.game_state.calculate_score(&data.constants);
            let entry = LeaderboardEntry {
                score,
                date: date_str,
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
    pub fn update(&mut self) {
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
            GuidelineEngine::update_detection(&mut self.game_state, current_time);

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

            // Update items (curses, deterioration)
            ItemService::update_items(&mut self.game_state, current_time);
        }
    }

    /// Draw the current screen
    pub fn draw(&self) -> UiAction {
        clear_background(Color::from_hex(0x1a1a2e));

        // Apply screen shake offset
        let (_shake_x, _shake_y) = self.screen_shake.get_offset();

        // Draw main content - delegate to screen modules
        let action = match self.screen {
            Screen::Loading => menu_screens::draw_loading(self.game_data.as_ref()),
            Screen::MainMenu => menu_screens::draw_main_menu(&self.player_stats, self.game_data.as_ref()),
            Screen::Briefing => menu_screens::draw_briefing(&self.game_state, self.game_data.as_ref()),
            Screen::Game => self.draw_game_phase(),
            Screen::GameOver => menu_screens::draw_game_over(&self.game_state, self.game_data.as_ref()),
            Screen::Success => menu_screens::draw_success(&self.game_state, self.game_data.as_ref()),
            Screen::SkillTree => meta_screens::draw_skill_tree(&self.player_stats, self.game_data.as_ref()),
            Screen::Almanac => meta_screens::draw_almanac(&self.player_stats, self.game_data.as_ref()),
            Screen::Leaderboard => meta_screens::draw_leaderboard(&self.player_stats, self.game_data.as_ref()),
        };

        // Draw overlays if toggled on during game
        if self.screen == Screen::Game {
            let game_data_ref = self.game_data.as_ref();
            if self.show_rules {
                game_screens::draw_rules_panel(&self.game_state, game_data_ref);
            }
            if self.show_inventory {
                game_screens::draw_inventory_modal(&self.game_state, game_data_ref);
            }
        }

        // Draw weather particles
        self.particles.draw();

        // Draw atmospheric overlays during gameplay
        if self.screen == Screen::Game {
            use crate::data::WeatherType;
            use crate::engine::effects::{draw_danger_overlay, draw_tension_vignette};
            
            // Fog weather overlay
            if self.game_state.current_weather.weather_type == WeatherType::Fog {
                draw_fog_overlay(0.12);
            }
            
            // Danger overlay - based on accumulated route risk and passenger state
            // Calculate danger from route history and current passenger stress
            let route_danger = self.game_state.route_history.iter()
                .rev()
                .take(3) // Last 3 routes
                .map(|r| r.risk_level as f32)
                .sum::<f32>() / 15.0; // Normalize (max 5 risk * 3 routes = 15)
            
            // Add danger from passenger distress
            let passenger_danger = self.game_state.current_passenger_need_state.as_ref()
                .map(|ns| {
                    // High stability = low danger, low stability = high danger
                    (1.0 - ns.stability) * 0.5
                })
                .unwrap_or(0.0);
            
            let total_danger = (route_danger + passenger_danger).clamp(0.0, 1.0);
            if total_danger > 0.1 {
                draw_danger_overlay(total_danger);
            }
            
            // Tension vignette - based on passenger stress level
            let tension = self.game_state.current_passenger_need_state.as_ref()
                .map(|ns| ns.level as f32 / 100.0) // Normalize level (0-100 to 0-1)
                .unwrap_or(0.0);
            
            if tension > 0.3 {
                draw_tension_vignette((tension - 0.3) * 1.5); // Scale up after threshold
            }
        }

        // Draw glitch effect on game over
        if self.screen == Screen::GameOver {
            let glitch_intensity = (get_time() % 2.0) as f32 / 2.0;
            draw_glitch_effect(glitch_intensity * 0.5);
        }

        // Draw pause menu overlay if active
        if self.show_pause_menu && self.screen == Screen::Game {
            use macroquad_toolkit::ui::button;
            
            // Dim background
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
            
            // Pause menu panel
            let panel_w = 300.0;
            let panel_h = 200.0;
            let panel_x = (screen_width() - panel_w) / 2.0;
            let panel_y = (screen_height() - panel_h) / 2.0;
            
            draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_hex(0x2d2d44));
            draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::from_hex(0x4ecdc4));
            
            // Title
            let title = "⏸️ PAUSED";
            let title_dims = measure_text(title, None, 28, 1.0);
            draw_text(title, panel_x + (panel_w - title_dims.width) / 2.0, panel_y + 40.0, 28.0, WHITE);
            
            // Resume button
            let btn_w = 200.0;
            let btn_h = 40.0;
            let btn_x = panel_x + (panel_w - btn_w) / 2.0;
            
            if button(btn_x, panel_y + 70.0, btn_w, btn_h, "Resume (ESC)") {
                return UiAction::TogglePauseMenu;
            }
            
            // Return to Menu button
            if button(btn_x, panel_y + 120.0, btn_w, btn_h, "🏠 Return to Menu") {
                return UiAction::ReturnToMenu;
            }
        }

        // Draw transition overlay (always on top)
        self.transition.draw();

        action
    }

    /// Draw the current game phase (during active gameplay)
    fn draw_game_phase(&self) -> UiAction {
        // Draw status bar and capture any action from it
        let status_action = if let Some(ref data) = self.game_data {
            StatusBar::draw(&self.game_state, &data.constants, self.game_data.as_ref())
        } else {
            UiAction::None
        };
        
        // If status bar returned an action, use it
        if status_action != UiAction::None {
            return status_action;
        }
        
        // Delegate to game_screens module
        game_screens::draw_game(&self.game_state, self.game_data.as_ref(), &self.player_stats)
    }



    /// Handle input
    pub fn handle_input(&mut self) {
        let actions = InputService::capture_input(self.screen, self.game_state.game_phase);
        for action in actions {
            self.handle_ui_action(action);
        }
    }

    /// Handle UI actions from draw phase
    pub fn handle_ui_action(&mut self, action: UiAction) {
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
             UiAction::SelectEventChoice(idx) => {
                 if self.screen == Screen::Game {
                     if let Some(ref data) = self.game_data {
                         RideService::resolve_event_choice(&mut self.game_state, data, idx);
                     }
                 }
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
                     || self.screen == Screen::Leaderboard
                     || (self.screen == Screen::Game && self.show_pause_menu) {
                     self.show_pause_menu = false;
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
            UiAction::TogglePauseMenu => {
                if self.screen == Screen::Game {
                    self.show_pause_menu = !self.show_pause_menu;
                    // Close other overlays when opening pause menu
                    if self.show_pause_menu {
                        self.show_rules = false;
                        self.show_inventory = false;
                    }
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

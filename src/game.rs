use macroquad::prelude::*;

mod render;
mod rules;

use crate::bot::{PlaytestBot, PlaytestDirective};
use crate::data::{ActionType, GameData, RouteType, Rule, RuleType};
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;
use crate::ui::layout;
use crate::ui::*;

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
    loading_frames: u32,
    last_hazard_update: f64,
    playtest_bot: Option<PlaytestBot>,
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
        let playtest_bot = PlaytestBot::from_launch_args();
        if let Some(bot) = &playtest_bot {
            bot.apply_test_unlocks(&mut player_stats, &game_data);
        }

        // Create game state using constants from loaded data
        let game_state = GameState::new(current_time, &game_data.constants.game_constants);

        Self {
            screen: Screen::Loading,
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
            loading_frames: 0,
            last_hazard_update: current_time,
            playtest_bot,
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "briefing" => self.start_game(),
            "gameplay" => {
                self.start_game();
                self.start_shift();
            }
            _ => {
                // Default: main menu. The boot flow lands here automatically
                // after a couple of loading frames (see `update`), so no
                // seeding is needed.
            }
        }
    }

    /// Save player stats
    fn save_stats(&self) {
        if let Some(data) = &self.game_data {
            eprintln!("{}", data.localization.system.saving);
        }
        if let Err(e) = Persistence::save(&self.player_stats) {
            if let Some(data) = &self.game_data {
                eprintln!("{}: {}", data.localization.system.error, e);
            } else {
                eprintln!("Failed to save: {}", e);
            }
        }
    }

    /// Start a new game
    pub fn start_game(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            self.player_stats.session_start = Some(current_time);

            // Reset game state using constants from data
            self.game_state
                .reset_for_new_shift(current_time, &data.constants.game_constants);

            // Generate rules
            let shift_rules = GameEngine::generate_shift_rules(
                self.player_stats.total_shifts_completed,
                &data.rules,
                &data.constants,
            );
            self.game_state.current_rules = shift_rules.visible_rules;
            self.game_state.hidden_rules = shift_rules.hidden_rules;
            self.game_state.difficulty_level = shift_rules.difficulty_level;

            // Apply the player's unlocked-skill effects for this shift.
            let skill_mods =
                SkillModifiers::from_unlocked(&data.skills, &self.player_stats.unlocked_skills);
            self.game_state.max_fuel = 100.0 + skill_mods.max_fuel_bonus;
            self.game_state.supernatural_protection += skill_mods.bonus_protection;
            // Glimpse: a chance to reveal one hidden rule up front.
            if skill_mods.reveal_hidden_chance > 0.0
                && macroquad_toolkit::rng::chance(skill_mods.reveal_hidden_chance)
            {
                if let Some(rule_id) = self.game_state.hidden_rules.first().map(|r| r.id) {
                    self.game_state.reveal_hidden_rule(rule_id);
                }
            }

            // Load guidelines for tell detection
            self.game_state.current_guidelines = data.guidelines.clone();

            // Initialize weather
            self.game_state.season = WeatherService::get_current_season(layout::DEFAULT_MONTH);
            self.game_state.current_weather =
                WeatherService::generate_initial_weather(&self.game_state.season, current_time);
            self.game_state.time_of_day =
                WeatherService::get_time_of_day(layout::DEFAULT_START_HOUR);
            self.game_state.environmental_hazards = WeatherService::generate_hazards(
                &self.game_state.current_weather,
                &self.game_state.time_of_day,
                &self.game_state.season,
                current_time,
            );
            self.sync_weather_rules();
            self.last_hazard_update = current_time;

            self.transition.fade_in();
            self.game_state.game_phase = GamePhase::Briefing;
            self.screen = Screen::Briefing;
        }
    }

    /// Start the shift after briefing
    pub fn start_shift(&mut self) {
        self.game_state.game_phase = GamePhase::Waiting;
        self.game_state.current_dialogue = Some(CurrentDialogue {
            text: "Dispatch is quiet. Find a passenger when you're ready.".to_string(),
            speaker: DialogueSpeaker::Narrator,
            timestamp: get_time(),
        });
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
    fn accept_ride(&mut self) {
        if let Err(reason) = RideService::accept_ride(&mut self.game_state) {
            self.game_state.game_over_reason = Some(reason);
            self.end_shift(false);
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

            if let RouteOutcome::GameOver(reason) = RideService::choose_route(
                &mut self.game_state,
                data,
                &mut self.player_stats,
                route,
                current_time,
            ) {
                self.game_state.game_over_reason = Some(reason);
                self.end_shift(false);
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
                current_time,
            );
        }
    }

    /// Continue after drop off
    fn continue_from_dropoff(&mut self) {
        self.game_state.current_passenger = None;
        self.game_state.current_passenger_dialogue = None;
        self.game_state.current_passenger_need_state = None;
        self.game_state.current_ride = None;
        self.game_state.current_event = None;
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
            let refuel_mult =
                SkillModifiers::from_unlocked(&data.skills, &self.player_stats.unlocked_skills)
                    .refuel_cost_mult;
            let fuel_needed = self.game_state.max_fuel - self.game_state.fuel;
            let cost = (fuel_needed * data.constants.fuel.cost_per_percent * refuel_mult) as u32;

            if self.game_state.earnings >= cost {
                self.game_state.fuel = self.game_state.max_fuel;
                self.game_state.earnings -= cost;
            }
        }
    }

    /// Refuel by 25%
    fn refuel_partial(&mut self) {
        if let Some(ref data) = self.game_data {
            let refuel_mult =
                SkillModifiers::from_unlocked(&data.skills, &self.player_stats.unlocked_skills)
                    .refuel_cost_mult;
            let fuel_needed = self.game_state.max_fuel - self.game_state.fuel;
            let amount = 25.0_f32.min(fuel_needed);
            let cost = (amount * data.constants.fuel.cost_per_percent * refuel_mult) as u32;

            if self.game_state.earnings >= cost {
                self.game_state.fuel =
                    (self.game_state.fuel + amount).min(self.game_state.max_fuel);
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
            self.game_state.game_phase = GamePhase::Success;
            self.screen = Screen::Success;
        } else {
            if self.game_state.game_over_reason.is_none() {
                self.game_state.game_over_reason = Some(if !earned_enough {
                    format!(
                        "You only earned ${} but needed ${}.",
                        self.game_state.earnings, self.game_state.minimum_earnings
                    )
                } else {
                    "The night shift has ended...".to_string()
                });
            }
            // Add screen shake for dramatic effect
            self.screen_shake.shake(15.0, 0.5);
            self.transition.fade_in();
            self.game_state.game_phase = GamePhase::GameOver;
            self.screen = Screen::GameOver;
        }

        // Record stats
        let play_time = self
            .player_stats
            .session_start
            .map(|start| ((get_time() - start) / 60.0).max(0.0) as u32)
            .unwrap_or(480 - self.game_state.time_remaining);

        self.player_stats.record_shift_completion(
            self.game_state.earnings,
            self.game_state.rides_completed,
            actually_successful,
            play_time,
        );
        self.player_stats.session_start = None;

        // Generate bank balance from earnings (50% of earnings goes to bank)
        let bank_earnings = self.game_state.earnings / 2;
        self.player_stats.bank_balance += bank_earnings;

        // Generate lore fragments
        // Base: 1 per completed ride
        let mut lore_earned = self.game_state.rides_completed;
        // Bonus: 2 per unlocked backstory this shift
        let backstories_unlocked = self
            .game_state
            .used_passengers
            .iter()
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
        self.change_screen(Screen::MainMenu);
        self.particles.clear();
    }

    /// Change screen with transition
    fn change_screen(&mut self, new_screen: Screen) {
        self.transition.fade_in();
        self.screen = new_screen;
        self.game_state.game_phase = match new_screen {
            Screen::Loading => GamePhase::Loading,
            Screen::MainMenu => GamePhase::MainMenu,
            Screen::Briefing => GamePhase::Briefing,
            Screen::Game => self.game_state.game_phase,
            Screen::GameOver => GamePhase::GameOver,
            Screen::Success => GamePhase::Success,
            Screen::SkillTree => GamePhase::SkillTree,
            Screen::Almanac => GamePhase::Almanac,
            Screen::Leaderboard => GamePhase::Leaderboard,
        };
        if new_screen != Screen::Game {
            self.particles.clear();
        }
    }

    fn sync_weather_rules(&mut self) {
        let Some(data) = self.game_data.as_ref() else {
            return;
        };

        let triggered_ids = WeatherService::get_weather_triggered_rules(
            &self.game_state.current_weather,
            &self.game_state.time_of_day,
        );

        self.game_state
            .current_rules
            .retain(|rule| rule.rule_type != RuleType::Weather || triggered_ids.contains(&rule.id));
        self.game_state
            .hidden_rules
            .retain(|rule| rule.rule_type != RuleType::Weather || triggered_ids.contains(&rule.id));

        for rule_id in triggered_ids {
            if self
                .game_state
                .current_rules
                .iter()
                .chain(self.game_state.hidden_rules.iter())
                .any(|rule| rule.id == rule_id)
            {
                continue;
            }

            let Some(rule) = data.rules.iter().find(|rule| rule.id == rule_id).cloned() else {
                continue;
            };
            let rule = Self::prepare_weather_rule(rule);
            let already_revealed = self
                .game_state
                .revealed_hidden_rules
                .iter()
                .any(|revealed| revealed.id == rule.id);

            if rule.visible || already_revealed {
                self.game_state.current_rules.push(rule);
            } else {
                self.game_state.hidden_rules.push(rule);
            }
        }
    }

    fn prepare_weather_rule(mut rule: Rule) -> Rule {
        if rule.action_key.is_none() {
            rule.action_key = match rule.trigger.as_deref() {
                Some("thunderstorm") => Some("use_wipers".to_string()),
                Some("heavy_fog") => Some("drive_dark".to_string()),
                Some("snow") => Some("speed_in_snow".to_string()),
                Some("latenight_badweather") => Some("stop_vehicle".to_string()),
                Some("low_visibility") => Some("use_ac".to_string()),
                Some("heavy_wind") => Some("open_window".to_string()),
                _ => None,
            };
        }
        if rule.action_key.is_some() && rule.action_type.is_none() {
            rule.action_type = Some(ActionType::Forbidden);
        }
        rule
    }

    /// Update game logic
    pub fn update(&mut self) {
        let current_time = get_time();
        let dt = (current_time - self.last_frame_time) as f32;
        self.last_frame_time = current_time;

        if self.screen == Screen::Loading {
            self.loading_frames += 1;
            if self.loading_frames >= 2 {
                self.change_screen(Screen::MainMenu);
            }
        }

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
                WeatherType::Fog if self.particles.count() < 30 => {
                    self.particles.spawn_fog(1);
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
                    current_time,
                );

                // Update time of day based on elapsed time
                self.game_state.time_of_day =
                    WeatherService::update_time_of_day(shift_start, current_time);
                self.sync_weather_rules();

                self.game_state.environmental_hazards.retain(|hazard| {
                    current_time - hazard.start_time < hazard.duration as f64 * 60.0
                });

                if current_time - self.last_hazard_update >= 60.0 {
                    let new_hazards = WeatherService::generate_hazards(
                        &self.game_state.current_weather,
                        &self.game_state.time_of_day,
                        &self.game_state.season,
                        current_time,
                    );
                    for hazard in new_hazards {
                        if !self
                            .game_state
                            .environmental_hazards
                            .iter()
                            .any(|h| h.id == hazard.id)
                        {
                            self.game_state.environmental_hazards.push(hazard);
                        }
                    }
                    self.last_hazard_update = current_time;
                }
            }

            // Update items (curses, deterioration)
            ItemService::update_items(&mut self.game_state, current_time);
        }
    }

    /// Handle input
    pub fn handle_input(&mut self) {
        let actions = InputService::capture_input(self.screen, self.game_state.game_phase);
        for action in actions {
            self.handle_ui_action(action);
        }
    }

    /// Let the optional playtest bot drive game actions.
    pub fn handle_playtest_bot(&mut self) {
        let directive = if let Some(bot) = self.playtest_bot.as_mut() {
            bot.next_action(
                self.screen,
                &self.game_state,
                &self.player_stats,
                self.game_data.as_ref(),
                get_time(),
            )
        } else {
            PlaytestDirective::None
        };

        match directive {
            PlaytestDirective::None => {}
            PlaytestDirective::Action(action) => self.handle_ui_action(action),
            PlaytestDirective::Stop(code) => {
                #[cfg(not(target_arch = "wasm32"))]
                std::process::exit(code);
                #[cfg(target_arch = "wasm32")]
                let _ = code;
            }
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
                if self.screen == Screen::Game {
                    self.accept_ride();
                }
            }
            UiAction::DeclineRide => {
                if self.screen == Screen::Game {
                    self.decline_ride();
                }
            }
            UiAction::SelectRoute(idx) => {
                let route_type = match idx {
                    0 => RouteType::Normal,
                    1 => RouteType::Shortcut,
                    2 => RouteType::Scenic,
                    3 => RouteType::Police,
                    _ => RouteType::Normal,
                };
                if self.screen == Screen::Game {
                    self.choose_route(route_type);
                }
            }
            UiAction::SelectEventChoice(idx) => {
                if self.screen == Screen::Game {
                    RideService::resolve_event_choice(&mut self.game_state, idx);
                }
            }
            UiAction::Continue => {
                if self.screen == Screen::Game {
                    match self.game_state.game_phase {
                        GamePhase::Waiting => self.spawn_passenger(),
                        GamePhase::Interaction => self.continue_to_destination(),
                        GamePhase::DropOff => self.continue_from_dropoff(),
                        _ => {}
                    }
                }
            }
            UiAction::ReturnToMenu => {
                if self.screen == Screen::GameOver
                    || self.screen == Screen::Success
                    || self.screen == Screen::SkillTree
                    || self.screen == Screen::Almanac
                    || self.screen == Screen::Leaderboard
                    || (self.screen == Screen::Game && self.show_pause_menu)
                {
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
            UiAction::EndShift => {
                if self.screen == Screen::Game
                    && self.game_state.game_phase == GamePhase::Waiting
                    && self.game_state.earnings >= self.game_state.minimum_earnings
                {
                    self.end_shift(true);
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
            UiAction::PerformRuleAction(action_key) => {
                if self.screen == Screen::Game {
                    self.perform_rule_action(action_key);
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
                self.change_screen(Screen::SkillTree);
            }
            UiAction::OpenAlmanac => {
                self.change_screen(Screen::Almanac);
            }
            UiAction::OpenLeaderboard => {
                self.change_screen(Screen::Leaderboard);
            }
            UiAction::DeleteSave => {
                if Persistence::delete_save().is_ok() {
                    self.player_stats = PlayerStats::new();
                    self.player_stats.init_achievements();
                }
            }
            UiAction::PurchaseSkill(skill_id) => {
                if let Some(ref data) = self.game_data {
                    if let Some(skill) = data.skills.iter().find(|s| s.id == skill_id) {
                        if self.player_stats.purchase_skill(&skill.id, skill.cost) {
                            self.player_stats.check_achievements(
                                self.game_state.earnings,
                                false,
                                self.game_state.rules_violated,
                            );
                            self.save_stats();
                        }
                    }
                }
            }
            UiAction::UpgradeAlmanacKnowledge(passenger_id) => {
                if let Some(ref data) = self.game_data {
                    let current_level = self
                        .player_stats
                        .get_almanac_entry(passenger_id)
                        .knowledge_level;
                    let cost = data.almanac.get_upgrade_cost(current_level + 1);
                    if self
                        .player_stats
                        .upgrade_almanac_knowledge(passenger_id, cost)
                    {
                        self.player_stats.check_achievements(
                            self.game_state.earnings,
                            false,
                            self.game_state.rules_violated,
                        );
                        self.save_stats();
                    }
                }
            }
            UiAction::None => {}
        }
    }
}

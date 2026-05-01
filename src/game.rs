use macroquad::prelude::*;

use crate::bot::{PlaytestBot, PlaytestDirective};
use crate::data::{ActionType, Consequence, ConsequenceType, GameData, RouteType, Rule, RuleType};
use crate::engine::*;
use crate::screens::{game_screens, menu_screens, meta_screens, Screen};
use crate::state::*;
use crate::ui::*;
use crate::ui::{layout, StatusBar}; // Import layout explicitly just in case, or rely on ui::*

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
    /// Accept current ride
    fn accept_ride(&mut self) {
        if let Some(ref data) = self.game_data {
            if let Err(reason) = RideService::accept_ride(&mut self.game_state, data) {
                self.game_state.game_over_reason = Some(reason);
                self.end_shift(false);
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
                current_time,
            ) {
                RouteOutcome::GameOver(reason) => {
                    self.game_state.game_over_reason = Some(reason);
                    self.end_shift(false);
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

    fn perform_rule_action(&mut self, action_key: String) {
        if !self.can_perform_cab_action(&action_key) {
            self.game_state.current_dialogue = Some(CurrentDialogue {
                text: format!(
                    "{} is not useful right now.",
                    Self::cab_action_label(&action_key)
                ),
                speaker: DialogueSpeaker::Driver,
                timestamp: get_time(),
            });
            return;
        }

        let current_time = get_time();
        let visible = GameEngine::check_rule_violation(
            &self.game_state.current_rules,
            &action_key,
            self.game_state.current_passenger.as_ref(),
            self.game_state.current_passenger_need_state.as_ref(),
        );

        if visible.violation || visible.rule.is_some() {
            self.resolve_cab_rule_action(visible, false, &action_key, current_time);
            return;
        }

        let hidden = GameEngine::check_rule_violation(
            &self.game_state.hidden_rules,
            &action_key,
            self.game_state.current_passenger.as_ref(),
            self.game_state.current_passenger_need_state.as_ref(),
        );

        if hidden.violation || hidden.rule.is_some() {
            self.resolve_cab_rule_action(hidden, true, &action_key, current_time);
            return;
        }

        self.game_state.adjust_player_trust(0.01);
        self.game_state.current_dialogue = Some(CurrentDialogue {
            text: format!(
                "You {}. Nothing answers.",
                Self::cab_action_phrase(&action_key)
            ),
            speaker: DialogueSpeaker::Driver,
            timestamp: current_time,
        });
    }

    fn resolve_cab_rule_action(
        &mut self,
        result: RuleEvaluationResult,
        hidden: bool,
        action_key: &str,
        current_time: f64,
    ) {
        if hidden {
            if let Some(rule) = &result.rule {
                self.game_state.reveal_hidden_rule(rule.id);
            }
        }

        self.apply_rule_need_adjustment(&result, current_time);

        let rule_title = result
            .rule
            .as_ref()
            .map(|rule| rule.title.clone())
            .unwrap_or_else(|| "Rule".to_string());

        if result.violation {
            self.game_state.rules_violated += 1;
            self.game_state.adjust_player_trust(-0.1);

            let message = result
                .message
                .clone()
                .unwrap_or_else(|| "You violated a rule.".to_string());

            if self.game_state.rule_immunity_charges > 0 {
                self.game_state.rule_immunity_charges -= 1;
                self.game_state.current_dialogue = Some(CurrentDialogue {
                    text: format!("A ward absorbs the {} violation. {}", rule_title, message),
                    speaker: DialogueSpeaker::Narrator,
                    timestamp: current_time,
                });
                return;
            }

            if self.game_state.rides_completed == 0 {
                self.game_state.current_dialogue = Some(CurrentDialogue {
                    text: if hidden {
                        format!("Hidden rule revealed: {}. {}", rule_title, message)
                    } else {
                        format!("Rule pressure spikes: {}. {}", rule_title, message)
                    },
                    speaker: DialogueSpeaker::Narrator,
                    timestamp: current_time,
                });
                return;
            }

            if let Some(rule) = &result.rule {
                self.apply_rule_consequences(&rule.break_consequences, current_time);
            }

            self.game_state.game_over_reason = Some(message);
            self.end_shift(false);
        } else {
            self.game_state.adjust_player_trust(0.05);
            if let Some(rule) = &result.rule {
                self.apply_rule_consequences(&rule.exception_rewards, current_time);
            }
            self.game_state.current_dialogue = Some(CurrentDialogue {
                text: format!(
                    "{} was dangerous on paper, but the passenger needed it.",
                    Self::cab_action_label(action_key)
                ),
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
        }
    }

    fn apply_rule_need_adjustment(&mut self, result: &RuleEvaluationResult, current_time: f64) {
        if let (Some(mut need_state), Some(passenger)) = (
            self.game_state.current_passenger_need_state.clone(),
            self.game_state.current_passenger.clone(),
        ) {
            let triggered = PassengerStateMachine::apply_rule_outcome(
                &mut need_state,
                &passenger,
                result,
                current_time,
            );
            self.game_state.current_passenger_need_state = Some(need_state);
            PassengerStateMachine::merge_detected_tells(
                &mut self.game_state.detected_tells,
                triggered,
                passenger.id,
                current_time,
            );
        }
    }

    fn apply_rule_consequences(&mut self, consequences: &[Consequence], current_time: f64) {
        for consequence in consequences {
            if macroquad::rand::gen_range(0.0, 1.0) > consequence.probability.clamp(0.0, 1.0) {
                continue;
            }

            match consequence.consequence_type {
                ConsequenceType::Death => {}
                ConsequenceType::Survival => {}
                ConsequenceType::Reputation => {
                    if let Some(passenger_id) =
                        self.game_state.current_passenger.as_ref().map(|p| p.id)
                    {
                        if let Some(rep) =
                            self.game_state.passenger_reputation.get_mut(&passenger_id)
                        {
                            if consequence.value >= 0 {
                                rep.positive_choices += consequence.value as u32;
                            } else {
                                rep.negative_choices += consequence.value.unsigned_abs();
                            }
                            rep.interactions += 1;
                            rep.last_encounter = current_time;
                        }
                    }
                }
                ConsequenceType::Money => {
                    if consequence.value >= 0 {
                        self.game_state.earnings += consequence.value as u32;
                    } else {
                        self.game_state.earnings = self
                            .game_state
                            .earnings
                            .saturating_sub(consequence.value.unsigned_abs());
                    }
                }
                ConsequenceType::Fuel => {
                    self.game_state.fuel =
                        (self.game_state.fuel + consequence.value as f32).clamp(0.0, 100.0);
                }
                ConsequenceType::Time => {
                    if consequence.value >= 0 {
                        self.game_state.time_remaining += consequence.value as u32;
                    } else {
                        self.game_state.time_remaining = self
                            .game_state
                            .time_remaining
                            .saturating_sub(consequence.value.unsigned_abs());
                    }
                }
                ConsequenceType::Item => {
                    if let Some(source) = self
                        .game_state
                        .current_passenger
                        .as_ref()
                        .map(|passenger| passenger.name.clone())
                    {
                        self.game_state
                            .inventory
                            .push(crate::data::ItemDatabase::create_item(
                                "Crumpled Note",
                                &source,
                                current_time,
                            ));
                    }
                }
                ConsequenceType::StoryUnlock => {
                    if let Some(passenger_id) =
                        self.game_state.current_passenger.as_ref().map(|p| p.id)
                    {
                        self.player_stats.mark_passenger_encountered(passenger_id);
                    }
                }
            }
        }
    }

    fn can_perform_cab_action(&self, action_key: &str) -> bool {
        if self.screen != Screen::Game || self.game_state.current_passenger.is_none() {
            return false;
        }

        match action_key {
            "accept_tip" => self.game_state.game_phase == GamePhase::DropOff,
            "stop_vehicle" => matches!(
                self.game_state.game_phase,
                GamePhase::Driving | GamePhase::Interaction
            ),
            _ => matches!(
                self.game_state.game_phase,
                GamePhase::RideRequest
                    | GamePhase::Driving
                    | GamePhase::Interaction
                    | GamePhase::GuidelineDecision
                    | GamePhase::DropOff
            ),
        }
    }

    fn cab_action_label(action_key: &str) -> &'static str {
        match action_key {
            "eye_contact" => "Make Eye Contact",
            "play_music" => "Play Music",
            "accept_tip" => "Accept Tip",
            "open_window" => "Open Window",
            "use_wipers" => "Use Wipers",
            "drive_dark" => "Kill Headlights",
            "use_ac" => "Use AC",
            "stop_vehicle" => "Stop Cab",
            _ => "Cab Action",
        }
    }

    fn cab_action_phrase(action_key: &str) -> &'static str {
        match action_key {
            "eye_contact" => "meet the passenger's eyes",
            "play_music" => "turn on the radio",
            "accept_tip" => "accept the offered tip",
            "open_window" => "crack the window",
            "use_wipers" => "switch on the wipers",
            "drive_dark" => "kill the headlights",
            "use_ac" => "turn on the AC",
            "stop_vehicle" => "pull the cab over",
            _ => "try the control",
        }
    }

    /// Evaluate a guideline decision
    fn evaluate_guideline_decision(&mut self, action: GuidelineAction) {
        let current_time = get_time();

        if let (Some(guideline), Some(passenger)) = (
            self.game_state.active_guideline.clone(),
            self.game_state.current_passenger.clone(),
        ) {
            // Evaluate the choice using the guideline engine
            let result = GuidelineEngine::evaluate_guideline_choice(
                &guideline,
                action,
                &passenger,
                &self.game_state,
            );

            // Record the decision
            let tells_present: Vec<_> = self
                .game_state
                .detected_tells
                .iter()
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

            if result.is_safe {
                self.game_state.adjust_player_trust(0.08);
            } else {
                self.game_state.adjust_player_trust(-0.12);
            }

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
                        self.game_state.player_trust =
                            (self.game_state.player_trust + 0.1).min(1.0);
                    }
                    ConsequenceType::Reputation => {
                        // Update passenger reputation
                        let rep_change = consequence.value;
                        if let Some(rep) =
                            self.game_state.passenger_reputation.get_mut(&passenger.id)
                        {
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

            let completion_route = self
                .game_state
                .current_ride
                .as_ref()
                .and_then(|ride| ride.route_type)
                .or_else(|| {
                    self.game_state
                        .route_history
                        .last()
                        .map(|entry| entry.route_type)
                })
                .unwrap_or(RouteType::Normal);
            self.complete_ride(completion_route);
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

    /// Draw the current screen
    pub fn draw(&self) -> UiAction {
        clear_background(Color::from_hex(0x1a1a2e));

        // Apply screen shake offset
        let (_shake_x, _shake_y) = self.screen_shake.get_offset();

        // Draw main content - delegate to screen modules
        let action = match self.screen {
            Screen::Loading => menu_screens::draw_loading(self.game_data.as_ref()),
            Screen::MainMenu => {
                menu_screens::draw_main_menu(&self.player_stats, self.game_data.as_ref())
            }
            Screen::Briefing => {
                menu_screens::draw_briefing(&self.game_state, self.game_data.as_ref())
            }
            Screen::Game => self.draw_game_phase(),
            Screen::GameOver => {
                menu_screens::draw_game_over(&self.game_state, self.game_data.as_ref())
            }
            Screen::Success => {
                menu_screens::draw_success(&self.game_state, self.game_data.as_ref())
            }
            Screen::SkillTree => {
                meta_screens::draw_skill_tree(&self.player_stats, self.game_data.as_ref())
            }
            Screen::Almanac => {
                meta_screens::draw_almanac(&self.player_stats, self.game_data.as_ref())
            }
            Screen::Leaderboard => {
                meta_screens::draw_leaderboard(&self.player_stats, self.game_data.as_ref())
            }
        };

        // Draw overlays if toggled on during game
        if self.screen == Screen::Game {
            let game_data_ref = self.game_data.as_ref();
            if self.show_rules {
                let rules_action = game_screens::draw_rules_panel(&self.game_state, game_data_ref);
                if rules_action != UiAction::None {
                    return rules_action;
                }
            }
            if self.show_inventory {
                let inventory_action =
                    game_screens::draw_inventory_modal(&self.game_state, game_data_ref);
                if inventory_action != UiAction::None {
                    return inventory_action;
                }
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
            let route_danger = self
                .game_state
                .route_history
                .iter()
                .rev()
                .take(3) // Last 3 routes
                .map(|r| r.risk_level as f32)
                .sum::<f32>()
                / 15.0; // Normalize (max 5 risk * 3 routes = 15)

            // Add danger from passenger distress
            let passenger_danger = self
                .game_state
                .current_passenger_need_state
                .as_ref()
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
            let tension = self
                .game_state
                .current_passenger_need_state
                .as_ref()
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
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.72),
            );
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.02, 0.025, 0.030, 0.18),
            );

            let panel = UiRect::centered_x(
                (screen_height() - 360.0) / 2.0,
                screen_width().min(520.0),
                360.0,
            );
            draw_glass_panel(panel, colors::BORDER);
            let inner = panel.inset(spacing::PADDING_LG);

            draw_text(
                "PAUSED",
                inner.x,
                inner.y + 36.0,
                fonts::SIZE_XXL,
                colors::CAB_YELLOW,
            );
            draw_small_caps(
                "Shift suspended. The meter is still waiting.",
                inner.x,
                inner.y + 66.0,
                fonts::SIZE_SM,
                colors::TEXT_MUTED,
            );

            let stat_y = inner.y + 104.0;
            let stat_gap = 10.0;
            let stat_w = (inner.w - stat_gap * 2.0) / 3.0;
            let mins = self.game_state.time_remaining % 60;
            let hours = self.game_state.time_remaining / 60;
            let stats = [
                (
                    "Fuel",
                    format!("{:.0}%", self.game_state.fuel),
                    get_fuel_color(self.game_state.fuel),
                ),
                (
                    "Earned",
                    format!("${}", self.game_state.earnings),
                    colors::ACCENT_GOLD,
                ),
                (
                    "Time",
                    format!("{}:{:02}", hours, mins),
                    colors::TEXT_SECONDARY,
                ),
            ];
            for (idx, (label, value, color)) in stats.iter().enumerate() {
                let rect = UiRect::new(
                    inner.x + idx as f32 * (stat_w + stat_gap),
                    stat_y,
                    stat_w,
                    72.0,
                );
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    Color::new(0.025, 0.030, 0.032, 0.92),
                );
                draw_rectangle(rect.x, rect.y, 4.0, rect.h, *color);
                draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, colors::BORDER_DIM);
                draw_text(value, rect.x + 14.0, rect.y + 30.0, fonts::SIZE_LG, *color);
                draw_small_caps(
                    label,
                    rect.x + 14.0,
                    rect.y + 54.0,
                    fonts::SIZE_XS,
                    colors::TEXT_MUTED,
                );
            }

            let action_y = stat_y + 104.0;
            if draw_glass_button(
                UiRect::new(inner.x, action_y, inner.w, 48.0),
                "Resume (ESC)",
                colors::CAB_YELLOW,
                true,
            ) {
                return UiAction::TogglePauseMenu;
            }

            if draw_glass_button(
                UiRect::new(inner.x, action_y + 62.0, inner.w, 48.0),
                "Return to Menu",
                colors::ACCENT_DANGER,
                true,
            ) {
                return UiAction::ReturnToMenu;
            }

            draw_rectangle(
                panel.x,
                panel.bottom() - 1.0,
                panel.w,
                1.0,
                Color::new(0.95, 0.58, 0.08, 0.55),
            );
        }

        // Draw transition overlay (always on top)
        self.transition.draw();

        action
    }

    /// Draw the current game phase (during active gameplay)
    fn draw_game_phase(&self) -> UiAction {
        let phase_action = game_screens::draw_game(
            &self.game_state,
            self.game_data.as_ref(),
            &self.player_stats,
        );

        // Draw status bar above the phase artwork and capture any action from it.
        let status_action = if let Some(ref data) = self.game_data {
            StatusBar::draw(&self.game_state, &data.constants, self.game_data.as_ref())
        } else {
            UiAction::None
        };

        if status_action != UiAction::None {
            return status_action;
        }
        phase_action
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

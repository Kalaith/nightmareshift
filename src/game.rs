use macroquad::prelude::*;

mod actions;
mod render;
mod rules;

use crate::bot::{PlaytestBot, PlaytestDirective};
use crate::data::{ActionType, GameData, RouteType, Rule, RuleType};
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;
use crate::ui::layout;
use macroquad_toolkit::ui::ScrollArea;

/// What a press on the menu's delete button should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeleteDecision {
    /// Prime the button and wait for a second press.
    Arm,
    /// Confirmed — destroy the save.
    Erase,
}

/// Main game structure
pub struct Game {
    screen: Screen,
    game_data: Option<GameData>,
    game_state: GameState,
    player_stats: PlayerStats,
    show_rules: bool,
    show_inventory: bool,
    show_pause_menu: bool,
    transition: ScreenFade,
    particles: WeatherParticles,
    screen_shake: ScreenShake,
    last_frame_time: f64,
    loading_frames: u32,
    last_hazard_update: f64,
    playtest_bot: Option<PlaytestBot>,
    skill_tree_scroll: ScrollArea,
    almanac_scroll: ScrollArea,
    almanac_selected: Option<u32>,
    /// When the menu's delete button is armed until, if it is. A first click
    /// arms it and a second inside the window confirms; anything else lets it
    /// lapse. Deleting takes every meta-progression the player has.
    delete_armed_until: Option<f64>,
    /// Set while the screenshot harness is driving the game. Capture scenes
    /// seed bank balance, lore, and almanac levels directly into
    /// `player_stats`; persisting any of that would hand the player a save
    /// they did not earn, so saving is suppressed for the whole run.
    capture_mode: bool,
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
            transition: ScreenFade::new(0.3),
            particles: WeatherParticles::new(),
            screen_shake: ScreenShake::new(15.0),
            last_frame_time: current_time,
            loading_frames: 0,
            last_hazard_update: current_time,
            playtest_bot,
            skill_tree_scroll: ScrollArea::new(),
            almanac_scroll: ScrollArea::new(),
            almanac_selected: None,
            delete_armed_until: None,
            capture_mode: false,
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_mode = true;
        match scene {
            "briefing" => self.start_game(),
            // The skill tree with currency in hand, so the purchase buttons
            // and the lore exchange are both live in the capture.
            "skill_tree" => {
                self.player_stats.bank_balance += 2500;
                self.player_stats.lore_fragments += 40;
                self.change_screen(Screen::SkillTree);
            }
            "gameplay" => {
                self.start_game();
                self.start_shift();
            }
            // Where a run ends. Three states of the same screen: a lost
            // shift, an interim night survived, and a completed run.
            "game_over" => {
                self.start_game();
                self.game_state.night = 3;
                self.game_state.earnings = 118;
                self.game_state.rides_completed = 4;
                self.game_state.time_remaining = 96;
                self.game_state.game_over_reason =
                    Some("The passenger's need became uncontrollable.".to_string());
                self.game_state.game_phase = GamePhase::GameOver;
                self.change_screen(Screen::GameOver);
            }
            "night_complete" => {
                self.start_game();
                self.game_state.night = 2;
                self.game_state.earnings = 288;
                self.game_state.rides_completed = 7;
                self.game_state.time_remaining = 74;
                self.game_state.run_complete = false;
                self.game_state.shift_payout = MetaPayout {
                    bank: 144,
                    lore: 11,
                    ..MetaPayout::default()
                };
                self.game_state.game_phase = GamePhase::Success;
                self.change_screen(Screen::Success);
            }
            "run_complete" => {
                self.start_game();
                self.game_state.night = 5;
                self.game_state.earnings = 471;
                self.game_state.rides_completed = 9;
                self.game_state.time_remaining = 51;
                self.game_state.run_complete = true;
                self.game_state.shift_payout = MetaPayout {
                    bank: 235,
                    lore: 14,
                    run_bonus_bank: 1500,
                    run_bonus_lore: 15,
                };
                self.game_state.game_phase = GamePhase::Success;
                self.change_screen(Screen::Success);
            }
            // The leaderboard with a spread of recorded runs, so the ranking
            // and the achievement list are both populated in the capture.
            // The menu with the delete button already armed, so the warning
            // state is visible without a click.
            "delete_armed" => {
                self.player_stats.bank_balance += 4200;
                self.player_stats.lore_fragments += 260;
                self.delete_armed_until = Some(get_time() + 3600.0);
                self.change_screen(Screen::MainMenu);
            }
            "leaderboard" => {
                let entries = [
                    (1840_u32, 9_u32, 4_u32, 0_u32, true),
                    (1470, 7, 3, 1, true),
                    (1120, 6, 3, 0, true),
                    (860, 5, 2, 2, false),
                    (410, 3, 1, 1, false),
                    (150, 1, 0, 3, false),
                ];
                for (score, rides, difficulty, violations, survived) in entries {
                    self.player_stats.add_leaderboard_entry(LeaderboardEntry {
                        score,
                        date: "2026-07-29 23:15".to_string(),
                        survived,
                        passengers_transported: rides,
                        difficulty_level: difficulty,
                        rules_violated: violations,
                    });
                }
                self.change_screen(Screen::Leaderboard);
            }
            // The inventory holding a cursed item and a plain one, so the
            // curse line and its way out are visible in the capture.
            "inventory" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(data) = &self.game_data {
                    let now = get_time();
                    for name in ["Old Locket", "Crystal Pendant", "Crumpled Note"] {
                        let item = data.items.create_item(name, "Mrs. Chen", now);
                        self.game_state.inventory.push(item);
                    }
                }
                self.show_inventory = true;
            }
            // The rules panel mid-ride, so each rule's authored reason for
            // existing is visible in the capture.
            "rules_panel" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                self.show_rules = true;
            }
            // A trade offer with a mixed inventory: an item this passenger
            // wants, one they do not, and one that cannot be traded at all.
            // Exercises the wanted-item highlight and the tradeable filter.
            "trade" => {
                self.start_game();
                self.start_shift();
                self.spawn_passenger();
                if let Some(data) = &self.game_data {
                    if let Some(passenger) = self.game_state.current_passenger.clone() {
                        let now = get_time();
                        let wanted = passenger
                            .wanted_items
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "Old Key".to_string());
                        for name in ["Blessed Medallion", "Crumpled Note", wanted.as_str()] {
                            let item = data.items.create_item(name, &passenger.name, now);
                            self.game_state.inventory.push(item);
                        }
                        let offered = data.items.create_item("Tarot Card", &passenger.name, now);
                        self.game_state.last_ride_completion = Some(RideCompletion {
                            passenger: passenger.clone(),
                            fare_earned: passenger.fare,
                            items_received: Vec::new(),
                            backstory_unlocked: None,
                        });
                        self.game_state.pending_trade = Some((passenger.name.clone(), offered));
                        self.game_state.game_phase = GamePhase::DropOff;
                    }
                }
            }
            // A ride offer with the almanac fully studied, so the dossier the
            // request screen draws is visible in the capture.
            "ride_request" => {
                self.start_game();
                self.start_shift();
                if let Some(data) = &self.game_data {
                    for passenger in &data.passengers {
                        self.player_stats.mark_passenger_encountered(passenger.id);
                        for _ in 0..3 {
                            self.player_stats.lore_fragments += 99;
                            let level = self
                                .player_stats
                                .get_almanac_entry(passenger.id)
                                .knowledge_level;
                            let cost = data.almanac.get_upgrade_cost(level + 1);
                            self.player_stats
                                .upgrade_almanac_knowledge(passenger.id, cost);
                        }
                    }
                }
                self.spawn_passenger();
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
        if self.capture_mode {
            return;
        }
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

    /// Start a new run from night 1.
    pub fn start_game(&mut self) {
        self.game_state.night = 1;
        self.game_state.run_complete = false;
        self.begin_night();
    }

    /// Advance to the next night of the current run.
    fn advance_night(&mut self) {
        self.game_state.night += 1;
        self.begin_night();
    }

    /// Set up and begin the current night (`game_state.night`), scaling
    /// difficulty and the earnings quota with how deep into the run we are.
    fn begin_night(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            self.player_stats.session_start = Some(current_time);

            // Reset per-night resources (fuel, time, earnings) but keep the run's
            // night counter, which is owned by start_game/advance_night.
            self.game_state
                .reset_for_new_shift(current_time, &data.constants.game_constants);

            // Difficulty escalates with the night within the run, layered on top
            // of the player's lifetime experience, and drives the rule count.
            let night = self.game_state.night;
            let max_diff = data.constants.scoring.max_difficulty;
            let base_diff = self.player_stats.suggested_difficulty();
            let difficulty_step = data.constants.game_constants.difficulty_increase_per_night;
            let effective_diff = (base_diff + (night - 1) * difficulty_step).min(max_diff);
            let synthetic_xp = effective_diff * data.constants.scoring.experience_per_level;
            let shift_rules =
                GameEngine::generate_shift_rules(synthetic_xp, &data.rules, &data.constants);
            self.game_state.current_rules = shift_rules.visible_rules;
            self.game_state.hidden_rules = shift_rules.hidden_rules;
            self.game_state.difficulty_level = shift_rules.difficulty_level;

            // The nightly quota rises by an authored share of the base each
            // night: 150, 225, 300, 375, 450 across a five-night run at the
            // shipped 0.5, against a shift whose fuel and clock do not grow.
            let base_quota = data.constants.game_constants.minimum_earnings;
            let step = data.constants.game_constants.quota_increase_per_night;
            let growth = (base_quota as f32 * step * (night - 1) as f32).round() as u32;
            self.game_state.minimum_earnings = base_quota + growth;

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

            self.transition.begin_scene();
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
        self.transition.begin_scene();
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
        let Some(constants) = self.game_data.as_ref().map(|data| data.constants.clone()) else {
            return;
        };
        if let Err(reason) = RideService::accept_ride(&mut self.game_state, &constants) {
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
        // Leaving the drop-off declines any offer still on the table. The trade
        // modal only draws during DropOff, so a surviving offer would go
        // invisible and then reappear over the next passenger's drop-off,
        // trading an item the previous passenger was holding.
        self.game_state.pending_trade = None;

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

        let nights_per_run = self
            .game_data
            .as_ref()
            .map(|d| d.constants.game_constants.nights_per_run)
            .unwrap_or(5)
            .max(1);

        if actually_successful {
            // Add survival bonus
            if let Some(ref data) = self.game_data {
                self.game_state.earnings += data.constants.game_constants.survival_bonus;
            }
            // Surviving the final night completes the run; otherwise this is an
            // interim night and the results screen offers to press on.
            self.game_state.run_complete = self.game_state.night >= nights_per_run;
            self.transition.begin_scene();
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
            self.screen_shake.shake(1.0, 0.5);
            self.transition.begin_scene();
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
        self.game_state.shift_payout.bank = bank_earnings;

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
        self.game_state.shift_payout.lore = lore_earned;

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

        // Check and unlock achievements, paying whatever each one is worth.
        let unlocked = self.player_stats.check_achievements(
            self.game_state.earnings,
            actually_successful,
            self.game_state.rules_violated,
        );
        self.pay_achievement_rewards(&unlocked);

        // Surviving every night of a run is the game's headline result and
        // used to pay nothing beyond the per-night survival bonus.
        if actually_successful && self.game_state.run_complete {
            if let Some(data) = &self.game_data {
                let nights = data.constants.game_constants.nights_per_run.max(1);
                let payout = data.rewards.run_completion.payout(nights);
                self.player_stats.bank_balance += payout.bank;
                self.player_stats.lore_fragments += payout.lore;
                self.game_state.shift_payout.run_bonus_bank = payout.bank;
                self.game_state.shift_payout.run_bonus_lore = payout.lore;
            }
        }

        // Auto-save after shift
        self.save_stats();
    }

    /// Pay the bank and lore a freshly unlocked achievement is worth.
    ///
    /// The six achievements were pure scoreboard entries; paying them makes
    /// the behaviours they name — surviving, mastering the almanac, buying
    /// into the skill tree — feed the currencies that buy the next upgrade.
    fn pay_achievement_rewards(&mut self, unlocked: &[String]) {
        let Some(data) = &self.game_data else {
            return;
        };
        let mut total = crate::data::Payout::default();
        for id in unlocked {
            let payout = data.rewards.for_achievement(id);
            total.bank += payout.bank;
            total.lore += payout.lore;
        }
        if total.is_empty() {
            return;
        }
        self.player_stats.bank_balance += total.bank;
        self.player_stats.lore_fragments += total.lore;
    }

    /// How long a primed delete stays primed.
    const DELETE_CONFIRM_WINDOW: f64 = 5.0;

    /// First press arms the delete, second inside the window carries it out.
    ///
    /// This used to erase the save on a single click from the menu. Everything
    /// the meta-progression holds — bank balance, lore, almanac levels,
    /// unlocked skills, the leaderboard, achievements — went with one press of
    /// a button sitting directly under "Leaderboard".
    fn arm_or_delete_save(&mut self) {
        let now = get_time();
        match Self::delete_decision(self.delete_armed_until, now) {
            DeleteDecision::Arm => {
                self.delete_armed_until = Some(now + Self::DELETE_CONFIRM_WINDOW);
            }
            DeleteDecision::Erase => {
                self.delete_armed_until = None;
                if Persistence::delete_save().is_ok() {
                    self.player_stats = PlayerStats::new();
                    self.player_stats.init_achievements();
                }
            }
        }
    }

    /// Whether a press on the delete button arms it or carries it out.
    ///
    /// Split out from `arm_or_delete_save` so it can be tested without a
    /// window: the branch that decides whether a save is destroyed is worth
    /// pinning, and `get_time` needs a graphics context.
    fn delete_decision(armed_until: Option<f64>, now: f64) -> DeleteDecision {
        match armed_until {
            Some(until) if now < until => DeleteDecision::Erase,
            _ => DeleteDecision::Arm,
        }
    }

    /// Let an untouched delete prompt lapse rather than waiting for a click.
    fn expire_delete_prompt(&mut self) {
        if let Some(until) = self.delete_armed_until {
            if get_time() >= until || self.screen != Screen::MainMenu {
                self.delete_armed_until = None;
            }
        }
    }

    /// Return to main menu
    fn return_to_menu(&mut self) {
        self.change_screen(Screen::MainMenu);
        self.particles.clear();
    }

    /// Change screen with transition
    fn change_screen(&mut self, new_screen: Screen) {
        self.transition.begin_scene();
        self.screen = new_screen;
        // Only the screens that correspond to a moment in a shift move the
        // phase; the meta screens leave the night as it was.
        self.game_state.game_phase = match new_screen {
            Screen::Loading | Screen::MainMenu => GamePhase::Loading,
            Screen::Briefing => GamePhase::Briefing,
            Screen::GameOver => GamePhase::GameOver,
            Screen::Success => GamePhase::Success,
            Screen::Game | Screen::SkillTree | Screen::Almanac | Screen::Leaderboard => {
                self.game_state.game_phase
            }
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

        self.expire_delete_prompt();

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

            // Stranded: not enough left for any route out. End the night
            // where it stands rather than leaving the player with four
            // disabled buttons and a clock that only moves when they drive.
            if self.game_state.game_phase == GamePhase::Driving {
                let stranded = self
                    .game_data
                    .as_ref()
                    .map(|data| {
                        RideService::is_stranded(&self.game_state, data, &self.player_stats)
                    })
                    .unwrap_or(false);
                if stranded {
                    self.game_state.game_over_reason = Some(
                        "You could not make the next leg. The shift ends where it stands."
                            .to_string(),
                    );
                    let earned_enough =
                        self.game_state.earnings >= self.game_state.minimum_earnings;
                    self.end_shift(earned_enough);
                    return;
                }
            }

            // Update guideline decision timer
            let mut guideline_timed_out = false;
            if self.game_state.game_phase == GamePhase::GuidelineDecision {
                if let Some(start_time) = self.game_state.guideline_decision_start_time {
                    let elapsed = (current_time - start_time) as f32;
                    self.game_state.guideline_time_remaining = (30.0 - elapsed).max(0.0);
                    guideline_timed_out = self.game_state.guideline_time_remaining <= 0.0;
                }
            }
            // Time's up: force the decision, defaulting to following the guideline.
            if guideline_timed_out {
                self.evaluate_guideline_decision(GuidelineAction::Follow);
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
}

#[cfg(test)]
mod delete_tests {
    use super::{DeleteDecision, Game};

    /// A first press must never erase. This was a single click from the main
    /// menu, taking the bank, every lore fragment, every almanac level and
    /// every unlocked skill with it.
    #[test]
    fn a_first_press_only_arms() {
        assert_eq!(Game::delete_decision(None, 100.0), DeleteDecision::Arm);
    }

    /// A second press inside the window is the confirmation.
    #[test]
    fn a_second_press_inside_the_window_erases() {
        let armed_until = 100.0 + Game::DELETE_CONFIRM_WINDOW;
        assert_eq!(
            Game::delete_decision(Some(armed_until), 101.0),
            DeleteDecision::Erase
        );
    }

    /// Once the window has passed the prompt is stale, and a press starts
    /// over rather than destroying a save the player stopped thinking about.
    #[test]
    fn a_press_after_the_window_arms_again() {
        assert_eq!(
            Game::delete_decision(Some(100.0), 100.0),
            DeleteDecision::Arm
        );
        assert_eq!(
            Game::delete_decision(Some(100.0), 500.0),
            DeleteDecision::Arm
        );
    }
}

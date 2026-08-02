use macroquad::prelude::*;

mod actions;
mod capture_scenes;
mod render;
mod rules;
mod shift;

use crate::bot::{PlaytestBot, PlaytestDirective};
use crate::data::{ActionType, GameData, RouteType, Rule, RuleType};
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;
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
    /// Why `game_data` is `None`, when it is. Shown on the loading screen,
    /// which refuses to advance — a structural data failure used to be a
    /// panic, which on the web build is a silent black canvas.
    data_error: Option<String>,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let current_time = get_time();

        // The data is embedded at compile time, so a failure here is a
        // build defect — but the window still deserves to say so rather
        // than panic to a black canvas.
        let (game_data, data_error) = match GameData::load() {
            Ok(data) => (Some(data), None),
            Err(error) => {
                eprintln!("Game data failed to load: {error}");
                (None, Some(error))
            }
        };

        // Try to load saved player stats
        let mut player_stats = Persistence::load().unwrap_or_else(|_| PlayerStats::new());
        player_stats.init_achievements();
        let playtest_bot = PlaytestBot::from_launch_args();
        if let (Some(bot), Some(data)) = (&playtest_bot, &game_data) {
            bot.apply_test_unlocks(&mut player_stats, data);
        }

        // Create game state using constants from loaded data
        let constants = game_data
            .as_ref()
            .map(|data| data.constants.game_constants.clone())
            .unwrap_or_default();
        let game_state = GameState::new(current_time, &constants);

        Self {
            screen: Screen::Loading,
            game_data,
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
            data_error,
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
        // A fresh ride starts a fresh ledger; the drop-off screen reads it.
        self.game_state.consequence_notes.clear();
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

    /// True while the driving screen would let this route be clicked: not
    /// hazard-blocked, and the quoted fuel/time cost is payable. The engine
    /// still adds curse and streak pressure on top of the quote, so an open
    /// route can legitimately end the shift once actually taken.
    fn route_open(
        state: &GameState,
        stats: &PlayerStats,
        data: &GameData,
        route: RouteType,
    ) -> bool {
        if state
            .environmental_hazards
            .iter()
            .any(|hazard| hazard.blocks_route(route))
        {
            return false;
        }
        let quote = RouteService::quote_route(route, state, data, stats);
        (state.fuel as u32) >= quote.fuel && state.time_remaining >= quote.time
    }

    /// Route selection with the driving screen's gate applied, so the digit
    /// keys and the playtest bot cannot take a route the cards refuse. When
    /// every route is closed the night cannot continue by any input, so it
    /// ends instead of soft-locking on the route screen.
    fn select_route(&mut self, route: RouteType) {
        let Some(data) = self.game_data.as_ref() else {
            return;
        };
        if Self::route_open(&self.game_state, &self.player_stats, data, route) {
            self.choose_route(route);
            return;
        }
        let any_open = [
            RouteType::Normal,
            RouteType::Shortcut,
            RouteType::Scenic,
            RouteType::Police,
        ]
        .into_iter()
        .any(|open| Self::route_open(&self.game_state, &self.player_stats, data, open));
        if !any_open {
            self.game_state.game_over_reason =
                Some("No route the cab can still afford. The night ends here.".to_string());
            self.end_shift(false);
        }
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
                return;
            }

            if self.game_state.game_phase == GamePhase::DropOff {
                self.pay_for_keeping_the_rule(current_time);
            }
        }
    }

    /// Continue past an event that offered nothing to answer, onto the
    /// ride's final leg — the same exit `resolve_event_choice` takes, minus
    /// a consequence to apply.
    fn continue_past_event(&mut self) {
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

    /// Refuel to full capacity
    fn refuel_full(&mut self) {
        if let Some(ref data) = self.game_data {
            let refuel_mult =
                SkillModifiers::from_unlocked(&data.skills, &self.player_stats.unlocked_skills)
                    .refuel_cost_mult;
            let fuel_needed = self.game_state.max_fuel - self.game_state.fuel;
            let cost = data.constants.fuel.refuel_cost(fuel_needed, refuel_mult);

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
            let cost = data.constants.fuel.refuel_cost(amount, refuel_mult);

            if self.game_state.earnings >= cost {
                self.game_state.fuel =
                    (self.game_state.fuel + amount).min(self.game_state.max_fuel);
                self.game_state.earnings -= cost;
            }
        }
    }

    /// Use an item from inventory
    fn use_item(&mut self, idx: usize) {
        let Some(constants) = self
            .game_data
            .as_ref()
            .map(|data| data.constants.reputation.clone())
        else {
            return;
        };
        if ItemService::use_item(&mut self.game_state, idx, &constants, get_time()) {
            self.show_inventory = false;
        }
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
            // With no data there is nothing to advance to; the loading
            // screen holds and shows the error instead.
            if self.loading_frames >= 2 && self.game_data.is_some() {
                self.change_screen(Screen::MainMenu);
            }
        }

        self.expire_delete_prompt();
        // Whatever pushed the passenger's need this frame, settle what their
        // escalation did to the driver's standing exactly once.
        self.game_state.settle_passenger_trust();

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

            // Update guideline decision timer.
            //
            // The countdown is measured against an absolute start time, so it
            // kept running behind the pause menu — open the thirty-second
            // decision, press ESC to think it over, and the game would make
            // the choice for you while the menu was up. Pausing pushes the
            // start forward instead, holding whatever is left on the clock.
            let mut guideline_timed_out = false;
            if self.game_state.game_phase == GamePhase::GuidelineDecision {
                if self.show_pause_menu {
                    if let Some(start_time) = self.game_state.guideline_decision_start_time {
                        self.game_state.guideline_decision_start_time =
                            Some(start_time + dt as f64);
                    }
                } else if let Some(start_time) = self.game_state.guideline_decision_start_time {
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
            GuidelineEngine::update_detection(
                &mut self.game_state,
                &self.player_stats,
                current_time,
            );

            // Dynamic weather updates
            if self.game_state.shift_start_time.is_some() {
                self.game_state.current_weather = WeatherService::update_weather(
                    &self.game_state.current_weather,
                    &self.game_state.season,
                    current_time,
                );

                // The in-fiction clock advances with the shift's own
                // minutes, which routes spend, rather than wall time.
                let minutes_gone = self
                    .game_data
                    .as_ref()
                    .map(|data| {
                        data.constants
                            .game_constants
                            .initial_time
                            .saturating_sub(self.game_state.time_remaining)
                    })
                    .unwrap_or(0);
                self.game_state.time_of_day = WeatherService::time_of_day_after(minutes_gone);
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

    /// Which modal overlay is eating input this frame, if any. The pause
    /// menu wins over the panels because opening it closes them.
    fn active_overlay(&self) -> Overlay {
        if self.screen != Screen::Game {
            Overlay::None
        } else if self.show_pause_menu {
            Overlay::Pause
        } else if self.show_rules || self.show_inventory {
            Overlay::Panel
        } else {
            Overlay::None
        }
    }

    /// Handle input
    pub fn handle_input(&mut self) {
        let actions = InputService::capture_input(
            self.screen,
            self.game_state.game_phase,
            self.active_overlay(),
        );
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

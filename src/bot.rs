//! Automated playtest bot for smoke-testing the core gameplay loop.

use crate::data::{GameData, Passenger, PreferenceLevel, RouteType};
use crate::engine::{RouteService, SkillModifiers};
use crate::screens::Screen;
use crate::state::{AlmanacEntry, GamePhase, GameState, PlayerStats, RouteStreak};
use crate::ui::UiAction;

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaytestStrategy {
    Coverage,
    Conservative,
    Learned,
}

impl PlaytestStrategy {
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "conservative" | "safe" => Self::Conservative,
            "learned" | "smart" => Self::Learned,
            _ => Self::Coverage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaytestDirective {
    None,
    Action(UiAction),
    Stop(i32),
}

#[derive(Debug)]
pub struct PlaytestBot {
    strategy: PlaytestStrategy,
    max_shifts: u32,
    completed_shifts: u32,
    action_delay: f64,
    last_action_time: f64,
    route_cursor: usize,
    event_cursor: usize,
    guideline_cursor: usize,
    decision_count: u32,
    current_shift_logged: bool,
    last_signature: String,
    stale_since: f64,
    almanac_level: u32,
    unlock_all_skills: bool,
}

impl PlaytestBot {
    pub fn from_launch_args() -> Option<Self> {
        #[cfg(target_arch = "wasm32")]
        {
            None
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let args: Vec<String> = std::env::args().collect();
            let env_enabled = std::env::var("NIGHTMARE_SHIFT_BOT")
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            let arg_enabled = args.iter().any(|arg| arg == "--bot");
            if !env_enabled && !arg_enabled {
                return None;
            }

            let mut max_shifts = parse_u32_arg(&args, "--bot-shifts")
                .or_else(|| parse_env_u32("NIGHTMARE_SHIFT_BOT_SHIFTS"))
                .unwrap_or(3)
                .max(1);
            if args.iter().any(|arg| arg == "--bot-once") {
                max_shifts = 1;
            }

            let strategy = parse_string_arg(&args, "--bot-strategy")
                .or_else(|| std::env::var("NIGHTMARE_SHIFT_BOT_STRATEGY").ok())
                .map(|value| PlaytestStrategy::parse(&value))
                .unwrap_or(PlaytestStrategy::Coverage);

            let delay_ms = parse_u32_arg(&args, "--bot-delay-ms")
                .or_else(|| parse_env_u32("NIGHTMARE_SHIFT_BOT_DELAY_MS"))
                .unwrap_or(150);
            let almanac_level = parse_u32_arg(&args, "--bot-almanac-level")
                .or_else(|| parse_env_u32("NIGHTMARE_SHIFT_BOT_ALMANAC_LEVEL"))
                .unwrap_or(0)
                .min(3);
            // The skill tree is half of what meta-progression is supposed to
            // buy, and the harness could only seed the almanac, so a run's
            // skills were whatever happened to be in the save. That makes the
            // tree's contribution unmeasurable.
            let unlock_all_skills = args.iter().any(|arg| arg == "--bot-all-skills")
                || std::env::var("NIGHTMARE_SHIFT_BOT_ALL_SKILLS")
                    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                    .unwrap_or(false);

            let bot = Self {
                strategy,
                max_shifts,
                completed_shifts: 0,
                action_delay: delay_ms as f64 / 1000.0,
                last_action_time: 0.0,
                route_cursor: 0,
                event_cursor: 0,
                guideline_cursor: 0,
                decision_count: 0,
                current_shift_logged: false,
                last_signature: String::new(),
                stale_since: 0.0,
                almanac_level,
                unlock_all_skills,
            };

            eprintln!(
                "[BOT] Enabled: strategy={:?}, shifts={}, delay={}ms, almanac_level={}, all_skills={}",
                bot.strategy,
                bot.max_shifts,
                delay_ms,
                bot.almanac_level,
                bot.unlock_all_skills
            );
            Some(bot)
        }
    }

    pub fn apply_test_unlocks(&self, stats: &mut PlayerStats, data: &GameData) {
        if self.unlock_all_skills {
            stats.unlocked_skills = data.skills.iter().map(|skill| skill.id.clone()).collect();
            eprintln!("[BOT] Unlocked all {} skills.", stats.unlocked_skills.len());
        }

        if self.almanac_level == 0 {
            return;
        }

        for passenger in &data.passengers {
            stats.almanac_progress.insert(
                passenger.id,
                AlmanacEntry {
                    passenger_id: passenger.id,
                    encountered: true,
                    knowledge_level: self.almanac_level,
                },
            );

            if self.almanac_level >= 3 {
                stats.unlock_backstory(passenger.id);
            }
        }

        eprintln!(
            "[BOT] Applied almanac level {} to {} passengers.",
            self.almanac_level,
            data.passengers.len()
        );
    }

    pub fn next_action(
        &mut self,
        screen: Screen,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
        now: f64,
    ) -> PlaytestDirective {
        self.update_stale_watchdog(screen, state, now);
        if self.is_stale(now) {
            eprintln!(
                "[BOT] Stopping: no phase/progress change for {:.1}s at {:?}/{:?}",
                now - self.stale_since,
                screen,
                state.game_phase
            );
            self.log_state_summary(state);
            return PlaytestDirective::Stop(2);
        }

        if now - self.last_action_time < self.action_delay {
            return PlaytestDirective::None;
        }

        let action = match screen {
            Screen::Loading => UiAction::None,
            Screen::MainMenu => {
                self.current_shift_logged = false;
                UiAction::StartGame
            }
            Screen::Briefing => {
                self.current_shift_logged = false;
                UiAction::StartGame
            }
            Screen::Game => self.game_action(state, stats, data),
            Screen::GameOver | Screen::Success => {
                if !self.current_shift_logged {
                    self.log_terminal_shift(screen, state);
                    self.current_shift_logged = true;
                    self.completed_shifts += 1;
                }

                if self.completed_shifts >= self.max_shifts {
                    eprintln!("[BOT] Finished {} shift(s).", self.completed_shifts);
                    return PlaytestDirective::Stop(0);
                }

                // Press on through interim nights so the campaign path (nights
                // 2..N) is exercised; otherwise begin a fresh run.
                if screen == Screen::Success && !state.run_complete {
                    UiAction::NextNight
                } else {
                    UiAction::TryAgain
                }
            }
            Screen::SkillTree | Screen::Almanac | Screen::Leaderboard => UiAction::ReturnToMenu,
        };

        if action == UiAction::None {
            PlaytestDirective::None
        } else {
            self.last_action_time = now;
            self.decision_count += 1;
            eprintln!(
                "[BOT] #{:03} {:?}/{:?} -> {:?}",
                self.decision_count, screen, state.game_phase, action
            );
            PlaytestDirective::Action(action)
        }
    }

    fn game_action(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
    ) -> UiAction {
        match state.game_phase {
            GamePhase::Waiting => {
                if state.earnings >= state.minimum_earnings {
                    UiAction::EndShift
                } else if state.fuel < 35.0 && state.earnings >= 15 {
                    UiAction::RefuelPartial
                } else {
                    UiAction::Continue
                }
            }
            GamePhase::RideRequest => UiAction::AcceptRide,
            GamePhase::Driving => {
                UiAction::SelectRoute(self.choose_route_index(state, stats, data))
            }
            GamePhase::Interaction => {
                if let Some(event) = &state.current_event {
                    if event.choices.is_empty() {
                        UiAction::Continue
                    } else {
                        let idx = self.event_cursor % event.choices.len();
                        self.event_cursor += 1;
                        UiAction::SelectEventChoice(idx)
                    }
                } else {
                    UiAction::Continue
                }
            }
            GamePhase::GuidelineDecision => {
                if self.strategy == PlaytestStrategy::Coverage {
                    let action = if self.guideline_cursor.is_multiple_of(2) {
                        UiAction::FollowGuideline
                    } else {
                        UiAction::BreakGuideline
                    };
                    self.guideline_cursor += 1;
                    action
                } else {
                    Self::read_the_passenger(state, data)
                }
            }
            GamePhase::DropOff => UiAction::Continue,
            GamePhase::GameOver | GamePhase::Success => UiAction::None,
            _ => UiAction::None,
        }
    }

    /// Decide a guideline the way an informed player would: if a tell detected
    /// for this guideline names an exception where breaking is the safer
    /// course, break it; otherwise follow.
    ///
    /// Blindly following was harmless while the decision phase was
    /// unreachable. Now that it fires, following an exception with
    /// `breakingSafer` set is the "misread the passenger" death, so the
    /// non-coverage strategies have to actually read.
    fn read_the_passenger(state: &GameState, data: Option<&GameData>) -> UiAction {
        let Some((guideline, data)) = state.active_guideline.as_ref().zip(data) else {
            return UiAction::FollowGuideline;
        };
        let breaking_safer = state
            .detected_tells
            .iter()
            .filter(|tell| tell.related_guideline == Some(guideline.id))
            .filter_map(|tell| tell.exception_id.as_deref())
            .any(|exception_id| {
                data.guidelines
                    .iter()
                    .flat_map(|g| g.exceptions.iter())
                    .any(|e| e.id == exception_id && e.breaking_safer)
            });

        if breaking_safer {
            UiAction::BreakGuideline
        } else {
            UiAction::FollowGuideline
        }
    }

    fn choose_route_index(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
    ) -> usize {
        let candidates = available_route_indices(state);
        if candidates.is_empty() {
            return 0;
        }

        match self.strategy {
            PlaytestStrategy::Coverage => {
                let idx = candidates[self.route_cursor % candidates.len()];
                self.route_cursor += 1;
                idx
            }
            PlaytestStrategy::Conservative => [0, 3, 2, 1]
                .into_iter()
                .find(|idx| candidates.contains(idx))
                .unwrap_or(candidates[0]),
            PlaytestStrategy::Learned => self.choose_learned_route(state, stats, data, &candidates),
        }
    }

    fn choose_learned_route(
        &mut self,
        state: &GameState,
        stats: &PlayerStats,
        data: Option<&GameData>,
        candidates: &[usize],
    ) -> usize {
        let Some(passenger) = state.current_passenger.as_ref() else {
            return candidates[0];
        };
        let Some(data) = data else {
            let idx = candidates[self.route_cursor % candidates.len()];
            self.route_cursor += 1;
            return idx;
        };

        let knowledge = stats.get_almanac_entry(passenger.id).knowledge_level;
        if knowledge < 2 {
            let idx = candidates[self.route_cursor % candidates.len()];
            self.route_cursor += 1;
            return idx;
        }

        let passenger_risk = state
            .current_passenger
            .as_ref()
            .and_then(|p| data.get_location(&p.pickup).map(|l| l.risk_level))
            .unwrap_or(1);
        let route_mastery = stats.route_mastery_map();
        let skill_mods = SkillModifiers::from_unlocked(&data.skills, &stats.unlocked_skills);
        let mut evaluated = Vec::new();

        for idx in candidates {
            let route = route_for_index(*idx);
            let costs = RouteService::calculate_route_costs(
                route,
                &data.constants,
                passenger_risk,
                Some(&state.current_weather),
                Some(&state.time_of_day),
                &state.environmental_hazards,
                &route_mastery,
                Some(passenger),
                &skill_mods,
            );

            if state.fuel < costs.fuel as f32 || state.time_remaining < costs.time {
                continue;
            }

            evaluated.push((*idx, route, costs));
        }

        if evaluated.is_empty() {
            return candidates[0];
        }

        let min_next_route_time = evaluated
            .iter()
            .map(|(_, _, costs)| costs.time)
            .min()
            .unwrap_or(0);

        evaluated
            .iter()
            .max_by_key(|(_, route, costs)| {
                let fare = estimate_fare(data, state, passenger, *route);
                let projected_earnings = state.earnings + fare;
                let remaining_time = state.time_remaining.saturating_sub(costs.time);
                let preference_score = passenger
                    .get_route_preference(*route)
                    .map(|pref| match pref.preference {
                        PreferenceLevel::Loves => 45,
                        PreferenceLevel::Likes => 28,
                        PreferenceLevel::Neutral => 8,
                        PreferenceLevel::Dislikes => -18,
                        PreferenceLevel::Fears => -55,
                    })
                    .unwrap_or(0);

                let finish_score = if projected_earnings >= state.minimum_earnings {
                    if remaining_time == 0 {
                        260
                    } else if remaining_time < min_next_route_time {
                        -180
                    } else {
                        90 - remaining_time.min(90) as i32
                    }
                } else if remaining_time > 0 && remaining_time < min_next_route_time {
                    -240
                } else {
                    0
                };

                let shortcut_rule_penalty = if *route == RouteType::Shortcut
                    && state.rides_completed > 0
                    && state
                        .current_rules
                        .iter()
                        .any(|rule| rule.forbids_action("take_shortcut"))
                {
                    -300
                } else {
                    0
                };

                fare as i32 * 12
                    + preference_score
                    + finish_score
                    + shortcut_rule_penalty
                    + stats.get_route_usage(*route).min(20) as i32
                    - costs.time as i32 * 4
                    - costs.fuel as i32 * 3
                    - costs.risk as i32 * 8
            })
            .map(|(idx, _, _)| *idx)
            .unwrap_or(candidates[0])
    }

    fn update_stale_watchdog(&mut self, screen: Screen, state: &GameState, now: f64) {
        let signature = format!(
            "{:?}|{:?}|{:?}|{}|{}|{:.0}|{}|{}",
            screen,
            state.game_phase,
            state.driving_phase,
            state.rides_completed,
            state.route_history.len(),
            state.fuel,
            state.earnings,
            state.used_passengers.len()
        );

        if signature != self.last_signature {
            self.last_signature = signature;
            self.stale_since = now;
        } else if self.stale_since == 0.0 {
            self.stale_since = now;
        }
    }

    fn is_stale(&self, now: f64) -> bool {
        self.stale_since > 0.0 && now - self.stale_since > 12.0
    }

    fn log_terminal_shift(&self, screen: Screen, state: &GameState) {
        let route_summary = state
            .route_history
            .iter()
            .map(|entry| format!("{:?}", entry.route_type))
            .collect::<Vec<_>>()
            .join(" -> ");
        eprintln!(
            "[BOT] Shift {} ended on {:?}: rides={}, earnings=${}, fuel={:.0}%, time={}, routes=[{}], reason={}",
            self.completed_shifts + 1,
            screen,
            state.rides_completed,
            state.earnings,
            state.fuel,
            state.time_remaining,
            route_summary,
            state
                .game_over_reason
                .as_deref()
                .unwrap_or("completed")
        );
    }

    fn log_state_summary(&self, state: &GameState) {
        eprintln!(
            "[BOT] State: phase={:?}, rides={}, fuel={:.0}%, earnings=${}, time={}, routes={}, passenger={}",
            state.game_phase,
            state.rides_completed,
            state.fuel,
            state.earnings,
            state.time_remaining,
            state.route_history.len(),
            state
                .current_passenger
                .as_ref()
                .map(|passenger| passenger.name.as_str())
                .unwrap_or("none")
        );
    }
}

fn available_route_indices(state: &GameState) -> Vec<usize> {
    [0, 1, 2, 3]
        .into_iter()
        .filter(|idx| {
            let route = route_for_index(*idx);
            !state
                .environmental_hazards
                .iter()
                .any(|hazard| hazard.blocks_route(route))
        })
        .collect()
}

fn route_for_index(idx: usize) -> RouteType {
    match idx {
        1 => RouteType::Shortcut,
        2 => RouteType::Scenic,
        3 => RouteType::Police,
        _ => RouteType::Normal,
    }
}

fn estimate_fare(
    data: &GameData,
    state: &GameState,
    passenger: &Passenger,
    route: RouteType,
) -> u32 {
    let route_mult = match route {
        RouteType::Shortcut => data.constants.route_fares.shortcut,
        RouteType::Normal => data.constants.route_fares.normal,
        RouteType::Scenic => data.constants.route_fares.scenic,
        RouteType::Police => data.constants.route_fares.police,
    };

    let pref_mult = passenger
        .get_route_preference(route)
        .map(|pref| pref.fare_modifier)
        .unwrap_or(1.0);

    let streak_mult = route_streak_multiplier(
        state.consecutive_route_streak.as_ref(),
        route,
        data.constants.consecutive_route.penalty_per_repeat,
    );
    let rep_mult = state
        .passenger_reputation
        .get(&passenger.id)
        .map(|rep| rep.fare_multiplier(&data.constants.reputation))
        .unwrap_or(1.0);

    (passenger.fare as f32 * route_mult * pref_mult * streak_mult * rep_mult)
        .max(5.0)
        .round() as u32
}

fn route_streak_multiplier(
    streak: Option<&RouteStreak>,
    route: RouteType,
    penalty_per_repeat: f32,
) -> f32 {
    if let Some(streak) = streak {
        if streak.route_type == route && streak.count >= 2 {
            return (1.0 - (streak.count - 1) as f32 * penalty_per_repeat).max(0.25);
        }
    }

    1.0
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_u32_arg(args: &[String], name: &str) -> Option<u32> {
    let prefix = format!("{}=", name);
    args.iter()
        .find_map(|arg| arg.strip_prefix(&prefix))
        .and_then(|value| value.parse().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_env_u32(name: &str) -> Option<u32> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_string_arg(args: &[String], name: &str) -> Option<String> {
    let prefix = format!("{}=", name);
    args.iter()
        .find_map(|arg| arg.strip_prefix(&prefix))
        .map(ToOwned::to_owned)
}

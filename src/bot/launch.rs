//! How the playtest bot is configured.
//!
//! Everything that turns command-line arguments and environment variables
//! into a `PlaytestBot`, plus the unlocks a run is seeded with. Kept apart
//! from the bot's decisions because this is what a measurement is set up
//! with, not how it plays: `--bot-almanac-level` and `--bot-skills` are the
//! knobs the progression comparisons turn.

use super::PlaytestBot;
// The bot never starts on the web, where there are no arguments to read.
#[cfg(not(target_arch = "wasm32"))]
use super::PlaytestStrategy;
use crate::data::GameData;
use crate::state::{AlmanacEntry, PlayerStats};

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
            // Unlocking the whole tree measures the tree. Isolating one
            // effect — is the fare multiplier really the 1.21 it multiplies
            // out to? — needs a named subset, because everything the other
            // nineteen nodes do lands in the same numbers.
            let named_skills: Vec<String> = parse_string_arg(&args, "--bot-skills")
                .or_else(|| std::env::var("NIGHTMARE_SHIFT_BOT_SKILLS").ok())
                .map(|value| {
                    value
                        .split(',')
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();

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
                named_skills,
                soothed_at_leg: None,
                soothe_cursor: 0,
            };

            eprintln!(
                "[BOT] Enabled: strategy={:?}, shifts={}, delay={}ms, almanac_level={}, all_skills={}, skills={:?}",
                bot.strategy,
                bot.max_shifts,
                delay_ms,
                bot.almanac_level,
                bot.unlock_all_skills,
                bot.named_skills
            );
            Some(bot)
        }
    }

    pub fn apply_test_unlocks(&self, stats: &mut PlayerStats, data: &GameData) {
        if self.unlock_all_skills {
            stats.unlocked_skills = data.skills.iter().map(|skill| skill.id.clone()).collect();
            eprintln!("[BOT] Unlocked all {} skills.", stats.unlocked_skills.len());
        } else if !self.named_skills.is_empty() {
            let known: Vec<String> = data.skills.iter().map(|skill| skill.id.clone()).collect();
            for id in &self.named_skills {
                if !known.contains(id) {
                    eprintln!("[BOT] Unknown skill id {id:?} - ignored.");
                }
            }
            stats.unlocked_skills = self
                .named_skills
                .iter()
                .filter(|id| known.contains(id))
                .cloned()
                .collect();
            eprintln!("[BOT] Unlocked {:?}.", stats.unlocked_skills);
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
                    ..AlmanacEntry::default()
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

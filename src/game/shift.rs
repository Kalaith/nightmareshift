//! The shape of a night and of a run.
//!
//! Starting a run, setting up each night with its escalating quota and
//! difficulty, ending a shift, and paying out what it earned into the bank
//! and the almanac. Kept apart from the frame loop and the ride flow because
//! this is the layer the meta-progression is settled in.

use super::Game;
use crate::engine::*;
use crate::screens::Screen;
use crate::state::*;
use macroquad::prelude::*;

/// How much of the night has been worked, when the wall clock is unavailable.
///
/// Both halves of this were wrong. The shift length was written here as a bare
/// `480` rather than read from `INITIAL_TIME`, so retuning the night in
/// `constants.json` would have left this one number behind. And the subtraction
/// is on `u32`: the Tarot Card grants a `time_bonus` of 60 and nothing caps the
/// clock at its starting value, so a card played early puts `time_remaining`
/// above the shift length and the old expression underflowed -- a panic in a
/// debug build, and about eight thousand years of logged play time in a release
/// one.
fn minutes_on_the_clock(initial_time: u32, time_remaining: u32) -> u32 {
    initial_time.saturating_sub(time_remaining)
}

impl Game {
    /// Start a new run from night 1.
    pub fn start_game(&mut self) {
        self.game_state.night = 1;
        self.game_state.run_complete = false;
        // Deal the saved standings out as this run's working copy.
        self.game_state.passenger_reputation = self.player_stats.passenger_reputation.clone();
        // Each run falls in its own month, so the seasonal spawn weights,
        // the winter hazard bonus, and the winter conditions branch rotate
        // into play across runs instead of being locked to October.
        self.game_state.campaign_month = self.game_state.rng.range_i32(1, 13) as u32;
        self.begin_night();
    }

    /// Advance to the next night of the current run.
    pub(super) fn advance_night(&mut self) {
        self.game_state.night += 1;
        self.begin_night();
    }

    /// Set up and begin the current night (`game_state.night`), scaling
    /// difficulty and the earnings quota with how deep into the run we are.
    pub(super) fn begin_night(&mut self) {
        if let Some(ref data) = self.game_data {
            let current_time = get_time();
            self.player_stats.session_start = Some(current_time);

            // Reset per-night resources (fuel, time, earnings) but keep the run's
            // night counter, which is owned by start_game/advance_night.
            self.game_state
                .reset_for_new_shift(current_time, &data.constants.game_constants);
            // A night starts with a clear windshield: no overlay opened over
            // a previous screen may survive into this one.
            self.overlays.close_all();

            // Difficulty escalates with the night within the run, layered on top
            // of the player's lifetime experience, and drives the rule count.
            let night = self.game_state.night;
            let max_diff = data.constants.scoring.max_difficulty;
            let base_diff = self.player_stats.suggested_difficulty();
            let difficulty_step = data.constants.game_constants.difficulty_increase_per_night;
            let effective_diff = (base_diff + (night - 1) * difficulty_step).min(max_diff);
            let synthetic_xp = effective_diff * data.constants.scoring.experience_per_level;
            let shift_rules = GameEngine::generate_shift_rules(
                &mut self.game_state.rng,
                synthetic_xp,
                &data.rules,
                &data.constants,
            );
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
                && self.game_state.rng.chance(skill_mods.reveal_hidden_chance)
            {
                if let Some(rule_id) = self.game_state.hidden_rules.first().map(|r| r.id) {
                    self.game_state.reveal_hidden_rule(rule_id);
                }
            }

            // Load guidelines for tell detection
            self.game_state.current_guidelines = data.guidelines.clone();

            // Initialize weather
            self.game_state.season =
                WeatherService::get_current_season(self.game_state.campaign_month);
            self.game_state.current_weather = WeatherService::generate_initial_weather(
                &mut self.game_state.rng,
                &self.game_state.season,
                current_time,
            );
            self.game_state.time_of_day = WeatherService::time_of_day_after(0);
            self.game_state.environmental_hazards = WeatherService::generate_hazards(
                &mut self.game_state.rng,
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

    /// Continue after drop off
    pub(super) fn continue_from_dropoff(&mut self) {
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

    /// End the shift
    pub(super) fn end_shift(&mut self, success: bool) {
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
            .unwrap_or_else(|| {
                let initial = self
                    .game_data
                    .as_ref()
                    .map(|data| data.constants.game_constants.initial_time)
                    .unwrap_or(0);
                minutes_on_the_clock(initial, self.game_state.time_remaining)
            });

        self.player_stats
            .record_shift_completion(&self.game_state, actually_successful, play_time);
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

        // Fold the night's standings back into the save. The shift plays
        // against the working copy on `game_state`; without this, every
        // reputation earned died with the run.
        self.player_stats.passenger_reputation = self.game_state.passenger_reputation.clone();

        // Add leaderboard entry
        if let Some(ref data) = self.game_data {
            #[cfg(not(target_arch = "wasm32"))]
            let date_str = {
                use chrono::Local;
                Local::now().format("%Y-%m-%d %H:%M").to_string()
            };
            // The web build has no wall clock, and every row reading "Session"
            // told the player nothing — the leaderboard's whole job is
            // telling ten runs apart. The shift counter is already
            // incremented by `record_shift_completion` above and does the
            // same work: it orders the entries and distinguishes them.
            #[cfg(target_arch = "wasm32")]
            let date_str = format!("Shift {}", self.player_stats.total_shifts_completed);

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
        let unlocked = self.player_stats.check_achievements(Some(FinishedShift::of(
            &self.game_state,
            actually_successful,
        )));
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
    pub(super) fn pay_achievement_rewards(&mut self, unlocked: &[String]) {
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
}

#[cfg(test)]
mod tests {
    use super::minutes_on_the_clock;
    use crate::data::loader::{load_constants, load_item_catalog};
    use crate::engine::ItemService;
    use crate::state::GameState;

    /// The precondition the clock arithmetic has to survive: a shift can hold
    /// more time than it started with.
    #[test]
    fn a_time_bonus_can_push_the_clock_past_the_shift_length() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let initial = constants.game_constants.initial_time;

        let generous = catalog
            .names()
            .into_iter()
            .map(|name| catalog.create_item(&name, "test", 0.0))
            .find(|item| {
                item.can_use
                    && item.effects.iter().any(|effect| {
                        matches!(effect.effect_type, crate::data::ItemEffectType::TimeBonus)
                    })
            })
            .expect("an item that adds time");

        let mut state = GameState::new(0.0, &constants.game_constants);
        state.current_passenger = None;
        assert_eq!(state.time_remaining, initial);
        for effect in &generous.effects {
            ItemService::apply_item_effect(effect, &mut state, &constants.reputation, 0.0);
        }
        assert!(
            state.time_remaining > initial,
            "{} did not push the clock past the shift length, so this test              no longer guards the underflow",
            generous.name
        );

        assert_eq!(
            minutes_on_the_clock(initial, state.time_remaining),
            0,
            "a clock fuller than the shift length must read as no time worked"
        );
    }

    /// And the ordinary case still measures the night.
    #[test]
    fn the_clock_measures_what_the_night_has_spent() {
        assert_eq!(minutes_on_the_clock(480, 480), 0);
        assert_eq!(minutes_on_the_clock(480, 300), 180);
        assert_eq!(minutes_on_the_clock(480, 0), 480);
    }
}

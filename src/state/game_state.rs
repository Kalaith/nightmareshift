//! Core game state structure.

use crate::data::*;
use std::collections::HashMap;

/// What is happening inside a shift.
///
/// This deliberately does not mirror `Screen`. It used to carry `MainMenu`,
/// `SkillTree`, `Almanac` and `Leaderboard` variants that existed only to
/// shadow the screen the player was on, kept in step by hand in
/// `change_screen` while five other places assigned `screen` directly and
/// could leave the two disagreeing. Nothing ever read them. Navigation is
/// `Screen`'s job; this describes the night.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GamePhase {
    /// Before a shift has begun — boot, menus, and the meta screens.
    #[default]
    Loading,
    Briefing,
    Waiting,
    RideRequest,
    Driving,
    Interaction,
    GuidelineDecision,
    DropOff,
    GameOver,
    Success,
}

/// Driving sub-phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrivingPhase {
    Pickup,
    Destination,
}

/// Need stage progression
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum NeedStage {
    #[default]
    Calm,
    Warning,
    Critical,
    Meltdown,
}

/// Relationship level with a passenger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RelationshipLevel {
    Hostile,
    #[default]
    Neutral,
    Friendly,
    Trusted,
}

/// Current ride information
#[derive(Debug, Clone)]
pub struct CurrentRide {
    pub passenger: Passenger,
    pub pickup_location: String,
    pub destination_location: String,
    pub route_type: Option<RouteType>,
    pub driving_phase: DrivingPhase,
    pub start_time: f64,
}

/// Passenger need state tracking
#[derive(Debug, Clone)]
pub struct PassengerNeedState {
    pub level: u32,
    pub stage: NeedStage,
    pub stability: f32,
    pub last_updated: f64,
    pub revealed_stages: HashMap<NeedStage, bool>,
    pub profile: PassengerStateProfile,
}

impl PassengerNeedState {
    /// Create from passenger's state profile
    pub fn from_passenger(passenger: &Passenger, current_time: f64) -> Option<Self> {
        let profile = passenger.state_profile.clone()?;
        let stage = Self::calculate_stage(profile.initial_level, &profile.thresholds);

        Some(Self {
            level: profile.initial_level.clamp(0, 100),
            stage,
            stability: 1.0 - (profile.initial_level as f32 / 100.0),
            last_updated: current_time,
            revealed_stages: HashMap::new(),
            profile,
        })
    }

    /// Calculate stage from level and thresholds
    pub fn calculate_stage(level: u32, thresholds: &NeedThresholds) -> NeedStage {
        if level >= thresholds.meltdown {
            NeedStage::Meltdown
        } else if level >= thresholds.critical {
            NeedStage::Critical
        } else if level >= thresholds.warning {
            NeedStage::Warning
        } else {
            NeedStage::Calm
        }
    }
}

/// A detected behavioral tell
#[derive(Debug, Clone)]
pub struct DetectedTell {
    pub tell: PassengerTell,
    pub passenger_id: u32,
    pub detection_time: f64,
    pub player_noticed: bool,
    pub related_guideline: Option<u32>,
    pub exception_id: Option<String>,
}

/// Passenger reputation tracking
#[derive(Debug, Clone, Default)]
pub struct PassengerReputation {
    pub interactions: u32,
    pub positive_choices: u32,
    pub negative_choices: u32,
    pub last_encounter: f64,
    pub relationship_level: RelationshipLevel,
}

impl PassengerReputation {
    /// Update reputation based on choice outcome
    pub fn update(&mut self, positive: bool, current_time: f64, constants: &ReputationConstants) {
        self.interactions += 1;
        self.last_encounter = current_time;

        if positive {
            self.positive_choices += 1;
        } else {
            self.negative_choices += 1;
        }

        // Recalculate relationship level
        let ratio = if self.interactions > 0 {
            self.positive_choices as f32 / self.interactions as f32
        } else {
            0.5
        };

        self.relationship_level = if ratio >= constants.trusted_ratio
            && self.interactions >= constants.minimum_interactions_for_trusted
        {
            RelationshipLevel::Trusted
        } else if ratio >= constants.friendly_ratio {
            RelationshipLevel::Friendly
        } else if ratio <= constants.hostile_ratio {
            RelationshipLevel::Hostile
        } else {
            RelationshipLevel::Neutral
        };
    }

    /// Get fare multiplier based on relationship
    pub fn fare_multiplier(&self, constants: &ReputationConstants) -> f32 {
        match self.relationship_level {
            RelationshipLevel::Trusted => constants.trusted_fare_mult,
            RelationshipLevel::Friendly => constants.friendly_fare_mult,
            RelationshipLevel::Hostile => constants.hostile_fare_mult,
            RelationshipLevel::Neutral => constants.default_fare_mult,
        }
    }

    /// Get risk modifier based on relationship
    pub fn risk_modifier(&self, constants: &ReputationConstants) -> i32 {
        match self.relationship_level {
            RelationshipLevel::Trusted => constants.trusted_risk_mod,
            RelationshipLevel::Hostile => constants.hostile_risk_mod,
            _ => 0,
        }
    }
}

/// Route history entry
#[derive(Debug, Clone)]

pub struct RouteHistoryEntry {
    pub route_type: RouteType,
    pub driving_phase: DrivingPhase,
    pub fuel_cost: u32,
    pub time_cost: u32,
    pub risk_level: u32,
    pub passenger_id: Option<u32>,
    pub timestamp: f64,
}

/// Consecutive route streak tracking
#[derive(Debug, Clone, Default)]
pub struct RouteStreak {
    pub route_type: RouteType,
    pub count: u32,
}

/// What a finished shift paid into the meta-progression currencies.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetaPayout {
    pub bank: u32,
    pub lore: u32,
    /// Extra bank paid for finishing the whole run, separate from the night.
    pub run_bonus_bank: u32,
    pub run_bonus_lore: u32,
}

impl MetaPayout {
    /// True when the run-completion bonus was part of this payout.
    pub fn completed_a_run(&self) -> bool {
        self.run_bonus_bank > 0 || self.run_bonus_lore > 0
    }
}

/// A rule imposed for a limited number of rides.
#[derive(Debug, Clone)]
pub struct TemporaryRuleState {
    pub rule_id: u32,
    pub rides_remaining: u32,
}

/// Guideline decision history
#[derive(Debug, Clone)]

pub struct GuidelineDecision {
    pub guideline_id: u32,
    pub passenger_id: u32,
    pub action: GuidelineAction,
    pub was_correct: bool,
    pub tells_present: Vec<PassengerTell>,
    pub timestamp: f64,
}

/// Guideline action choice
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidelineAction {
    Follow,
    Break,
}

/// Ride completion data for display
#[derive(Debug, Clone)]
pub struct RideCompletion {
    pub passenger: Passenger,
    pub fare_earned: u32,
    pub items_received: Vec<InventoryItem>,
    pub backstory_unlocked: Option<(String, String)>, // (name, backstory)
}

/// Dialogue display
#[derive(Debug, Clone)]

pub struct CurrentDialogue {
    pub text: String,
    pub speaker: DialogueSpeaker,
    pub timestamp: f64,
}

/// The result of handing a passenger something.
#[derive(Debug, Clone)]
pub struct TradeOutcome {
    pub text: String,
    /// Whether the item was on the passenger's `wantedItems` list — the only
    /// case that pays standing and settles their need.
    pub was_wanted: bool,
}

/// Who is speaking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum DialogueSpeaker {
    Passenger,
    Driver,
    Narrator,
}

/// Complete game state
#[derive(Debug, Clone)]
pub struct GameState {
    // Core resources
    pub fuel: f32,
    /// Maximum fuel capacity for this shift (100 plus any capacity skills).
    pub max_fuel: f32,
    /// Current night within the run (1-based).
    pub night: u32,
    /// True once the final night of a run has been survived.
    pub run_complete: bool,
    /// Whether the one-shot end-of-shift warning has already been given.
    pub shift_end_warning_shown: bool,
    /// Bank and lore this shift paid into the meta-progression, and any
    /// separate bonus for completing the run, so the outcome screen can say
    /// what the night was worth beyond its own fare.
    pub shift_payout: MetaPayout,
    pub earnings: u32,
    pub time_remaining: u32,
    pub rides_completed: u32,
    pub rules_violated: u32,

    // Current shift
    pub current_rules: Vec<Rule>,
    pub hidden_rules: Vec<Rule>,
    pub revealed_hidden_rules: Vec<Rule>,
    /// Rules a passenger imposed for a few rides, and how many are left.
    pub temporary_rules: Vec<TemporaryRuleState>,
    pub current_guidelines: Vec<Guideline>,
    pub inventory: Vec<InventoryItem>,
    pub current_passenger: Option<Passenger>,
    pub current_passenger_dialogue: Option<String>,
    pub current_ride: Option<CurrentRide>,
    pub current_event: Option<MidRideEvent>, // Mid-ride event
    pub game_phase: GamePhase,
    pub driving_phase: Option<DrivingPhase>,
    pub used_passengers: Vec<u32>,
    pub shift_start_time: Option<f64>,
    pub difficulty_level: u32,

    // Passenger state machine
    pub current_passenger_need_state: Option<PassengerNeedState>,
    pub detected_tells: Vec<DetectedTell>,
    pub player_trust: f32,
    pub decision_history: Vec<GuidelineDecision>,

    // Route tracking
    pub route_history: Vec<RouteHistoryEntry>,
    pub consecutive_route_streak: Option<RouteStreak>,

    // Weather system
    pub current_weather: WeatherCondition,
    pub time_of_day: TimeOfDay,
    pub season: Season,
    pub environmental_hazards: Vec<EnvironmentalHazard>,

    // Persistence
    pub passenger_reputation: HashMap<u32, PassengerReputation>,
    pub minimum_earnings: u32,

    // Item effects tracking
    pub rule_immunity_charges: u32,
    pub supernatural_protection: u32,
    pub curse_danger_bonus: u32,
    pub pending_trade: Option<(String, InventoryItem)>, // (passenger_name, offered_item)
    /// What the last swap did, for the drop-off screen to say out loud.
    ///
    /// Completing a trade wrote its result into `current_dialogue`, which only
    /// the driving screen renders — and accepting the next ride overwrites it.
    /// So the reputation and the relief a wanted item earns were paid in
    /// silence, and handing over the wrong thing said nothing at all. This is
    /// read on the screen the trade actually happens on.
    pub trade_outcome: Option<TradeOutcome>,

    // UI state
    pub current_dialogue: Option<CurrentDialogue>,
    pub pending_route_dialogue: Option<String>,
    pub last_ride_completion: Option<RideCompletion>,
    pub game_over_reason: Option<String>,

    // Guideline decision state
    pub active_guideline: Option<Guideline>,
    pub guideline_decision_start_time: Option<f64>,
    pub guideline_time_remaining: f32,
}

impl GameState {
    /// Create a new game state with initial values from constants
    pub fn new(_current_time: f64, constants: &GameConstants) -> Self {
        Self {
            fuel: constants.initial_fuel as f32,
            max_fuel: 100.0,
            night: 1,
            run_complete: false,
            shift_end_warning_shown: false,
            shift_payout: MetaPayout::default(),
            earnings: 0,
            time_remaining: constants.initial_time,
            rides_completed: 0,
            rules_violated: 0,
            current_rules: Vec::new(),
            hidden_rules: Vec::new(),
            revealed_hidden_rules: Vec::new(),
            temporary_rules: Vec::new(),
            current_guidelines: Vec::new(),
            inventory: Vec::new(),
            current_passenger: None,
            current_passenger_dialogue: None,
            current_ride: None,
            current_event: None,
            game_phase: GamePhase::Loading,
            driving_phase: None,
            used_passengers: Vec::new(),
            shift_start_time: None,
            difficulty_level: 0,
            current_passenger_need_state: None,
            detected_tells: Vec::new(),
            player_trust: 0.5,
            decision_history: Vec::new(),
            route_history: Vec::new(),
            consecutive_route_streak: None,
            current_weather: WeatherCondition::default(),
            time_of_day: TimeOfDay::default(),
            season: Season::default(),
            environmental_hazards: Vec::new(),
            passenger_reputation: HashMap::new(),
            minimum_earnings: constants.minimum_earnings,
            rule_immunity_charges: 0,
            supernatural_protection: 0,
            curse_danger_bonus: 0,
            pending_trade: None,
            trade_outcome: None,
            current_dialogue: None,
            pending_route_dialogue: None,
            last_ride_completion: None,
            game_over_reason: None,
            active_guideline: None,
            guideline_decision_start_time: None,
            guideline_time_remaining: 30.0,
        }
    }

    /// Reset for a new shift using constants
    pub fn reset_for_new_shift(&mut self, current_time: f64, constants: &GameConstants) {
        self.fuel = constants.initial_fuel as f32;
        self.max_fuel = 100.0;
        self.earnings = 0;
        self.time_remaining = constants.initial_time;
        self.minimum_earnings = constants.minimum_earnings;
        self.rides_completed = 0;
        self.rules_violated = 0;
        self.current_rules.clear();
        self.hidden_rules.clear();
        self.revealed_hidden_rules.clear();
        self.temporary_rules.clear();
        self.current_guidelines.clear();
        self.current_passenger = None;
        self.current_passenger_dialogue = None;
        self.current_ride = None;
        self.current_event = None;
        self.game_phase = GamePhase::Waiting;
        self.driving_phase = None;
        self.used_passengers.clear();
        self.shift_start_time = Some(current_time);
        self.shift_end_warning_shown = false;
        self.shift_payout = MetaPayout::default();
        self.current_passenger_need_state = None;
        self.detected_tells.clear();
        self.route_history.clear();
        self.consecutive_route_streak = None;
        self.environmental_hazards.clear();
        self.player_trust = 0.5;
        self.rule_immunity_charges = 0;
        self.supernatural_protection = 0;
        self.curse_danger_bonus = 0;
        self.pending_trade = None;
        self.trade_outcome = None;
        self.current_dialogue = None;
        self.pending_route_dialogue = None;
        self.last_ride_completion = None;
        self.game_over_reason = None;
        self.active_guideline = None;
        self.guideline_decision_start_time = None;
        self.guideline_time_remaining = 30.0;
    }

    /// Check if time is running out
    pub fn is_time_critical(&self, constants: &ConstantsData) -> bool {
        self.time_remaining <= constants.timing.critical_time_threshold
    }

    /// Announce, once per shift, that the night is nearly over.
    ///
    /// `TIMING.SHIFT_END_WARNING_THRESHOLD` is authored at sixty minutes and
    /// was read by nothing; only `CRITICAL_TIME_THRESHOLD` reached the game,
    /// and at fifteen minutes it recolours the clock too late to act on —
    /// every route costs between eighteen and thirty-two. This fires while
    /// there is still a decision to make: one more fare, or bank the night.
    pub fn take_shift_end_warning(&mut self, constants: &ConstantsData) -> bool {
        if self.shift_end_warning_shown
            || self.time_remaining > constants.timing.shift_end_warning_threshold
        {
            return false;
        }
        self.shift_end_warning_shown = true;
        true
    }

    /// Check if shift should end
    pub fn should_end_shift(&self) -> bool {
        self.time_remaining == 0 || self.fuel <= 0.0
    }

    /// Reveal a hidden rule and move it into the active visible rules.
    pub fn reveal_hidden_rule(&mut self, rule_id: u32) -> Option<Rule> {
        let index = self
            .hidden_rules
            .iter()
            .position(|rule| rule.id == rule_id)?;
        let rule = self.hidden_rules.remove(index);
        if !self.revealed_hidden_rules.iter().any(|r| r.id == rule.id) {
            self.revealed_hidden_rules.push(rule.clone());
        }
        if !self.current_rules.iter().any(|r| r.id == rule.id) {
            self.current_rules.push(rule.clone());
        }
        Some(rule)
    }

    /// Clamp player trust after a gameplay outcome.
    pub fn adjust_player_trust(&mut self, delta: f32) {
        self.player_trust = (self.player_trust + delta).clamp(0.0, 1.0);
    }

    /// Update consecutive route streak
    pub fn update_route_streak(&mut self, route: RouteType) {
        if let Some(ref mut streak) = self.consecutive_route_streak {
            if streak.route_type == route {
                streak.count += 1;
            } else {
                *streak = RouteStreak {
                    route_type: route,
                    count: 1,
                };
            }
        } else {
            self.consecutive_route_streak = Some(RouteStreak {
                route_type: route,
                count: 1,
            });
        }
    }

    /// Get passenger reputation, creating if needed
    pub fn get_passenger_reputation(&mut self, passenger_id: u32) -> &mut PassengerReputation {
        self.passenger_reputation.entry(passenger_id).or_default()
    }

    /// Calculate current score
    pub fn calculate_score(&self, constants: &ConstantsData) -> u32 {
        let base = self.earnings;
        let ride_bonus = self.rides_completed * constants.scoring.ride_bonus;
        let violation_penalty = self.rules_violated * constants.scoring.rule_violation_penalty;

        // Time left over is only worth something if the night was actually
        // worked. Paid unconditionally it rewards not driving: a shift that
        // ends on the first fare keeps the whole clock, and at two points a
        // minute that is 960 — which outscored real shifts on the
        // leaderboard, where an eight-hour night with nine passengers and no
        // violations sat below two instant losses.
        let time_bonus = if self.earnings >= self.minimum_earnings {
            self.time_remaining * constants.scoring.time_bonus_multiplier
        } else {
            0
        };

        (base + ride_bonus + time_bonus).saturating_sub(violation_penalty)
    }
}

#[cfg(test)]
mod score_tests {
    use super::*;
    use crate::data::loader::load_constants;

    fn shift(earnings: u32, rides: u32, time_left: u32, violations: u32) -> GameState {
        let constants = load_constants();
        let mut state = GameState::new(0.0, &constants.game_constants);
        state.earnings = earnings;
        state.rides_completed = rides;
        state.time_remaining = time_left;
        state.rules_violated = violations;
        state
    }

    /// A held decision keeps whatever is left on its clock.
    ///
    /// The countdown is `30 - (now - start)`, so a pause that does not move
    /// the start is not a pause at all — the timer ran behind the menu and
    /// forced the choice. Pushing the start by the same amount as the wall
    /// clock leaves the remaining time unchanged.
    #[test]
    fn holding_the_start_time_holds_the_countdown() {
        let remaining = |start: f64, now: f64| (30.0 - (now - start) as f32).max(0.0);

        let start = 100.0;
        let opened_at = 110.0;
        assert_eq!(remaining(start, opened_at), 20.0);

        // Six seconds spent reading the pause menu, the start pushed with it.
        let paused_for = 6.0;
        let held_start = start + paused_for;
        assert_eq!(remaining(held_start, opened_at + paused_for), 20.0);

        // Without the push those six seconds would have come off the clock.
        assert_eq!(remaining(start, opened_at + paused_for), 14.0);
    }

    /// The run bonus is what distinguishes finishing a campaign from
    /// surviving another night, so the outcome screen keys its extra line on
    /// this. A night's own banking must not trip it.
    #[test]
    fn only_a_run_bonus_counts_as_completing_a_run() {
        let nightly = MetaPayout {
            bank: 300,
            lore: 12,
            ..MetaPayout::default()
        };
        assert!(!nightly.completed_a_run());

        let finished = MetaPayout {
            bank: 300,
            lore: 12,
            run_bonus_bank: 1500,
            run_bonus_lore: 15,
        };
        assert!(finished.completed_a_run());
    }

    /// A fresh shift starts with nothing recorded, so a night cannot inherit
    /// the previous one's payout on the screen that reports it.
    #[test]
    fn a_new_shift_clears_the_recorded_payout() {
        let constants = load_constants();
        let mut state = shift(400, 8, 100, 0);
        state.shift_payout = MetaPayout {
            bank: 200,
            lore: 9,
            run_bonus_bank: 1500,
            run_bonus_lore: 15,
        };
        state.reset_for_new_shift(0.0, &constants.game_constants);
        assert_eq!(state.shift_payout.bank, 0);
        assert_eq!(state.shift_payout.lore, 0);
        assert!(!state.shift_payout.completed_a_run());
    }

    /// A night that ended before it started must not outscore one that was
    /// worked. The time bonus used to be paid regardless, so keeping the
    /// whole clock by losing the first fare scored 960 — above real shifts
    /// on the leaderboard.
    #[test]
    fn an_instant_loss_does_not_outscore_a_worked_night() {
        let constants = load_constants();
        let instant_loss = shift(0, 0, constants.game_constants.initial_time, 0);
        let worked = shift(200, 6, 90, 1);
        assert!(
            worked.calculate_score(&constants) > instant_loss.calculate_score(&constants),
            "a worked night scored {} against {} for losing immediately",
            worked.calculate_score(&constants),
            instant_loss.calculate_score(&constants)
        );
    }

    /// Falling short of the quota forfeits the time bonus entirely, so what
    /// is left on the clock cannot carry a failed night.
    #[test]
    fn time_left_pays_only_when_the_quota_is_met() {
        let constants = load_constants();
        let quota = constants.game_constants.minimum_earnings;

        let short = shift(quota - 1, 3, 200, 0);
        let met = shift(quota, 3, 200, 0);
        let expected_step = constants.scoring.time_bonus_multiplier * 200 + 1;
        assert_eq!(
            met.calculate_score(&constants) - short.calculate_score(&constants),
            expected_step,
            "crossing the quota did not turn the time bonus on"
        );
    }

    /// Among nights that met the quota, finishing sooner still scores higher
    /// — the bonus keeps the meaning it was added for.
    #[test]
    fn finishing_sooner_still_pays() {
        let constants = load_constants();
        let quota = constants.game_constants.minimum_earnings;
        let brisk = shift(quota + 50, 5, 180, 0);
        let slow = shift(quota + 50, 5, 40, 0);
        assert!(brisk.calculate_score(&constants) > slow.calculate_score(&constants));
    }

    /// Violations still cost, and cannot push a score below zero.
    #[test]
    fn violations_cost_without_underflowing() {
        let constants = load_constants();
        let reckless = shift(0, 0, 0, 99);
        assert_eq!(reckless.calculate_score(&constants), 0);
    }
}

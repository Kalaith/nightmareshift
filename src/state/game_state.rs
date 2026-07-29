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
    pub earnings: u32,
    pub time_remaining: u32,
    pub rides_completed: u32,
    pub rules_violated: u32,

    // Current shift
    pub current_rules: Vec<Rule>,
    pub hidden_rules: Vec<Rule>,
    pub revealed_hidden_rules: Vec<Rule>,
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
            earnings: 0,
            time_remaining: constants.initial_time,
            rides_completed: 0,
            rules_violated: 0,
            current_rules: Vec::new(),
            hidden_rules: Vec::new(),
            revealed_hidden_rules: Vec::new(),
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
        let time_bonus = self.time_remaining * constants.scoring.time_bonus_multiplier;
        let violation_penalty = self.rules_violated * constants.scoring.rule_violation_penalty;

        (base + ride_bonus + time_bonus).saturating_sub(violation_penalty)
    }
}

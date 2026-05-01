//! Game constants loaded from JSON.
//!
//! All game balance values are loaded from assets/constants.json.
//! Do not hardcode values - add them to the JSON file instead.

use serde::{Deserialize, Serialize};

/// Storage keys for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageKeys {
    #[serde(rename = "PLAYER_STATS")]
    pub player_stats: String,
    #[serde(rename = "LEADERBOARD")]
    pub leaderboard: String,
    #[serde(rename = "SAVE_GAME")]
    pub save_game: String,
    #[serde(rename = "BACKSTORY_PROGRESS")]
    pub backstory_progress: String,
}

/// Core game constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstants {
    #[serde(rename = "INITIAL_FUEL")]
    pub initial_fuel: u32,
    #[serde(rename = "INITIAL_TIME")]
    pub initial_time: u32,
    #[serde(rename = "SURVIVAL_BONUS")]
    pub survival_bonus: u32,
    #[serde(rename = "BACKSTORY_UNLOCK_FIRST")]
    pub backstory_unlock_first: f32,
    #[serde(rename = "BACKSTORY_UNLOCK_REPEAT")]
    pub backstory_unlock_repeat: f32,
    #[serde(rename = "MINIMUM_EARNINGS")]
    pub minimum_earnings: u32,
    #[serde(rename = "FUEL_COST_SHORTCUT")]
    pub fuel_cost_shortcut: u32,
    #[serde(rename = "FUEL_COST_NORMAL")]
    pub fuel_cost_normal: u32,
    #[serde(rename = "FUEL_COST_SCENIC")]
    pub fuel_cost_scenic: u32,
    #[serde(rename = "FUEL_COST_POLICE")]
    pub fuel_cost_police: u32,
    #[serde(rename = "TIME_COST_SHORTCUT")]
    pub time_cost_shortcut: u32,
    #[serde(rename = "TIME_COST_NORMAL")]
    pub time_cost_normal: u32,
    #[serde(rename = "TIME_COST_SCENIC")]
    pub time_cost_scenic: u32,
    #[serde(rename = "TIME_COST_POLICE")]
    pub time_cost_police: u32,
    #[serde(rename = "ROUTE_PREFERENCE_STRESS_SCALE")]
    pub route_preference_stress_scale: f32,
    #[serde(rename = "RISK_NORMAL")]
    pub risk_normal: u32,
    #[serde(rename = "RISK_SHORTCUT")]
    pub risk_shortcut: u32,
    #[serde(rename = "RISK_SCENIC")]
    pub risk_scenic: u32,
    #[serde(rename = "RISK_POLICE")]
    pub risk_police: u32,
}

/// Fuel system thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelConstants {
    #[serde(rename = "LOW_FUEL_WARNING")]
    pub low_fuel_warning: u32,
    #[serde(rename = "CRITICAL_FUEL")]
    pub critical_fuel: u32,
    #[serde(rename = "EMPTY_TANK")]
    pub empty_tank: u32,
    #[serde(rename = "FUEL_CHECK_MINIMUM")]
    pub fuel_check_minimum: u32,
    #[serde(rename = "COST_PER_PERCENT")]
    pub cost_per_percent: f32,
}

/// Timing constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConstants {
    #[serde(rename = "RIDE_REQUEST_BASE_DELAY_MS")]
    pub ride_request_base_delay_ms: u32,
    #[serde(rename = "RIDE_REQUEST_RANDOM_DELAY_MS")]
    pub ride_request_random_delay_ms: u32,
    #[serde(rename = "PASSENGER_INTERACTION_DELAY_MS")]
    pub passenger_interaction_delay_ms: u32,
    #[serde(rename = "DIALOGUE_DISPLAY_DELAY_MS")]
    pub dialogue_display_delay_ms: u32,
    #[serde(rename = "GAME_START_DELAY_MS")]
    pub game_start_delay_ms: u32,
    #[serde(rename = "SHIFT_END_WARNING_THRESHOLD")]
    pub shift_end_warning_threshold: u32,
    #[serde(rename = "CRITICAL_TIME_THRESHOLD")]
    pub critical_time_threshold: u32,
}

/// Probability constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbabilityConstants {
    #[serde(rename = "SUPERNATURAL_ENCOUNTER")]
    pub supernatural_encounter: f32,
    #[serde(rename = "HIGH_RISK_ENCOUNTER")]
    pub high_risk_encounter: f32,
    #[serde(rename = "ITEM_DROP")]
    pub item_drop: f32,
    #[serde(rename = "HIDDEN_RULE_VIOLATION")]
    pub hidden_rule_violation: f32,
    #[serde(rename = "RELATED_PASSENGER_SPAWN")]
    pub related_passenger_spawn: f32,
    #[serde(rename = "TRADE_OFFER_CHANCE")]
    pub trade_offer_chance: f32,
}

/// Risk level thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConstants {
    #[serde(rename = "SAFE")]
    pub safe: u32,
    #[serde(rename = "LOW_RISK")]
    pub low_risk: u32,
    #[serde(rename = "MEDIUM_RISK")]
    pub medium_risk: u32,
    #[serde(rename = "HIGH_RISK")]
    pub high_risk: u32,
    #[serde(rename = "EXTREME_RISK")]
    pub extreme_risk: u32,
    #[serde(rename = "MAX_RISK_LEVEL")]
    pub max_risk_level: u32,
    #[serde(rename = "SUPERNATURAL_THRESHOLD")]
    pub supernatural_threshold: u32,
}

/// Reputation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConstants {
    #[serde(rename = "TRUSTED_RATIO")]
    pub trusted_ratio: f32,
    #[serde(rename = "FRIENDLY_RATIO")]
    pub friendly_ratio: f32,
    #[serde(rename = "HOSTILE_RATIO")]
    pub hostile_ratio: f32,
    #[serde(rename = "MINIMUM_INTERACTIONS_FOR_TRUSTED")]
    pub minimum_interactions_for_trusted: u32,
    #[serde(rename = "TRUSTED_FARE_MULT")]
    pub trusted_fare_mult: f32,
    #[serde(rename = "FRIENDLY_FARE_MULT")]
    pub friendly_fare_mult: f32,
    #[serde(rename = "HOSTILE_FARE_MULT")]
    pub hostile_fare_mult: f32,
    #[serde(rename = "DEFAULT_FARE_MULT")]
    pub default_fare_mult: f32,
    #[serde(rename = "TRUSTED_RISK_MOD")]
    pub trusted_risk_mod: i32,
    #[serde(rename = "HOSTILE_RISK_MOD")]
    pub hostile_risk_mod: i32,
}

/// Route fare multipliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteFareConstants {
    #[serde(rename = "SHORTCUT")]
    pub shortcut: f32,
    #[serde(rename = "NORMAL")]
    pub normal: f32,
    #[serde(rename = "SCENIC")]
    pub scenic: f32,
    #[serde(rename = "POLICE")]
    pub police: f32,
}

/// Consecutive route penalties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsecutiveRouteConstants {
    #[serde(rename = "WARNING_THRESHOLD")]
    pub warning_threshold: u32,
    #[serde(rename = "VIOLATION_THRESHOLD")]
    pub violation_threshold: u32,
    #[serde(rename = "PENALTY_PER_REPEAT")]
    pub penalty_per_repeat: f32,
    #[serde(rename = "RISK_INCREASE_PER_REPEAT")]
    pub risk_increase_per_repeat: u32,
}

/// Route variation bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteVariationConstants {
    #[serde(rename = "MINIMUM_FUEL_COST")]
    pub minimum_fuel_cost: u32,
    #[serde(rename = "MINIMUM_TIME_COST")]
    pub minimum_time_cost: u32,
}

/// Scoring multipliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConstants {
    #[serde(rename = "TIME_BONUS_MULTIPLIER")]
    pub time_bonus_multiplier: u32,
    #[serde(rename = "RIDE_BONUS")]
    pub ride_bonus: u32,
    #[serde(rename = "RULE_VIOLATION_PENALTY")]
    pub rule_violation_penalty: u32,
    #[serde(rename = "MAX_DIFFICULTY")]
    pub max_difficulty: u32,
    #[serde(rename = "EXPERIENCE_PER_LEVEL")]
    pub experience_per_level: u32,
    #[serde(rename = "SURVIVAL_BONUS")]
    pub survival_bonus: u32,
}

/// Rarity spawn weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityWeights {
    pub common: u32,
    #[serde(default)]
    pub uncommon: u32,
    pub rare: u32,
    pub legendary: u32,
}

/// Screen identifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screens {
    #[serde(rename = "LOADING")]
    pub loading: String,
    #[serde(rename = "LEADERBOARD")]
    pub leaderboard: String,
    #[serde(rename = "BRIEFING")]
    pub briefing: String,
    #[serde(rename = "GAME")]
    pub game: String,
    #[serde(rename = "GAME_OVER")]
    pub game_over: String,
    #[serde(rename = "SUCCESS")]
    pub success: String,
    #[serde(rename = "SKILL_TREE")]
    pub skill_tree: String,
    #[serde(rename = "ALMANAC")]
    pub almanac: String,
}

/// Root constants structure matching constants.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantsData {
    #[serde(rename = "STORAGE_KEYS")]
    pub storage_keys: StorageKeys,
    #[serde(rename = "GAME_CONSTANTS")]
    pub game_constants: GameConstants,
    #[serde(rename = "FUEL")]
    pub fuel: FuelConstants,
    #[serde(rename = "TIMING")]
    pub timing: TimingConstants,
    #[serde(rename = "PROBABILITIES")]
    pub probabilities: ProbabilityConstants,
    #[serde(rename = "RISK")]
    pub risk: RiskConstants,
    #[serde(rename = "REPUTATION")]
    pub reputation: ReputationConstants,
    #[serde(rename = "ROUTE_FARES")]
    pub route_fares: RouteFareConstants,
    #[serde(rename = "CONSECUTIVE_ROUTE")]
    pub consecutive_route: ConsecutiveRouteConstants,
    #[serde(rename = "ROUTE_VARIATIONS")]
    pub route_variations: RouteVariationConstants,
    #[serde(rename = "SCORING")]
    pub scoring: ScoringConstants,
    #[serde(rename = "RARITY_WEIGHTS")]
    pub rarity_weights: RarityWeights,
    #[serde(rename = "SCREENS")]
    pub screens: Screens,
}

// No default() implementation - all values MUST come from constants.json
// If JSON fails to parse, it's a development error that should fail compilation

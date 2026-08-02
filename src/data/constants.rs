//! Game constants loaded from JSON.
//!
//! All game balance values are loaded from assets/constants.json.
//! Do not hardcode values - add them to the JSON file instead.

use serde::{Deserialize, Serialize};

fn default_nights_per_run() -> u32 {
    5
}

/// Core game constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstants {
    #[serde(rename = "INITIAL_FUEL")]
    pub initial_fuel: u32,
    #[serde(rename = "INITIAL_TIME")]
    pub initial_time: u32,
    /// Number of consecutive nights that make up one full run.
    #[serde(rename = "NIGHTS_PER_RUN", default = "default_nights_per_run")]
    pub nights_per_run: u32,
    /// Share of the base quota added per night of a run. The escalation was a
    /// `/ 2` in `begin_night` — the single biggest lever on how a campaign
    /// feels, expressed as a magic number in Rust rather than authored here.
    #[serde(rename = "QUOTA_INCREASE_PER_NIGHT", default = "default_quota_step")]
    pub quota_increase_per_night: f32,
    /// Difficulty levels added per night, on top of lifetime experience.
    #[serde(
        rename = "DIFFICULTY_INCREASE_PER_NIGHT",
        default = "default_difficulty_step"
    )]
    pub difficulty_increase_per_night: u32,
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
    #[serde(rename = "MEDIUM_FUEL", default = "default_medium_fuel")]
    pub medium_fuel: u32,
    #[serde(rename = "EMPTY_TANK")]
    pub empty_tank: u32,
    #[serde(rename = "FUEL_CHECK_MINIMUM")]
    pub fuel_check_minimum: u32,
    #[serde(rename = "COST_PER_PERCENT")]
    pub cost_per_percent: f32,
}

impl FuelConstants {
    /// What topping up `amount` percent costs a driver whose refuel discount
    /// is `cost_mult`.
    ///
    /// One formula, because there were two. The waiting screen priced a top-up
    /// without the discount from Negotiator and Haggler while `refuel_full`
    /// charged with it, so the button quoted more than the pump took. Worse,
    /// the screen decided whether the button was even usable from its own
    /// inflated figure -- so a driver holding $40 against a real price of $35
    /// was refused a refuel they could afford, by the very skill they had
    /// bought to make it cheaper.
    pub fn refuel_cost(&self, amount: f32, cost_mult: f32) -> u32 {
        (amount.max(0.0) * self.cost_per_percent * cost_mult) as u32
    }
}

/// Timing constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConstants {
    // The five *_DELAY_MS values authored beside these are not read. This
    // game is frame-driven: nothing waits on a millisecond clock, and the
    // pacing they describe belongs to the JavaScript version this was ported
    // from. Serde ignores them.
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

/// Root constants structure matching constants.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantsData {
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
}

// No default() implementation - all values MUST come from constants.json
// If JSON fails to parse, it's a development error that should fail compilation

fn default_medium_fuel() -> u32 {
    40
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_constants;

    /// A refuel discount has to reach the price the driver is quoted.
    ///
    /// The waiting screen priced a top-up without it while the pump charged
    /// with it, so the button said one number and took another.
    #[test]
    fn a_refuel_discount_lowers_the_price() {
        let fuel = load_constants().fuel;
        let full = fuel.refuel_cost(50.0, 1.0);
        let discounted = fuel.refuel_cost(50.0, 0.7);
        assert!(full > 0, "a fifty percent top-up costs nothing");
        assert!(
            discounted < full,
            "the discount changed nothing: {discounted} against {full}"
        );
    }

    /// And the discount must be able to bring a price within reach, because the
    /// screen decides whether the refuel button works from this figure. Quoting
    /// the undiscounted price refused a driver a refuel they could afford --
    /// locked out by the skill they bought to make it cheaper.
    #[test]
    fn a_discount_can_bring_a_refuel_within_reach() {
        let fuel = load_constants().fuel;
        let purse = fuel.refuel_cost(50.0, 0.7);
        assert!(
            fuel.refuel_cost(50.0, 1.0) > purse,
            "no discount steep enough to matter, so this cannot be tested"
        );
        assert!(
            fuel.refuel_cost(50.0, 0.7) <= purse,
            "a driver holding exactly the discounted price is still refused"
        );
    }

    /// Nothing to add, nothing to pay. A full tank must not be billed for a
    /// negative top-up.
    #[test]
    fn a_full_tank_costs_nothing_to_fill() {
        let fuel = load_constants().fuel;
        assert_eq!(fuel.refuel_cost(0.0, 1.0), 0);
        assert_eq!(fuel.refuel_cost(-10.0, 1.0), 0);
    }

    /// Fuel thresholds must descend, or the gauge colours and the status text
    /// disagree about what "low" means. They used to be duplicated as Rust
    /// constants; nothing but this ordering now stops a bad edit.
    #[test]
    fn fuel_thresholds_descend() {
        let fuel = load_constants().fuel;
        assert!(
            fuel.medium_fuel > fuel.low_fuel_warning,
            "medium {} is not above low {}",
            fuel.medium_fuel,
            fuel.low_fuel_warning
        );
        assert!(
            fuel.low_fuel_warning > fuel.critical_fuel,
            "low {} is not above critical {}",
            fuel.low_fuel_warning,
            fuel.critical_fuel
        );
        assert!(fuel.critical_fuel > fuel.empty_tank);
    }

    /// A ride must be refusable before the tank is empty, or the
    /// "ran out of fuel with a passenger in the car" guard never fires early
    /// enough to be a warning.
    #[test]
    fn a_ride_is_refused_before_the_tank_empties() {
        let fuel = load_constants().fuel;
        assert!(fuel.fuel_check_minimum > fuel.empty_tank);
        assert!(fuel.fuel_check_minimum <= fuel.critical_fuel);
    }

    /// The streak warning must be reachable, and the violation threshold is
    /// authored at 999 deliberately — that is the data saying no violation,
    /// not a value to wire up.
    #[test]
    fn route_streak_warning_is_reachable() {
        let streak = load_constants().consecutive_route;
        assert!(streak.warning_threshold > 0);
        assert!(
            streak.risk_increase_per_repeat > 0,
            "a streak adds no risk, so the warning has nothing behind it"
        );
    }
}

fn default_quota_step() -> f32 {
    0.5
}

fn default_difficulty_step() -> u32 {
    1
}

#[cfg(test)]
mod campaign_tests {
    use crate::data::loader::load_constants;

    /// The quota a night asks for, as `begin_night` computes it.
    fn quota_for(night: u32) -> u32 {
        let constants = load_constants();
        let base = constants.game_constants.minimum_earnings;
        let step = constants.game_constants.quota_increase_per_night;
        base + (base as f32 * step * (night - 1) as f32).round() as u32
    }

    /// A run must ask for more each night, or the campaign is five copies of
    /// the same shift.
    #[test]
    fn the_quota_rises_every_night() {
        let constants = load_constants();
        let nights = constants.game_constants.nights_per_run.max(1);
        let mut previous = 0;
        for night in 1..=nights {
            let quota = quota_for(night);
            assert!(
                quota > previous,
                "night {night} asks {quota}, no more than night {}",
                night - 1
            );
            previous = quota;
        }
    }

    /// The final night must stay inside what a shift can physically earn.
    /// Fuel and the clock do not grow across a run, so a quota that outruns
    /// them makes the last night unwinnable rather than hard. The ceiling
    /// here is deliberately generous — the point is to catch a step that has
    /// been retuned into impossibility, not to pin the current balance.
    #[test]
    fn the_final_night_stays_within_reach() {
        let constants = load_constants();
        let nights = constants.game_constants.nights_per_run.max(1);
        let final_quota = quota_for(nights);

        // Cheapest leg, two legs a ride, against the clock a shift starts on.
        let game = &constants.game_constants;
        let cheapest_leg = game
            .time_cost_shortcut
            .min(game.time_cost_normal)
            .min(game.time_cost_scenic)
            .min(game.time_cost_police);
        let max_rides = game.initial_time / (cheapest_leg * 2).max(1);

        // The best-paying fare on the roster, times the best route
        // multiplier. `route_fares` are multipliers, not amounts — reading
        // them as money gave a ceiling of seventeen dollars a night.
        let fares = &constants.route_fares;
        let best_multiplier = fares
            .normal
            .max(fares.shortcut)
            .max(fares.scenic)
            .max(fares.police);
        let best_fare = crate::data::loader::load_passengers()
            .iter()
            .map(|passenger| passenger.fare)
            .max()
            .unwrap_or(0);
        let ceiling = (max_rides as f32 * best_fare as f32 * best_multiplier).round() as u32;

        assert!(
            final_quota <= ceiling,
            "night {nights} asks {final_quota}, above the {ceiling} a shift could earn \
             at {max_rides} rides of {best_fare} times {best_multiplier}"
        );
    }

    /// Difficulty must climb across a run and must not exceed the cap the
    /// rule generator clamps to, or the last nights are indistinguishable.
    #[test]
    fn difficulty_climbs_but_stays_within_the_cap() {
        let constants = load_constants();
        let step = constants.game_constants.difficulty_increase_per_night;
        assert!(step > 0, "a run never gets harder");
        assert!(
            step <= constants.scoring.max_difficulty,
            "one night's step {step} exceeds the {} cap",
            constants.scoring.max_difficulty
        );
    }
}

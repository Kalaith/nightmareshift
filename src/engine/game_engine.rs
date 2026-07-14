//! Core game engine logic.

use crate::data::*;
use crate::state::*;

/// Result of generating shift rules
#[derive(Debug, Clone)]
pub struct ShiftRules {
    pub visible_rules: Vec<Rule>,
    pub hidden_rules: Vec<Rule>,
    pub difficulty_level: u32,
}

/// Result of rule evaluation
#[derive(Debug, Clone, Default)]
pub struct RuleEvaluationResult {
    pub violation: bool,
    pub rule: Option<Rule>,
    pub message: Option<String>,
    pub need_adjustment: i32,
    pub triggered_exception: Option<GuidelineException>,
}

/// Aggregated gameplay modifiers derived from the player's unlocked skills.
///
/// Computed once from the skill definitions plus the player's unlocked list, so
/// every skill's effect actually reaches the systems it names.
#[derive(Debug, Clone, Copy)]
pub struct SkillModifiers {
    /// Multiplier on route fuel cost (< 1.0 is cheaper).
    pub fuel_cost_mult: f32,
    /// Flat bonus to maximum fuel capacity.
    pub max_fuel_bonus: f32,
    /// Multiplier on fuel/time/risk added by environmental hazards (< 1.0 softer).
    pub hazard_mult: f32,
    /// Per-shift chance to reveal one hidden rule up front.
    pub reveal_hidden_chance: f32,
    /// Multiplier on every fare earned (> 1.0 pays more).
    pub fare_mult: f32,
    /// Multiplier on refuel cost (< 1.0 is a discount).
    pub refuel_cost_mult: f32,
    /// Protective wards granted at the start of each shift.
    pub bonus_protection: u32,
}

impl Default for SkillModifiers {
    fn default() -> Self {
        Self {
            fuel_cost_mult: 1.0,
            max_fuel_bonus: 0.0,
            hazard_mult: 1.0,
            reveal_hidden_chance: 0.0,
            fare_mult: 1.0,
            refuel_cost_mult: 1.0,
            bonus_protection: 0,
        }
    }
}

impl SkillModifiers {
    /// Build the active modifiers from the skill catalog and the player's
    /// unlocked-skill list. Unknown targets are ignored.
    pub fn from_unlocked(skills: &[Skill], unlocked: &[String]) -> Self {
        let mut m = Self::default();
        for skill in skills {
            if !unlocked.contains(&skill.id) {
                continue;
            }
            let v = skill.effect.value as f32;
            match skill.effect.target.as_str() {
                "fuel_consumption" => m.fuel_cost_mult *= v,
                "max_fuel" => m.max_fuel_bonus += v,
                "hazard_damage" => m.hazard_mult *= v,
                "reveal_hidden_chance" => m.reveal_hidden_chance += v,
                "fare_multiplier" => m.fare_mult *= v,
                "refuel_discount" => m.refuel_cost_mult *= (1.0 - v).max(0.0),
                "supernatural_protection" => m.bonus_protection += v as u32,
                _ => {}
            }
        }
        m
    }
}

/// Core game engine for rule generation and violation checking
pub struct GameEngine;

impl GameEngine {
    /// Generate shift rules based on player experience
    pub fn generate_shift_rules(
        experience: u32,
        all_rules: &[Rule],
        constants: &ConstantsData,
    ) -> ShiftRules {
        let difficulty_level = (experience / constants.scoring.experience_per_level)
            .min(constants.scoring.max_difficulty);

        // Separate rules by type
        let basic_rules: Vec<&Rule> = all_rules.iter().filter(|r| r.is_basic()).collect();
        let conditional_rules: Vec<&Rule> =
            all_rules.iter().filter(|r| r.is_conditional()).collect();

        let mut selected_rules = Vec::new();

        // Select 2-3 basic rules
        let num_basic = 2 + macroquad_toolkit::rng::gen_range(0, 2);
        let mut basic_rules_clone = basic_rules.clone();
        macroquad_toolkit::rng::shuffle(&mut basic_rules_clone);
        let shuffled_basic = basic_rules_clone
            .into_iter()
            .take(num_basic.min(basic_rules.len()));
        for rule in shuffled_basic {
            selected_rules.push(rule.clone());
        }

        // Add conditional rules based on difficulty
        if difficulty_level >= 1 {
            let num_conditional = 1 + macroquad_toolkit::rng::gen_range(0, 2);
            let mut conditional_rules_clone = conditional_rules.clone();
            macroquad_toolkit::rng::shuffle(&mut conditional_rules_clone);
            let shuffled_conditional = conditional_rules_clone
                .into_iter()
                .take(num_conditional.min(conditional_rules.len()));
            for rule in shuffled_conditional {
                selected_rules.push(rule.clone());
            }
        }

        // Separate visible and hidden rules
        let visible_rules: Vec<Rule> = selected_rules
            .iter()
            .filter(|r| r.visible)
            .cloned()
            .collect();
        let hidden_rules: Vec<Rule> = selected_rules
            .iter()
            .filter(|r| !r.visible)
            .cloned()
            .collect();

        ShiftRules {
            visible_rules,
            hidden_rules,
            difficulty_level,
        }
    }

    /// Check if an action violates any active rule
    pub fn check_rule_violation(
        rules: &[Rule],
        action: &str,
        passenger: Option<&Passenger>,
        need_state: Option<&PassengerNeedState>,
    ) -> RuleEvaluationResult {
        for rule in rules {
            if rule.forbids_action(action) {
                // Check for passenger-specific exceptions
                if let Some(p) = passenger {
                    if Self::passenger_has_exception(rule, p, need_state) {
                        return RuleEvaluationResult {
                            violation: false,
                            rule: Some(rule.clone()),
                            message: Some("Exception applies - action is safe".to_string()),
                            need_adjustment: rule.exception_need_adjustment.unwrap_or(0),
                            triggered_exception: None,
                        };
                    }
                }

                return RuleEvaluationResult {
                    violation: true,
                    rule: Some(rule.clone()),
                    message: Some(rule.get_violation_message().to_string()),
                    need_adjustment: rule.break_need_adjustment.unwrap_or(0),
                    triggered_exception: None,
                };
            }
        }

        RuleEvaluationResult::default()
    }

    /// Check if a route choice violates weather-triggered driving rules.
    pub fn check_weather_route_violation(
        rules: &[Rule],
        route: RouteType,
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
    ) -> RuleEvaluationResult {
        for rule in rules.iter().filter(|r| r.rule_type == RuleType::Weather) {
            if Self::weather_route_triggers(rule, route, weather, time_of_day) {
                return RuleEvaluationResult {
                    violation: true,
                    rule: Some(rule.clone()),
                    message: Some(rule.get_violation_message().to_string()),
                    need_adjustment: rule.break_need_adjustment.unwrap_or(10),
                    triggered_exception: None,
                };
            }
        }

        RuleEvaluationResult::default()
    }

    fn weather_route_triggers(
        rule: &Rule,
        route: RouteType,
        weather: &WeatherCondition,
        time_of_day: &TimeOfDay,
    ) -> bool {
        match rule.trigger.as_deref() {
            Some("heavy_fog") => {
                weather.weather_type == WeatherType::Fog
                    && weather.intensity == WeatherIntensity::Heavy
                    && route == RouteType::Shortcut
            }
            Some("snow") => {
                weather.weather_type == WeatherType::Snow && route == RouteType::Shortcut
            }
            Some("latenight_badweather") => {
                time_of_day.phase == TimePhase::Latenight
                    && weather.weather_type != WeatherType::Clear
                    && route == RouteType::Scenic
            }
            Some("low_visibility") => weather.visibility < 30 && route == RouteType::Shortcut,
            _ => false,
        }
    }

    /// Check if passenger has an exception to a rule
    fn passenger_has_exception(
        _rule: &Rule,
        passenger: &Passenger,
        need_state: Option<&PassengerNeedState>,
    ) -> bool {
        // Check if passenger's personal rule conflicts/overrides
        if let Some(modification) = &passenger.rule_modification {
            if modification.can_modify {
                return true;
            }
        }

        // Check need state for exception activation
        if let Some(state) = need_state {
            if state.stage >= NeedStage::Warning {
                // High stress passengers may trigger exceptions
                if let Some(ref _exception_id) = state.profile.exception_id {
                    // Exception is active when passenger is stressed
                    return true;
                }
            }
        }

        false
    }

    /// Calculate fare with all modifiers
    pub fn calculate_fare(
        base_fare: u32,
        route: RouteType,
        passenger: &Passenger,
        consecutive_streak: Option<&RouteStreak>,
        reputation: Option<&PassengerReputation>,
        constants: &ConstantsData,
        destination_fare_modifier: f32,
    ) -> u32 {
        // Route fare multiplier
        let route_mult = match route {
            RouteType::Shortcut => constants.route_fares.shortcut,
            RouteType::Normal => constants.route_fares.normal,
            RouteType::Scenic => constants.route_fares.scenic,
            RouteType::Police => constants.route_fares.police,
        };

        // Passenger preference multiplier
        let pref_mult = passenger
            .get_route_preference(route)
            .map(|p| p.fare_modifier)
            .unwrap_or(1.0);

        // Consecutive route penalty
        let streak_mult = if let Some(streak) = consecutive_streak {
            if streak.route_type == route && streak.count >= 2 {
                1.0 - (streak.count - 1) as f32 * constants.consecutive_route.penalty_per_repeat
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Reputation multiplier
        let rep_mult = reputation
            .map(|r| r.fare_multiplier(&constants.reputation))
            .unwrap_or(1.0);

        // Calculate with all multipliers
        let fare = base_fare as f32
            * route_mult
            * pref_mult
            * streak_mult
            * rep_mult
            * destination_fare_modifier;

        // Add variation (±$5)
        let variation = macroquad_toolkit::rng::gen_range(-5.0, 5.0);

        (fare + variation).max(5.0) as u32
    }
}

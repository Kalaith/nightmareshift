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
#[derive(Debug, Clone)]
pub struct RuleEvaluationResult {
    pub violation: bool,
    pub rule: Option<Rule>,
    pub message: Option<String>,
    pub need_adjustment: i32,
    pub triggered_exception: Option<GuidelineException>,
}

impl Default for RuleEvaluationResult {
    fn default() -> Self {
        Self {
            violation: false,
            rule: None,
            message: None,
            need_adjustment: 0,
            triggered_exception: None,
        }
    }
}

/// Core game engine for rule generation and violation checking
pub struct GameEngine;

impl GameEngine {
    /// Generate shift rules based on player experience
    pub fn generate_shift_rules(experience: u32, all_rules: &[Rule], constants: &ConstantsData) -> ShiftRules {
        let difficulty_level = (experience / constants.scoring.experience_per_level).min(constants.scoring.max_difficulty);


        // Separate rules by type
        let basic_rules: Vec<&Rule> = all_rules.iter()
            .filter(|r| r.is_basic())
            .collect();
        let conditional_rules: Vec<&Rule> = all_rules.iter()
            .filter(|r| r.is_conditional())
            .collect();

        let mut selected_rules = Vec::new();

        // Select 2-3 basic rules
        let num_basic = 2 + macroquad_toolkit::rng::gen_range(0, 2);
        let mut basic_rules_clone = basic_rules.clone();
        macroquad_toolkit::rng::shuffle(&mut basic_rules_clone);
        let shuffled_basic = basic_rules_clone.into_iter().take(num_basic.min(basic_rules.len()));
        for rule in shuffled_basic {
            selected_rules.push(rule.clone());
        }

        // Add conditional rules based on difficulty
        if difficulty_level >= 1 {
            let num_conditional = 1 + macroquad_toolkit::rng::gen_range(0, 2);
            let mut conditional_rules_clone = conditional_rules.clone();
            macroquad_toolkit::rng::shuffle(&mut conditional_rules_clone);
            let shuffled_conditional = conditional_rules_clone.into_iter().take(num_conditional.min(conditional_rules.len()));
            for rule in shuffled_conditional {
                selected_rules.push(rule.clone());
            }
        }

        // Separate visible and hidden rules
        let visible_rules: Vec<Rule> = selected_rules.iter()
            .filter(|r| r.visible)
            .cloned()
            .collect();
        let hidden_rules: Vec<Rule> = selected_rules.iter()
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
    ) -> u32 {


        // Route fare multiplier
        let route_mult = match route {
            RouteType::Shortcut => constants.route_fares.shortcut,
            RouteType::Normal => constants.route_fares.normal,
            RouteType::Scenic => constants.route_fares.scenic,
            RouteType::Police => constants.route_fares.police,
        };

        // Passenger preference multiplier
        let pref_mult = passenger.get_route_preference(route)
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
        let rep_mult = reputation.map(|r| r.fare_multiplier(&constants.reputation)).unwrap_or(1.0);

        // Calculate with all multipliers
        let fare = base_fare as f32 * route_mult * pref_mult * streak_mult * rep_mult;

        // Add variation (±$5)
        let variation = macroquad_toolkit::rng::gen_range(-5.0, 5.0);
        
        (fare + variation).max(5.0) as u32
    }
}

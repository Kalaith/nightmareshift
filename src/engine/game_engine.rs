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

        // Separate rules by type. `Weather` rules are deliberately absent:
        // they are injected by the weather sync in `Game`, not drawn here.
        let pool_of = |kind: RuleType| -> Vec<&Rule> {
            all_rules.iter().filter(|r| r.rule_type == kind).collect()
        };

        let mut selected_rules: Vec<Rule> = Vec::new();

        // Select 2-3 basic rules
        let num_basic = 2 + macroquad_toolkit::rng::gen_range(0, 2);
        Self::draw_compatible_rules(&pool_of(RuleType::Basic), num_basic, &mut selected_rules);

        // Add conditional rules based on difficulty
        if difficulty_level >= 1 {
            let num_conditional = 1 + macroquad_toolkit::rng::gen_range(0, 2);
            Self::draw_compatible_rules(
                &pool_of(RuleType::Conditional),
                num_conditional,
                &mut selected_rules,
            );
        }

        // `Conflicting` and `Hidden` rules were referenced by no selection
        // path at all, so twelve of the twenty-eight authored rules could
        // never appear — including all three `Hidden` ones, which are the
        // whole point of `reveal_hidden_rule`, the Glimpse skill's
        // `reveal_hidden_chance`, and the "Hidden Rule Violated!" ending.
        // Both are expert-tier content, so they join from difficulty 2.
        if difficulty_level >= 2 {
            Self::draw_compatible_rules(&pool_of(RuleType::Conflicting), 1, &mut selected_rules);
            Self::draw_compatible_rules(&pool_of(RuleType::Hidden), 1, &mut selected_rules);
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

    /// Draw up to `count` rules from `pool` that do not contradict anything
    /// already selected.
    ///
    /// `conflictsWith` is authored on three rules and was read by nothing, so
    /// a shift could hand the player both "Make eye contact with passengers
    /// to ensure they're alert" and "Do not look directly at passengers
    /// tonight" — a night that cannot be driven cleanly no matter what the
    /// player does. Fewer rules is the right failure here: a shift short one
    /// rule is playable, a self-contradictory one is not.
    fn draw_compatible_rules(pool: &[&Rule], count: u32, selected: &mut Vec<Rule>) {
        let mut shuffled: Vec<&Rule> = pool.to_vec();
        macroquad_toolkit::rng::shuffle(&mut shuffled);

        let mut taken = 0;
        for rule in shuffled {
            if taken >= count {
                break;
            }
            if selected
                .iter()
                .any(|chosen| Self::rules_conflict(chosen, rule))
            {
                continue;
            }
            selected.push(rule.clone());
            taken += 1;
        }
    }

    /// Whether two rules contradict each other. Links are authored one-way —
    /// "Safety First" names "No Eye Contact" but not the reverse — so both
    /// directions are checked.
    fn rules_conflict(a: &Rule, b: &Rule) -> bool {
        a.id == b.id || a.conflicts_with.contains(&b.id) || b.conflicts_with.contains(&a.id)
    }

    /// Check if an action violates any active rule
    pub fn check_rule_violation(
        rules: &[Rule],
        action: &str,
        need_state: Option<&PassengerNeedState>,
        guidelines: &[Guideline],
    ) -> RuleEvaluationResult {
        for rule in rules {
            if rule.forbids_action(action) {
                // A need state only exists while a passenger is aboard, so it
                // carries the "is anyone here" check the passenger argument
                // used to make.
                if Self::passenger_has_exception(rule, need_state, guidelines) {
                    return RuleEvaluationResult {
                        violation: false,
                        rule: Some(rule.clone()),
                        message: Some("Exception applies - action is safe".to_string()),
                        need_adjustment: rule.exception_need_adjustment.unwrap_or(0),
                        triggered_exception: None,
                    };
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
    /// Whether breaking `rule` is what the current passenger actually needs.
    ///
    /// The passenger is identified entirely by their `need_state`, whose
    /// profile carries the `exceptionId`; the passenger itself became an
    /// unused parameter once `rule_modification` stopped short-circuiting
    /// here, so it is gone rather than underscored.
    ///
    /// The rule argument used to be `_rule` — ignored entirely — so any
    /// forbidden cab action relieved any stressed passenger by that rule's
    /// `exceptionNeedAdjustment`. Mrs. Chen, who wants to be looked at, was
    /// soothed just as well by opening a window, and the `exceptionId` every
    /// profile authors decided nothing. The rule must now belong to the
    /// guideline that owns the passenger's own exception.
    fn passenger_has_exception(
        rule: &Rule,
        need_state: Option<&PassengerNeedState>,
        guidelines: &[Guideline],
    ) -> bool {
        // `rule_modification` used to short-circuit to true here, which
        // excused The Collector, Madame Zelda and the Midnight Mayor from
        // every rule for the whole ride. What they can do is rewrite the
        // night's rules once as they get in — `RuleModificationService` — not
        // ignore the ones that remain.

        let Some(state) = need_state else {
            return false;
        };
        let Some(exception_id) = state.profile.exception_id.as_deref() else {
            return false;
        };

        // The passenger's own exception, on the guideline this rule belongs
        // to, and only once they are visibly far enough gone to justify it.
        guidelines
            .iter()
            .filter(|guideline| rule.related_guideline_id == Some(guideline.id))
            .flat_map(|guideline| guideline.exceptions.iter())
            .filter(|exception| exception.id == exception_id)
            .any(|exception| state.stage >= Self::required_stage(exception))
    }

    /// Whether a hidden-rule violation actually lands.
    ///
    /// `PROBABILITIES.HIDDEN_RULE_VIOLATION` is authored at 0.3 and was read
    /// by nothing, so breaking a rule the player had no way to know about
    /// caught them every single time. Every sibling in that block --
    /// supernatural encounters, high-risk encounters, item drops, kin spawns,
    /// trade offers -- gates whether an event fires, so this one reads the
    /// same way: a hidden rule is a lurking thing that mostly does not notice,
    /// not a guaranteed sentence on a rule nobody could have read.
    ///
    /// A miss reveals nothing. Getting away with it has to teach the player
    /// nothing, or the rule would out itself the first time and the Glimpse
    /// skill -- which buys exactly that knowledge -- would have nothing left
    /// to sell.
    pub fn hidden_violation_lands(constants: &ConstantsData) -> bool {
        macroquad_toolkit::rng::chance(constants.probabilities.hidden_rule_violation)
    }

    /// The rule in force tonight that this passenger's exception belongs to.
    ///
    /// This is the rule they need broken. Whether it is among tonight's rules
    /// is the single most useful thing a player can know about a passenger in
    /// advance — it is what the almanac's "Relief" line names — and until now
    /// it changed only whether an exception could be claimed, never how hard
    /// the ride was to hold together.
    pub fn passengers_rule_in_force<'a>(
        rules: &'a [Rule],
        need_state: Option<&PassengerNeedState>,
        guidelines: &[Guideline],
    ) -> Option<&'a Rule> {
        let exception_id = need_state?.profile.exception_id.as_deref()?;
        let owning: Vec<u32> = guidelines
            .iter()
            .filter(|guideline| {
                guideline
                    .exceptions
                    .iter()
                    .any(|exception| exception.id == exception_id)
            })
            .map(|guideline| guideline.id)
            .collect();
        rules.iter().find(|rule| {
            rule.related_guideline_id
                .is_some_and(|id| owning.contains(&id))
        })
    }

    /// The need stage an exception becomes available at.
    ///
    /// Every exception in `guidelineData.json` authors `requiredStage`, all
    /// nineteen of them saying "warning" — and the gate was a hardcoded
    /// `NeedStage::Warning` that never read the field. The two agreed, so
    /// nothing was visibly wrong; authoring "critical" for an exception that
    /// should cost more to earn would simply have been ignored.
    ///
    /// An absent or unrecognised name falls back to Warning, which is what the
    /// code did before and what every exception asks for. A typo therefore
    /// keeps the exception reachable rather than quietly stranding a passenger
    /// with no way to be soothed.
    ///
    /// Public because the almanac dossier quotes it: the stage a passenger's
    /// relief unlocks at is exactly the sort of thing studying them should
    /// buy, and it must be the number the engine gates on, not a second copy
    /// of the same rule.
    pub fn required_stage(exception: &GuidelineException) -> NeedStage {
        exception
            .required_stage
            .as_deref()
            .and_then(NeedStage::parse)
            .unwrap_or(NeedStage::Warning)
    }

    /// Calculate fare with all modifiers
    /// How far either way the meter can land off the settled figure, and the
    /// floor no fare falls below.
    ///
    /// Named because two places need them: the payout rolls inside this band and
    /// the ride offer has to quote a range wide enough to contain wherever it
    /// lands. A quote that excludes the roll is a quote the payout can fall
    /// outside of.
    pub const FARE_VARIATION: f32 = 5.0;
    pub const MINIMUM_FARE: f32 = 5.0;

    /// The fare before the meter's own wobble -- everything that is decided
    /// rather than rolled.
    ///
    /// Split out because `calculate_fare` cannot be asked what a road pays: it
    /// rolls each time it is called, so four calls are four samples rather than
    /// four roads. Calling it per route from a draw function would have made the
    /// number on screen flicker every frame.
    #[allow(clippy::too_many_arguments)]
    pub fn fare_before_variation(
        base_fare: u32,
        route: RouteType,
        passenger: &Passenger,
        consecutive_streak: Option<&RouteStreak>,
        reputation: Option<&PassengerReputation>,
        constants: &ConstantsData,
        destination_fare_modifier: f32,
    ) -> f32 {
        let route_mult = match route {
            RouteType::Shortcut => constants.route_fares.shortcut,
            RouteType::Normal => constants.route_fares.normal,
            RouteType::Scenic => constants.route_fares.scenic,
            RouteType::Police => constants.route_fares.police,
        };

        let pref_mult = passenger
            .get_route_preference(route)
            .map(|p| p.fare_modifier)
            .unwrap_or(1.0);

        let streak_mult = if let Some(streak) = consecutive_streak {
            if streak.route_type == route && streak.count >= 2 {
                1.0 - (streak.count - 1) as f32 * constants.consecutive_route.penalty_per_repeat
            } else {
                1.0
            }
        } else {
            1.0
        };

        let rep_mult = reputation
            .map(|r| r.fare_multiplier(&constants.reputation))
            .unwrap_or(1.0);

        base_fare as f32
            * route_mult
            * pref_mult
            * streak_mult
            * rep_mult
            * destination_fare_modifier
    }

    pub fn calculate_fare(
        base_fare: u32,
        route: RouteType,
        passenger: &Passenger,
        consecutive_streak: Option<&RouteStreak>,
        reputation: Option<&PassengerReputation>,
        constants: &ConstantsData,
        destination_fare_modifier: f32,
    ) -> u32 {
        let fare = Self::fare_before_variation(
            base_fare,
            route,
            passenger,
            consecutive_streak,
            reputation,
            constants,
            destination_fare_modifier,
        );

        let variation =
            macroquad_toolkit::rng::gen_range(-Self::FARE_VARIATION, Self::FARE_VARIATION);

        (fare + variation).max(Self::MINIMUM_FARE) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{load_constants, load_rules};

    /// A generated shift must never contain two rules that contradict each
    /// other. Run many times because selection is random — a single draw
    /// passing proves nothing.
    #[test]
    fn generated_shifts_never_contradict_themselves() {
        let rules = load_rules();
        let constants = load_constants();
        for _ in 0..500 {
            for experience in [0, 10, 20, 30, 40] {
                let shift = GameEngine::generate_shift_rules(experience, &rules, &constants);
                let all: Vec<&Rule> = shift
                    .visible_rules
                    .iter()
                    .chain(shift.hidden_rules.iter())
                    .collect();
                for (i, a) in all.iter().enumerate() {
                    for b in all.iter().skip(i + 1) {
                        assert!(
                            !GameEngine::rules_conflict(a, b),
                            "shift paired {:?} with {:?}",
                            a.title,
                            b.title
                        );
                    }
                }
            }
        }
    }

    /// Conflicts are read in both directions, since they are authored one-way.
    #[test]
    fn conflicts_are_symmetric() {
        let rules = load_rules();
        let find = |id: u32| rules.iter().find(|r| r.id == id).expect("rule exists");
        // "Safety First" (11) names "No Eye Contact" (1); the reverse is not
        // authored, and the pair must still be recognised.
        let safety_first = find(11);
        let no_eye_contact = find(1);
        assert!(safety_first.conflicts_with.contains(&no_eye_contact.id));
        assert!(!no_eye_contact.conflicts_with.contains(&safety_first.id));
        assert!(GameEngine::rules_conflict(safety_first, no_eye_contact));
        assert!(GameEngine::rules_conflict(no_eye_contact, safety_first));
    }

    /// Every id named as a conflict must be a real rule, and no rule may
    /// conflict with itself.
    #[test]
    fn conflict_ids_are_real() {
        let rules = load_rules();
        let ids: Vec<u32> = rules.iter().map(|r| r.id).collect();
        for rule in &rules {
            assert!(
                !rule.conflicts_with.contains(&rule.id),
                "{:?} conflicts with itself",
                rule.title
            );
            for id in &rule.conflicts_with {
                assert!(ids.contains(id), "{:?} names unknown rule {id}", rule.title);
            }
        }
    }

    /// Every authored rule type that the generator owns must actually be
    /// reachable. `Conflicting` and `Hidden` were referenced by no selection
    /// path, so twelve of twenty-eight rules could never appear.
    #[test]
    fn every_generated_rule_type_is_reachable() {
        let rules = load_rules();
        let constants = load_constants();
        let mut seen: Vec<RuleType> = Vec::new();
        for _ in 0..500 {
            let shift = GameEngine::generate_shift_rules(40, &rules, &constants);
            for rule in shift.visible_rules.iter().chain(shift.hidden_rules.iter()) {
                if !seen.contains(&rule.rule_type) {
                    seen.push(rule.rule_type);
                }
            }
        }
        for kind in [
            RuleType::Basic,
            RuleType::Conditional,
            RuleType::Conflicting,
            RuleType::Hidden,
        ] {
            assert!(
                seen.contains(&kind),
                "{kind:?} rules never appear in a shift"
            );
        }
    }

    /// The hidden-rule mechanic needs hidden rules. `reveal_hidden_rule`, the
    /// Glimpse skill and the "Hidden Rule Violated!" ending all key off this
    /// list being non-empty at least sometimes.
    #[test]
    fn hidden_rules_actually_appear() {
        let rules = load_rules();
        let constants = load_constants();
        let with_hidden = (0..500)
            .filter(|_| {
                !GameEngine::generate_shift_rules(40, &rules, &constants)
                    .hidden_rules
                    .is_empty()
            })
            .count();
        assert!(
            with_hidden > 0,
            "no shift out of 500 at max difficulty carried a hidden rule"
        );
    }

    /// Every `SkillModifiers` field must move the number it names, by the
    /// amount the skill tree authors.
    ///
    /// This is a unit test rather than a bot measurement because aggregate
    /// play cannot answer it. Unlocking only the two fare skills and
    /// comparing average earnings per ride measured a 27.8% uplift against
    /// the 21% the multipliers work out to — not because the multiplier is
    /// wrong, but because a richer driver refuels more, survives longer, and
    /// carries a different mix of passengers, whose base fares run from $12
    /// to $100. The mix moves the average more than the multiplier does.
    #[test]
    fn the_fare_multiplier_is_exactly_what_the_tree_authors() {
        use crate::data::loader::{load_constants, load_passengers, load_skill_tree};

        let constants = load_constants();
        let skills = load_skill_tree();
        // The highest-paying fare on the roster, well clear of the $5 floor
        // `calculate_fare` clamps to — the first paying passenger's $12 fare
        // lands on it once the route and preference multipliers are applied,
        // and a clamped number cannot show a 1.21x ratio.
        let passenger = load_passengers()
            .into_iter()
            .max_by_key(|p| p.fare)
            .expect("a paying passenger");

        let fare_of = |unlocked: &[String]| {
            let mods = SkillModifiers::from_unlocked(&skills, unlocked);
            GameEngine::calculate_fare(
                passenger.fare,
                RouteType::Normal,
                &passenger,
                None,
                None,
                &constants,
                mods.fare_mult,
            )
        };

        // `calculate_fare` adds a +/-$5 variation, so a single pair of calls
        // cannot be compared. Averaging over many washes it out.
        let mean_fare = |unlocked: &[String]| {
            let total: u32 = (0..400).map(|_| fare_of(unlocked)).sum();
            total as f32 / 400.0
        };

        let plain = mean_fare(&[]);
        let both: Vec<String> = skills
            .iter()
            .filter(|s| s.effect.target == "fare_multiplier")
            .map(|s| s.id.clone())
            .collect();
        assert_eq!(both.len(), 2, "the tree no longer has two fare skills");

        let expected_mult: f32 = skills
            .iter()
            .filter(|s| s.effect.target == "fare_multiplier")
            .map(|s| s.effect.value as f32)
            .product();
        let boosted = mean_fare(&both);

        let ratio = boosted / plain;
        assert!(
            (ratio - expected_mult).abs() < 0.05,
            "mean fare went {plain:.1} -> {boosted:.1} ({ratio:.3}x)              against the authored {expected_mult:.3}x"
        );
    }

    /// The remaining modifiers must each be non-neutral once their skills are
    /// unlocked, or a node is bought and changes nothing.
    #[test]
    fn every_skill_modifier_moves_off_neutral() {
        use crate::data::loader::load_skill_tree;

        let skills = load_skill_tree();
        let all: Vec<String> = skills.iter().map(|s| s.id.clone()).collect();
        let none = SkillModifiers::from_unlocked(&skills, &[]);
        let full = SkillModifiers::from_unlocked(&skills, &all);

        assert!(full.fuel_cost_mult < none.fuel_cost_mult, "fuel cost");
        assert!(full.max_fuel_bonus > none.max_fuel_bonus, "max fuel");
        assert!(full.hazard_mult < none.hazard_mult, "hazard damage");
        assert!(
            full.reveal_hidden_chance > none.reveal_hidden_chance,
            "hidden rule reveal"
        );
        assert!(full.fare_mult > none.fare_mult, "fare");
        assert!(full.refuel_cost_mult < none.refuel_cost_mult, "refuel cost");
        assert!(
            full.bonus_protection > none.bonus_protection,
            "supernatural protection"
        );
    }

    /// Breaking the rule a passenger's own exception belongs to must relieve
    /// them; breaking an unrelated one must not. The rule argument used to be
    /// ignored, so every forbidden action soothed every stressed passenger
    /// equally and `exceptionId` decided nothing.
    #[test]
    fn only_the_passengers_own_rule_grants_an_exception() {
        use crate::data::loader::{load_guidelines, load_passengers};
        use crate::engine::PassengerStateMachine;

        let rules = load_rules();
        let guidelines = load_guidelines();
        let passengers = load_passengers();

        // Mrs. Chen's exception is `eye_contact_lonely`, owned by guideline
        // 1001, which rule 1 "No Eye Contact" belongs to.
        let chen = passengers.iter().find(|p| p.id == 1).expect("Mrs. Chen");
        let mut need = PassengerStateMachine::initialize(chen, 0.0).expect("Chen has a profile");
        // Push her past the warning threshold so the exception can be active.
        PassengerStateMachine::apply_stress_delta(&mut need, chen, 100, 0.0);

        let own_rule = rules.iter().find(|r| r.id == 1).expect("No Eye Contact");
        let other_rule = rules.iter().find(|r| r.id == 4).expect("Windows Sealed");

        assert!(
            GameEngine::passenger_has_exception(own_rule, Some(&need), &guidelines),
            "breaking her own rule does not relieve her"
        );
        assert!(
            !GameEngine::passenger_has_exception(other_rule, Some(&need), &guidelines),
            "an unrelated rule still relieves her"
        );
    }

    /// A hidden-rule violation lands about as often as it is authored to.
    ///
    /// Averaged over many rolls because a probability cannot be checked with
    /// one: a single call proves nothing and would pass whatever the constant
    /// said. The band is wide on purpose -- this is asserting that the number
    /// is consulted at all, not pinning the RNG.
    #[test]
    fn a_hidden_violation_lands_about_as_often_as_authored() {
        let constants = load_constants();
        let authored = constants.probabilities.hidden_rule_violation;

        const ROLLS: u32 = 4000;
        let landed = (0..ROLLS)
            .filter(|_| GameEngine::hidden_violation_lands(&constants))
            .count() as f32;
        let observed = landed / ROLLS as f32;

        assert!(
            (observed - authored).abs() < 0.05,
            "hidden violations landed {observed:.3} of the time, authored {authored:.3}"
        );
    }

    /// And the authored number has to leave both outcomes possible. At 1.0 a
    /// hidden rule is the guaranteed sentence it used to be; at 0.0 the whole
    /// hidden layer -- and the Glimpse skill that reveals it -- is inert.
    #[test]
    fn a_hidden_violation_is_neither_certain_nor_impossible() {
        let authored = load_constants().probabilities.hidden_rule_violation;
        assert!(
            authored > 0.0 && authored < 1.0,
            "HIDDEN_RULE_VIOLATION is {authored}, which makes the hidden layer              either harmless or unsurvivable"
        );
    }

    /// Keeping a rule has to be worth something, so the rule a passenger
    /// pulls against must author a reward for keeping it.
    ///
    /// Thirteen rules do and none of it paid until now. This checks the data
    /// rather than the payment, because the payment needs a `Game`: what it
    /// guards is a rule quietly losing its `followConsequences` and going back
    /// to being pure downside.
    #[test]
    fn the_rules_that_can_be_a_passengers_own_reward_keeping_them() {
        use crate::data::loader::{load_guidelines, load_passengers};
        use crate::engine::PassengerStateMachine;

        let rules = load_rules();
        let guidelines = load_guidelines();
        let mut checked = 0;
        for passenger in load_passengers() {
            let Some(need) = PassengerStateMachine::initialize(&passenger, 0.0) else {
                continue;
            };
            let Some(own) = GameEngine::passengers_rule_in_force(&rules, Some(&need), &guidelines)
            else {
                continue;
            };
            assert!(
                !own.follow_consequences.is_empty(),
                "{}'s own rule {} pays nothing for keeping it",
                passenger.name,
                own.id
            );
            checked += 1;
        }
        assert!(checked > 0, "no passenger's rule was found in the full set");
    }

    /// Every authored follow reward has to be a consequence type the appliers
    /// actually handle. `apply_rule_consequences` names Death, Survival,
    /// Reputation, Money, Fuel, Time, Item and StoryUnlock; a reward of any
    /// other kind would be accepted by serde and then dropped.
    #[test]
    fn every_follow_reward_is_a_kind_something_applies() {
        for rule in load_rules() {
            for reward in &rule.follow_consequences {
                assert!(
                    reward.probability > 0.0,
                    "rule {} authors a follow reward at zero probability",
                    rule.id
                );
                assert!(
                    !matches!(reward.consequence_type, ConsequenceType::Death),
                    "rule {} rewards keeping it with death",
                    rule.id
                );
            }
        }
    }

    /// The passenger's own rule is found when it is in force and not when it
    /// is absent, because that difference is now what decides how fast they
    /// wear down.
    #[test]
    fn a_passengers_own_rule_is_recognised_only_when_it_is_in_force() {
        use crate::data::loader::{load_guidelines, load_passengers};
        use crate::engine::PassengerStateMachine;

        let rules = load_rules();
        let guidelines = load_guidelines();
        // Mrs. Chen's exception is owned by guideline 1001, which rule 1
        // "No Eye Contact" belongs to.
        let chen = load_passengers()
            .into_iter()
            .find(|p| p.id == 1)
            .expect("Mrs. Chen");
        let need = PassengerStateMachine::initialize(&chen, 0.0).expect("Chen has a profile");

        let own: Vec<Rule> = rules.iter().filter(|r| r.id == 1).cloned().collect();
        let found = GameEngine::passengers_rule_in_force(&own, Some(&need), &guidelines);
        assert_eq!(
            found.map(|rule| rule.id),
            Some(1),
            "her own rule was in force and went unrecognised"
        );
        assert!(
            found.and_then(|rule| rule.follow_need_adjustment).is_some(),
            "rule 1 authors no followNeedAdjustment, so this wires nothing"
        );

        let others: Vec<Rule> = rules.iter().filter(|r| r.id != 1).cloned().collect();
        assert!(
            GameEngine::passengers_rule_in_force(&others, Some(&need), &guidelines).is_none(),
            "a shift without her rule still claimed to hold it"
        );
    }

    /// An exception waits for the stage it authors, not a stage the engine
    /// assumed. The gate was a hardcoded `NeedStage::Warning` while every
    /// exception carried a `requiredStage` the engine never read, so raising
    /// one to "critical" would have changed nothing.
    #[test]
    fn an_exception_waits_for_the_stage_it_authors() {
        use crate::data::loader::{load_guidelines, load_passengers};
        use crate::engine::PassengerStateMachine;

        let rules = load_rules();
        let mut guidelines = load_guidelines();
        let chen = load_passengers()
            .into_iter()
            .find(|p| p.id == 1)
            .expect("Mrs. Chen");
        let mut need = PassengerStateMachine::initialize(&chen, 0.0).expect("Chen has a profile");
        let own_rule = rules.iter().find(|r| r.id == 1).expect("No Eye Contact");

        // Make her exception cost more to earn than it does today.
        let exception = guidelines
            .iter_mut()
            .flat_map(|guideline| guideline.exceptions.iter_mut())
            .find(|exception| Some(exception.id.as_str()) == need.profile.exception_id.as_deref())
            .expect("Chen's exception");
        exception.required_stage = Some("critical".to_string());

        need.stage = NeedStage::Warning;
        assert!(
            !GameEngine::passenger_has_exception(own_rule, Some(&need), &guidelines),
            "a critical-only exception was granted at warning"
        );

        need.stage = NeedStage::Critical;
        assert!(
            GameEngine::passenger_has_exception(own_rule, Some(&need), &guidelines),
            "a critical-only exception was withheld at critical"
        );
    }

    /// Every authored `requiredStage` must name a stage. An unrecognised name
    /// falls back to Warning rather than stranding the passenger, so a typo
    /// would never show up in play — only here.
    #[test]
    fn every_authored_required_stage_names_a_stage() {
        use crate::data::loader::load_guidelines;

        let mut checked = 0;
        for guideline in load_guidelines() {
            for exception in &guideline.exceptions {
                if let Some(name) = exception.required_stage.as_deref() {
                    assert!(
                        NeedStage::parse(name).is_some(),
                        "exception {:?} on guideline {} requires unknown stage {name:?}",
                        exception.id,
                        guideline.id
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "no exception authors a requiredStage any more");
    }

    /// A calm passenger has no exception however well the rule matches.
    #[test]
    fn a_calm_passenger_grants_no_exception() {
        use crate::data::loader::{load_guidelines, load_passengers};
        use crate::engine::PassengerStateMachine;

        let rules = load_rules();
        let guidelines = load_guidelines();
        let chen = load_passengers()
            .into_iter()
            .find(|p| p.id == 1)
            .expect("Mrs. Chen");
        let need = PassengerStateMachine::initialize(&chen, 0.0).expect("Chen has a profile");
        let own_rule = rules.iter().find(|r| r.id == 1).expect("No Eye Contact");

        assert!(!GameEngine::passenger_has_exception(
            own_rule,
            Some(&need),
            &guidelines
        ));
    }

    /// A shift must still produce rules. Filtering conflicts too eagerly —
    /// or a data change that made everything conflict — would leave the
    /// player with nothing to obey.
    #[test]
    fn shifts_still_produce_rules() {
        let rules = load_rules();
        let constants = load_constants();
        for _ in 0..200 {
            let shift = GameEngine::generate_shift_rules(30, &rules, &constants);
            assert!(
                !shift.visible_rules.is_empty() || !shift.hidden_rules.is_empty(),
                "generated a shift with no rules at all"
            );
        }
    }
}

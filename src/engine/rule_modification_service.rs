//! Passengers who change the rules of the night.
//!
//! Three of the roster author a `ruleModification`: The Collector can remove a
//! rule "in exchange for a terrible price", Madame Zelda reveals a hidden one,
//! and the Midnight Mayor imposes his own Decree. Only `canModify` was ever
//! read, and it was read in `passenger_has_exception`, where returning true
//! meant these three were excused from *every* rule for the whole ride — the
//! opposite of a bargain with a terrible price. The `type` deciding which of
//! the three things happened was ignored, and the Mayor's authored rule was
//! dropped at parse for want of a field to land in.

use crate::data::*;
use crate::state::*;

/// What a passenger's arrival did to tonight's rules, for narration.
pub struct RuleChange {
    pub message: String,
}

pub struct RuleModificationService;

impl RuleModificationService {
    /// Apply a passenger's rule modification, once, as they get in.
    pub fn apply(state: &mut GameState, passenger: &Passenger) -> Option<RuleChange> {
        let modification = passenger.rule_modification.as_ref()?;
        if !modification.can_modify {
            return None;
        }

        match modification.modification_type.as_str() {
            "reveal_hidden" => Self::reveal_hidden(state, passenger),
            "remove_rule" => Self::remove_rule(state, passenger),
            "add_temporary" => Self::add_temporary(state, passenger, modification),
            _ => None,
        }
    }

    /// Madame Zelda sees what tonight is hiding.
    fn reveal_hidden(state: &mut GameState, passenger: &Passenger) -> Option<RuleChange> {
        let rule_id = state.hidden_rules.first().map(|rule| rule.id)?;
        let revealed = state.reveal_hidden_rule(rule_id)?;
        Some(RuleChange {
            message: format!(
                "{} names a rule you were never told: {}.",
                passenger.name, revealed.title
            ),
        })
    }

    /// The Collector takes a rule off the board, and takes something for it.
    ///
    /// The price is danger, not money: the rule is gone but the roads are
    /// worse for the rest of the ride.
    fn remove_rule(state: &mut GameState, passenger: &Passenger) -> Option<RuleChange> {
        if state.current_rules.is_empty() {
            return None;
        }
        let index = macroquad_toolkit::rng::gen_range(0, state.current_rules.len());
        let removed = state.current_rules.remove(index);
        state.curse_danger_bonus += 2;
        Some(RuleChange {
            message: format!(
                "{} folds \"{}\" away like a card. The night notices the gap.",
                passenger.name, removed.title
            ),
        })
    }

    /// The Midnight Mayor adds one of his own, for a few rides.
    fn add_temporary(
        state: &mut GameState,
        passenger: &Passenger,
        modification: &RuleModification,
    ) -> Option<RuleChange> {
        let authored = modification.new_rule.as_ref()?;
        if state
            .current_rules
            .iter()
            .any(|rule| rule.id == authored.id)
        {
            return None;
        }

        let mut rule = Rule {
            id: authored.id,
            title: authored.title.clone(),
            description: authored.description.clone(),
            difficulty: authored.difficulty,
            visible: true,
            ..Rule::default()
        };
        rule.rule_type = RuleType::Conditional;
        state.current_rules.push(rule);
        state.temporary_rules.push(TemporaryRuleState {
            rule_id: authored.id,
            rides_remaining: authored.duration.max(1),
        });

        Some(RuleChange {
            message: format!(
                "{} imposes \"{}\" for the next {} rides.",
                passenger.name,
                authored.title,
                authored.duration.max(1)
            ),
        })
    }

    /// Count down any temporary rules and drop the ones that have expired.
    /// Called once per completed ride.
    pub fn expire_temporary_rules(state: &mut GameState) -> Vec<String> {
        let mut lifted = Vec::new();
        for temporary in state.temporary_rules.iter_mut() {
            temporary.rides_remaining = temporary.rides_remaining.saturating_sub(1);
        }
        state.temporary_rules.retain(|temporary| {
            if temporary.rides_remaining > 0 {
                return true;
            }
            if let Some(index) = state
                .current_rules
                .iter()
                .position(|rule| rule.id == temporary.rule_id)
            {
                lifted.push(state.current_rules.remove(index).title);
            }
            false
        });
        lifted
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_passengers;

    use super::*;
    use crate::data::loader::{load_constants, load_rules};

    fn state_with_rules() -> GameState {
        let constants = load_constants();
        let mut state = GameState::new(0.0, &constants.game_constants);
        let rules = load_rules();
        state.current_rules = rules.iter().take(3).cloned().collect();
        state.hidden_rules = rules.iter().skip(3).take(1).cloned().collect();
        state
    }

    fn passenger(id: u32) -> Passenger {
        load_passengers()
            .into_iter()
            .find(|p| p.id == id)
            .expect("passenger exists")
    }

    /// Madame Zelda moves a hidden rule into the open.
    #[test]
    fn reveal_hidden_moves_a_rule_into_the_open() {
        let mut state = state_with_rules();
        let hidden_before = state.hidden_rules.len();
        let visible_before = state.current_rules.len();

        let change = RuleModificationService::apply(&mut state, &passenger(11));

        assert!(change.is_some(), "Madame Zelda revealed nothing");
        assert_eq!(state.hidden_rules.len(), hidden_before - 1);
        assert_eq!(state.current_rules.len(), visible_before + 1);
    }

    /// The Collector takes a rule away, and takes something for it.
    #[test]
    fn remove_rule_costs_something() {
        let mut state = state_with_rules();
        let before = state.current_rules.len();
        let danger_before = state.curse_danger_bonus;

        let change = RuleModificationService::apply(&mut state, &passenger(5));

        assert!(change.is_some(), "The Collector removed nothing");
        assert_eq!(state.current_rules.len(), before - 1);
        assert!(
            state.curse_danger_bonus > danger_before,
            "a rule was removed for free"
        );
    }

    /// The Midnight Mayor's Decree arrives, stays for its authored run of
    /// rides, and then lifts.
    #[test]
    fn add_temporary_imposes_then_lifts() {
        let mut state = state_with_rules();
        let mayor = passenger(15);
        let authored = mayor
            .rule_modification
            .as_ref()
            .and_then(|m| m.new_rule.as_ref())
            .expect("the Mayor authors a rule")
            .clone();

        assert!(RuleModificationService::apply(&mut state, &mayor).is_some());
        assert!(state.current_rules.iter().any(|r| r.id == authored.id));

        for ride in 1..authored.duration {
            let lifted = RuleModificationService::expire_temporary_rules(&mut state);
            assert!(
                lifted.is_empty(),
                "the Decree lifted after {ride} of {} rides",
                authored.duration
            );
            assert!(state.current_rules.iter().any(|r| r.id == authored.id));
        }

        let lifted = RuleModificationService::expire_temporary_rules(&mut state);
        assert_eq!(lifted, vec![authored.title.clone()]);
        assert!(!state.current_rules.iter().any(|r| r.id == authored.id));
        assert!(state.temporary_rules.is_empty());
    }

    /// Applying twice must not stack the same Decree.
    #[test]
    fn the_decree_is_not_imposed_twice() {
        let mut state = state_with_rules();
        let mayor = passenger(15);
        RuleModificationService::apply(&mut state, &mayor);
        let after_first = state.current_rules.len();
        RuleModificationService::apply(&mut state, &mayor);
        assert_eq!(state.current_rules.len(), after_first);
        assert_eq!(state.temporary_rules.len(), 1);
    }

    /// Every authored modification type must be one `apply` handles, or the
    /// passenger's whole gimmick silently does nothing.
    #[test]
    fn every_authored_modification_type_is_handled() {
        const HANDLED: [&str; 3] = ["reveal_hidden", "remove_rule", "add_temporary"];
        for passenger in load_passengers() {
            let Some(modification) = &passenger.rule_modification else {
                continue;
            };
            assert!(
                HANDLED.contains(&modification.modification_type.as_str()),
                "{} authors unhandled modification {:?}",
                passenger.name,
                modification.modification_type
            );
        }
    }

    /// An `add_temporary` passenger must author the rule they impose, or
    /// their arrival changes nothing.
    #[test]
    fn add_temporary_passengers_author_a_rule() {
        for passenger in load_passengers() {
            let Some(modification) = &passenger.rule_modification else {
                continue;
            };
            if modification.modification_type != "add_temporary" {
                continue;
            }
            let rule = modification
                .new_rule
                .as_ref()
                .unwrap_or_else(|| panic!("{} imposes no rule", passenger.name));
            assert!(
                !rule.title.trim().is_empty(),
                "{} imposes an untitled rule",
                passenger.name
            );
            assert!(
                rule.duration > 0,
                "{} imposes a rule for no rides",
                passenger.name
            );
        }
    }

    /// A temporary rule must not collide with an authored one, or lifting it
    /// would remove the wrong rule from the shift.
    #[test]
    fn temporary_rule_ids_do_not_collide() {
        let authored: Vec<u32> = crate::data::loader::load_rules()
            .iter()
            .map(|rule| rule.id)
            .collect();
        for passenger in load_passengers() {
            let Some(rule) = passenger
                .rule_modification
                .as_ref()
                .and_then(|m| m.new_rule.as_ref())
            else {
                continue;
            };
            assert!(
                !authored.contains(&rule.id),
                "{} imposes rule id {} which already exists",
                passenger.name,
                rule.id
            );
        }
    }
}

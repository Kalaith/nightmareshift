//! Skill tree and almanac data matching JSON files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect of a skill.
///
/// Dispatch is by `target`: `SkillModifiers::from_unlocked` consumes the
/// seven stat targets, and ability unlocks are consulted by trait id via
/// the unlocked-skill list. `effect_type` routes only one way — the
/// `"ability_unlock"` value marks a skill as a per-passenger ability; the
/// `stat_boost`/`mechanic_unlock`/`passive_bonus` labels are authorial
/// grouping with no behavior, and `value` on an ability unlock is a
/// constant 1. A test in this file holds every effect to one of the two
/// dispatch paths, because `from_unlocked` silently ignores an unknown
/// target — a typo would sell a skill that does nothing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub target: String,
    pub value: f64,
}

/// A skill that can be unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    // `icon` is authored on every skill and read by nothing: the skill tree
    // draws a category mark and the name's initials.
    pub category: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub effect: SkillEffect,
}

impl Skill {
    /// Check if prerequisites are met
    pub fn can_unlock(&self, unlocked_skills: &[String]) -> bool {
        self.prerequisites
            .iter()
            .all(|prereq| unlocked_skills.contains(prereq))
    }
}

/// Almanac level data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacLevel {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub rewards: Vec<String>,
}

impl AlmanacLevel {
    /// How this level reads on the almanac card, or `None` when it reveals
    /// nothing nameable.
    ///
    /// The sentence lives here rather than in the draw call so the wording is
    /// testable without a window -- the same reason `PlayerStats::reveal_story`
    /// holds what a story unlock means.
    /// `already_known` is anything the player has by another route, left out
    /// so a card that has just shown someone's backstory does not go on
    /// promising it one level later. A story can be earned in play instead of
    /// bought with lore.
    pub fn reveals_line(&self, already_known: &[&str]) -> Option<String> {
        let remaining: Vec<&str> = self
            .rewards
            .iter()
            .map(String::as_str)
            .filter(|reward| !already_known.contains(reward))
            .collect();
        if remaining.is_empty() {
            return None;
        }
        Some(format!("{} reveals {}", self.name, remaining.join(", ")))
    }
}

/// Lore costs for unlocking almanac levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoreCosts {
    #[serde(rename = "UNLOCK_LEVEL_1")]
    pub level_1: u32,
    #[serde(rename = "UNLOCK_LEVEL_2")]
    pub level_2: u32,
    #[serde(rename = "UNLOCK_LEVEL_3")]
    pub level_3: u32,
}

/// Almanac data structure matching almanacData.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlmanacData {
    #[serde(rename = "ALMANAC_LEVELS")]
    pub levels: HashMap<String, AlmanacLevel>,
    #[serde(rename = "LORE_COSTS")]
    pub lore_costs: LoreCosts,
}

impl AlmanacData {
    /// Get level by number
    pub fn get_level(&self, level: u32) -> Option<&AlmanacLevel> {
        self.levels.get(&level.to_string())
    }

    /// Get cost to unlock a specific level
    pub fn get_upgrade_cost(&self, target_level: u32) -> u32 {
        match target_level {
            1 => self.lore_costs.level_1,
            2 => self.lore_costs.level_2,
            3 => self.lore_costs.level_3,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::{load_almanac, load_skill_tree};
    use std::collections::HashSet;

    /// Every level a player can buy has to name what it reveals.
    ///
    /// The almanac card now prints "Mastered reveals Hidden Rules, True
    /// Nature, Backstory" beside the price, from this list. An empty list
    /// there puts the screen back to showing a price and a tier name and
    /// nothing about what the lore buys, which is what it did before.
    #[test]
    fn every_purchasable_level_names_what_it_reveals() {
        let almanac = load_almanac();
        for level in 1..=3 {
            let entry = almanac
                .get_level(level)
                .unwrap_or_else(|| panic!("no almanac level {level} authored"));
            assert!(!entry.name.trim().is_empty(), "level {level} has no name");
            assert!(
                !entry.rewards.is_empty(),
                "level {level} ({}) reveals nothing the player can be told about",
                entry.name
            );
            for reward in &entry.rewards {
                assert!(
                    !reward.trim().is_empty(),
                    "level {level} authors a blank reward"
                );
            }
        }
    }

    /// The sentence the card prints has to name the rewards, not just the
    /// tier. Printing only the tier is what the screen effectively did before,
    /// and it reads as informative while saying nothing.
    #[test]
    fn the_reveals_line_names_the_rewards() {
        let almanac = load_almanac();
        for level in 1..=3 {
            let entry = almanac.get_level(level).expect("a level");
            let line = entry
                .reveals_line(&[])
                .unwrap_or_else(|| panic!("level {level} prints nothing"));
            for reward in &entry.rewards {
                assert!(
                    line.contains(reward.as_str()),
                    "level {level} does not mention {reward:?} in {line:?}"
                );
            }
        }
    }

    /// Something already in hand is not promised again. A story earned in play
    /// shows on the card immediately, and the next level's line used to go on
    /// offering the same backstory one tier later.
    #[test]
    fn a_reward_already_in_hand_is_not_promised_again() {
        let mastered = load_almanac().get_level(3).expect("level 3").clone();
        assert!(
            mastered.rewards.iter().any(|r| r == "Backstory"),
            "level 3 no longer offers a backstory; pick another reward"
        );

        let full = mastered.reveals_line(&[]).expect("a line");
        assert!(full.contains("Backstory"));

        let trimmed = mastered.reveals_line(&["Backstory"]).expect("a line");
        assert!(
            !trimmed.contains("Backstory"),
            "a known backstory was promised again in {trimmed:?}"
        );
        assert!(
            trimmed.contains("True Nature"),
            "excluding one reward dropped the others: {trimmed:?}"
        );
    }

    /// Every skill must do something when bought. `from_unlocked` ignores
    /// unknown targets silently, so a typo'd target would sell a skill that
    /// does nothing; this buys each skill alone and requires a modifier to
    /// move. Ability unlocks dispatch by id instead, and their `value` is
    /// read by nothing — held to the constant 1 so it cannot masquerade as
    /// a magnitude.
    #[test]
    fn every_skill_moves_a_modifier_or_unlocks_an_ability() {
        use crate::engine::SkillModifiers;
        for skill in crate::data::loader::load_skill_tree() {
            if skill.effect.effect_type == "ability_unlock" {
                assert_eq!(
                    skill.effect.value, 1.0,
                    "{}: an ability unlock's value is read by nothing - author 1",
                    skill.id
                );
                continue;
            }
            let bought = SkillModifiers::from_unlocked(
                std::slice::from_ref(&skill),
                std::slice::from_ref(&skill.id),
            );
            assert!(
                bought != SkillModifiers::default(),
                "{}: target {:?} moved no modifier - the skill does nothing",
                skill.id,
                skill.effect.target
            );
        }
    }

    /// Excluding everything a level offers leaves nothing to say, rather than
    /// a tier name with a dangling "reveals".
    #[test]
    fn excluding_every_reward_prints_nothing() {
        let mastered = load_almanac().get_level(3).expect("level 3").clone();
        let all: Vec<&str> = mastered.rewards.iter().map(String::as_str).collect();
        assert!(mastered.reveals_line(&all).is_none());
    }

    /// A level that reveals nothing prints nothing, rather than a bare tier
    /// name with a dangling "reveals".
    #[test]
    fn a_level_that_reveals_nothing_prints_nothing() {
        let silent = super::AlmanacLevel {
            name: "Unknown".to_string(),
            description: String::new(),
            rewards: Vec::new(),
        };
        assert!(silent.reveals_line(&[]).is_none());
    }

    /// Level 0 is the not-yet-met state and is named, since the card falls
    /// back to a localized string only when the level is missing entirely.
    #[test]
    fn the_unmet_level_is_named() {
        let unmet = load_almanac().get_level(0).expect("level 0").name.clone();
        assert!(!unmet.trim().is_empty());
    }

    /// Each level has to cost something, or the tiers above it are free and
    /// the lore currency has no sink at the almanac.
    #[test]
    fn every_upgrade_costs_lore() {
        let almanac = load_almanac();
        for level in 1..=3 {
            assert!(
                almanac.get_upgrade_cost(level) > 0,
                "reaching level {level} is free"
            );
        }
    }

    /// A prerequisite naming a skill that does not exist locks its node out
    /// of the tree permanently, and `can_unlock` would never return true.
    #[test]
    fn no_prerequisite_dangles() {
        let skills = load_skill_tree();
        let ids: HashSet<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        for skill in &skills {
            for prerequisite in &skill.prerequisites {
                assert!(
                    ids.contains(prerequisite.as_str()),
                    "{} requires unknown skill {prerequisite:?}",
                    skill.id
                );
            }
        }
    }

    /// Every node must be reachable from an empty unlock list by satisfying
    /// prerequisites in some order. A cycle, or a node gated behind one,
    /// would be bank balance the player can never spend.
    #[test]
    fn every_skill_is_reachable_from_nothing() {
        let skills = load_skill_tree();
        let mut unlocked: HashSet<String> = HashSet::new();
        loop {
            let newly: Vec<String> = skills
                .iter()
                .filter(|s| !unlocked.contains(&s.id))
                .filter(|s| s.can_unlock(&unlocked.iter().cloned().collect::<Vec<_>>()))
                .map(|s| s.id.clone())
                .collect();
            if newly.is_empty() {
                break;
            }
            unlocked.extend(newly);
        }
        let stranded: Vec<&str> = skills
            .iter()
            .map(|s| s.id.as_str())
            .filter(|id| !unlocked.contains(*id))
            .collect();
        assert!(stranded.is_empty(), "unreachable skills: {stranded:?}");
    }

    /// At least one node must have no prerequisites, or the tree has no entry
    /// point regardless of how much bank the player saves.
    #[test]
    fn the_tree_has_an_entry_point() {
        assert!(load_skill_tree().iter().any(|s| s.prerequisites.is_empty()));
    }
}

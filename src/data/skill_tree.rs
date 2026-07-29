//! Skill tree and almanac data matching JSON files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect of a skill
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
    use crate::data::loader::load_skill_tree;
    use std::collections::HashSet;

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

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
    pub icon: String,
    pub category: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub effect: SkillEffect,
}

impl Skill {
    /// Check if prerequisites are met
    pub fn can_unlock(&self, unlocked_skills: &[String]) -> bool {
        self.prerequisites.iter().all(|prereq| unlocked_skills.contains(prereq))
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

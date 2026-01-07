//! Player statistics tracking across sessions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Leaderboard entry for a completed shift
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub score: u32,
    pub date: String,
    pub survived: bool,
    pub passengers_transported: u32,
    pub difficulty_level: u32,
    pub rules_violated: u32,
}

/// Almanac progress for a passenger
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlmanacEntry {
    pub passenger_id: u32,
    pub encountered: bool,
    pub knowledge_level: u32,
}

/// Achievement tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub unlocked: bool,
    pub unlock_date: Option<String>,
}

/// Persistent player statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Total shifts completed
    pub total_shifts_completed: u32,
    /// Total rides completed across all shifts
    pub total_rides_completed: u32,
    /// Total earnings across all shifts
    pub total_earnings: u32,
    /// Highest single shift earnings
    pub highest_shift_earnings: u32,
    /// Total play time in minutes
    pub total_play_time: u32,
    /// Number of survival bonuses earned
    pub survival_bonuses: u32,
    /// Total rules violated
    pub total_rules_violated: u32,
    /// Unlocked backstories by passenger ID
    pub unlocked_backstories: HashMap<u32, bool>,
    /// Passenger encounter counts
    pub passenger_encounters: HashMap<u32, u32>,
    /// Unlocked skills by ID
    pub unlocked_skills: Vec<String>,
    /// Bank balance for purchasing skills
    #[serde(default)]
    pub bank_balance: u32,
    /// Almanac progress by passenger ID
    #[serde(default)]
    pub almanac_progress: HashMap<u32, AlmanacEntry>,
    /// Lore fragments for upgrading almanac knowledge
    #[serde(default)]
    pub lore_fragments: u32,
    /// Leaderboard entries (top 10 scores)
    #[serde(default)]
    pub leaderboard: Vec<LeaderboardEntry>,
    /// Route usage counts
    pub route_usage: HashMap<String, u32>,
    /// Unlocked achievements
    #[serde(default)]
    pub achievements: Vec<Achievement>,
    /// Current session start time
    #[serde(skip)]
    pub session_start: Option<f64>,
}

impl PlayerStats {
    /// Create new empty player stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate player experience level
    pub fn experience_level(&self) -> u32 {
        (self.total_rides_completed + self.total_shifts_completed * 10) / 10
    }

    /// Calculate difficulty based on experience
    pub fn suggested_difficulty(&self) -> u32 {
        (self.total_shifts_completed / 10).min(4)
    }

    /// Record a completed shift
    pub fn record_shift_completion(&mut self, earnings: u32, rides: u32, survived: bool, play_time: u32) {
        self.total_shifts_completed += 1;
        self.total_rides_completed += rides;
        self.total_earnings += earnings;
        self.total_play_time += play_time;
        
        if earnings > self.highest_shift_earnings {
            self.highest_shift_earnings = earnings;
        }
        
        if survived {
            self.survival_bonuses += 1;
        }
    }

    /// Record a passenger encounter
    pub fn record_passenger_encounter(&mut self, passenger_id: u32) {
        *self.passenger_encounters.entry(passenger_id).or_insert(0) += 1;
    }

    /// Unlock a backstory
    pub fn unlock_backstory(&mut self, passenger_id: u32) {
        self.unlocked_backstories.insert(passenger_id, true);
    }

    /// Check if backstory is unlocked
    pub fn is_backstory_unlocked(&self, passenger_id: u32) -> bool {
        self.unlocked_backstories.get(&passenger_id).copied().unwrap_or(false)
    }

    /// Get encounter count for a passenger
    pub fn get_encounter_count(&self, passenger_id: u32) -> u32 {
        self.passenger_encounters.get(&passenger_id).copied().unwrap_or(0)
    }

    /// Check if this is first encounter with a passenger
    pub fn is_first_encounter(&self, passenger_id: u32) -> bool {
        self.get_encounter_count(passenger_id) == 0
    }

    /// Check if skill is unlocked
    pub fn is_skill_unlocked(&self, skill_id: &str) -> bool {
        self.unlocked_skills.contains(&skill_id.to_string())
    }

    /// Record route usage
    pub fn record_route_usage(&mut self, route: &str) {
        *self.route_usage.entry(route.to_string()).or_insert(0) += 1;
    }

    /// Add a leaderboard entry and sort (keep top 10)
    pub fn add_leaderboard_entry(&mut self, entry: LeaderboardEntry) {
        self.leaderboard.push(entry);
        self.leaderboard.sort_by(|a, b| b.score.cmp(&a.score));
        self.leaderboard.truncate(10);
    }

    /// Get almanac entry for a passenger (or default)
    pub fn get_almanac_entry(&self, passenger_id: u32) -> AlmanacEntry {
        self.almanac_progress.get(&passenger_id)
            .cloned()
            .unwrap_or(AlmanacEntry {
                passenger_id,
                encountered: false,
                knowledge_level: 0,
            })
    }

    /// Mark passenger as encountered
    pub fn mark_passenger_encountered(&mut self, passenger_id: u32) {
        let entry = self.almanac_progress.entry(passenger_id).or_insert(AlmanacEntry {
            passenger_id,
            encountered: false,
            knowledge_level: 0,
        });
        entry.encountered = true;
    }

    /// Upgrade almanac knowledge level
    pub fn upgrade_almanac_knowledge(&mut self, passenger_id: u32, cost: u32) -> bool {
        if self.lore_fragments >= cost {
            let entry = self.almanac_progress.entry(passenger_id).or_insert(AlmanacEntry {
                passenger_id,
                encountered: true,
                knowledge_level: 0,
            });

            if entry.knowledge_level < 3 {
                self.lore_fragments -= cost;
                entry.knowledge_level += 1;
                return true;
            }
        }
        false
    }

    /// Purchase a skill with bank balance
    pub fn purchase_skill(&mut self, skill_id: &str, cost: u32) -> bool {
        if self.bank_balance >= cost && !self.unlocked_skills.contains(&skill_id.to_string()) {
            self.bank_balance -= cost;
            self.unlocked_skills.push(skill_id.to_string());
            true
        } else {
            false
        }
    }

    /// Initialize achievements if empty
    pub fn init_achievements(&mut self) {
        if self.achievements.is_empty() {
            self.achievements = vec![
                Achievement {
                    id: "first_shift".to_string(),
                    name: "First Night".to_string(),
                    description: "Complete your first shift".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
                Achievement {
                    id: "survivor".to_string(),
                    name: "Survivor".to_string(),
                    description: "Survive 10 shifts".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
                Achievement {
                    id: "perfect_shift".to_string(),
                    name: "Perfect Shift".to_string(),
                    description: "Complete a shift without violating any rules".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
                Achievement {
                    id: "big_earner".to_string(),
                    name: "Big Earner".to_string(),
                    description: "Earn $500 in a single shift".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
                Achievement {
                    id: "almanac_scholar".to_string(),
                    name: "Almanac Scholar".to_string(),
                    description: "Master knowledge of 5 passengers".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
                Achievement {
                    id: "skill_collector".to_string(),
                    name: "Skill Collector".to_string(),
                    description: "Unlock 3 skills".to_string(),
                    unlocked: false,
                    unlock_date: None,
                },
            ];
        }
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str, date: String) -> bool {
        if let Some(achievement) = self.achievements.iter_mut().find(|a| a.id == achievement_id) {
            if !achievement.unlocked {
                achievement.unlocked = true;
                achievement.unlock_date = Some(date);
                return true;
            }
        }
        false
    }

    /// Check if achievement is unlocked
    pub fn is_achievement_unlocked(&self, achievement_id: &str) -> bool {
        self.achievements.iter().any(|a| a.id == achievement_id && a.unlocked)
    }

    /// Check and unlock achievements based on current stats
    pub fn check_achievements(&mut self, shift_earnings: u32, shift_survived: bool, shift_violations: u32) {
        use chrono::Local;
        let now = Local::now().format("%Y-%m-%d").to_string();

        // First shift
        if self.total_shifts_completed >= 1 {
            self.unlock_achievement("first_shift", now.clone());
        }

        // Survivor (10 shifts)
        if self.survival_bonuses >= 10 {
            self.unlock_achievement("survivor", now.clone());
        }

        // Perfect shift (no violations)
        if shift_survived && shift_violations == 0 {
            self.unlock_achievement("perfect_shift", now.clone());
        }

        // Big earner ($500 in single shift)
        if shift_earnings >= 500 {
            self.unlock_achievement("big_earner", now.clone());
        }

        // Almanac scholar (5 passengers mastered)
        let mastered = self.almanac_progress.values().filter(|e| e.knowledge_level >= 3).count();
        if mastered >= 5 {
            self.unlock_achievement("almanac_scholar", now.clone());
        }

        // Skill collector (3 skills)
        if self.unlocked_skills.len() >= 3 {
            self.unlock_achievement("skill_collector", now.clone());
        }
    }
}


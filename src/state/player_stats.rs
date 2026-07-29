//! Player statistics tracking across sessions.

use crate::data::RouteType;
use macroquad_toolkit::achievements::{Achievement, Achievements};
use serde::{Deserialize, Deserializer, Serialize};
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

/// Accepts saves from before the `Achievements` registry adoption, where
/// `achievements` was a bare `Vec<Achievement>` instead of the registry's
/// `{ achievements: [...] }` shape. Old and new shapes both deserialize
/// cleanly since `Achievement`'s fields are unchanged.
fn deserialize_achievements<'de, D>(deserializer: D) -> Result<Achievements, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Compat {
        Legacy(Vec<Achievement>),
        Current(Achievements),
    }

    Ok(match Compat::deserialize(deserializer)? {
        Compat::Legacy(list) => Achievements::from_definitions(list),
        Compat::Current(achievements) => achievements,
    })
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
    #[serde(default)]
    pub route_usage: HashMap<String, u32>,
    /// Unlocked achievements
    #[serde(default, deserialize_with = "deserialize_achievements")]
    pub achievements: Achievements,
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
    pub fn record_shift_completion(
        &mut self,
        earnings: u32,
        rides: u32,
        survived: bool,
        play_time: u32,
    ) {
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
        self.unlocked_backstories
            .get(&passenger_id)
            .copied()
            .unwrap_or(false)
    }

    /// Get encounter count for a passenger
    pub fn get_encounter_count(&self, passenger_id: u32) -> u32 {
        self.passenger_encounters
            .get(&passenger_id)
            .copied()
            .unwrap_or(0)
    }

    /// Check if this is first encounter with a passenger
    pub fn is_first_encounter(&self, passenger_id: u32) -> bool {
        self.get_encounter_count(passenger_id) == 0
    }

    /// Check if skill is unlocked
    pub fn is_skill_unlocked(&self, skill_id: &str) -> bool {
        self.unlocked_skills.contains(&skill_id.to_string())
    }

    /// Stable key used for persisted route familiarity.
    pub fn route_key(route: RouteType) -> &'static str {
        match route {
            RouteType::Normal => "Normal",
            RouteType::Shortcut => "Shortcut",
            RouteType::Scenic => "Scenic",
            RouteType::Police => "Police",
        }
    }

    /// Record route usage across runs.
    pub fn record_route_usage(&mut self, route: RouteType) {
        *self
            .route_usage
            .entry(Self::route_key(route).to_string())
            .or_insert(0) += 1;
    }

    /// Get persistent familiarity for a route.
    pub fn get_route_usage(&self, route: RouteType) -> u32 {
        self.route_usage
            .get(Self::route_key(route))
            .copied()
            .unwrap_or(0)
    }

    /// Build a route mastery map for services that still operate on RouteType keys.
    pub fn route_mastery_map(&self) -> HashMap<RouteType, u32> {
        [
            RouteType::Normal,
            RouteType::Shortcut,
            RouteType::Scenic,
            RouteType::Police,
        ]
        .into_iter()
        .filter_map(|route| {
            let usage = self.get_route_usage(route);
            (usage > 0).then_some((route, usage))
        })
        .collect()
    }

    /// Add a leaderboard entry and sort (keep top 10)
    pub fn add_leaderboard_entry(&mut self, entry: LeaderboardEntry) {
        self.leaderboard.push(entry);
        self.leaderboard
            .sort_by_key(|entry| std::cmp::Reverse(entry.score));
        self.leaderboard.truncate(10);
    }

    /// Get almanac entry for a passenger (or default)
    pub fn get_almanac_entry(&self, passenger_id: u32) -> AlmanacEntry {
        self.almanac_progress
            .get(&passenger_id)
            .cloned()
            .unwrap_or(AlmanacEntry {
                passenger_id,
                encountered: false,
                knowledge_level: 0,
            })
    }

    /// Mark passenger as encountered
    pub fn mark_passenger_encountered(&mut self, passenger_id: u32) {
        let entry = self
            .almanac_progress
            .entry(passenger_id)
            .or_insert(AlmanacEntry {
                passenger_id,
                encountered: false,
                knowledge_level: 0,
            });
        entry.encountered = true;
    }

    /// Upgrade almanac knowledge level
    pub fn upgrade_almanac_knowledge(&mut self, passenger_id: u32, cost: u32) -> bool {
        if self.lore_fragments >= cost {
            let entry = self
                .almanac_progress
                .entry(passenger_id)
                .or_insert(AlmanacEntry {
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

    /// The game's fixed achievement definitions (id, name, description).
    pub(crate) fn achievement_definitions() -> Vec<Achievement> {
        vec![
            Achievement::new("first_shift", "First Night", "Complete your first shift"),
            Achievement::new("survivor", "Survivor", "Survive 10 shifts"),
            Achievement::new(
                "perfect_shift",
                "Perfect Shift",
                "Complete a shift without violating any rules",
            ),
            Achievement::new("big_earner", "Big Earner", "Earn $500 in a single shift"),
            Achievement::new(
                "almanac_scholar",
                "Almanac Scholar",
                "Master knowledge of 5 passengers",
            ),
            Achievement::new("skill_collector", "Skill Collector", "Unlock 3 skills"),
        ]
    }

    /// Reconcile the achievement registry with the current definitions.
    /// Safe to call every load: unlock state and dates are preserved.
    pub fn init_achievements(&mut self) {
        self.achievements
            .sync_definitions(Self::achievement_definitions());
    }

    /// Unlock an achievement
    pub fn unlock_achievement(&mut self, achievement_id: &str, date: String) -> bool {
        self.achievements
            .unlock_with_date(achievement_id, Some(date))
    }

    /// Check if achievement is unlocked
    pub fn is_achievement_unlocked(&self, achievement_id: &str) -> bool {
        self.achievements.is_unlocked(achievement_id)
    }

    /// Check and unlock achievements based on current stats
    /// Evaluate every achievement condition, returning the ids that unlocked
    /// for the first time on this call so the caller can pay their reward.
    /// Already-unlocked achievements are never reported twice.
    pub fn check_achievements(
        &mut self,
        shift_earnings: u32,
        shift_survived: bool,
        shift_violations: u32,
    ) -> Vec<String> {
        let mut newly_unlocked = Vec::new();
        #[cfg(not(target_arch = "wasm32"))]
        let now = {
            use chrono::Local;
            Local::now().format("%Y-%m-%d").to_string()
        };
        #[cfg(target_arch = "wasm32")]
        let now = "Today".to_string();

        let mastered = self
            .almanac_progress
            .values()
            .filter(|e| e.knowledge_level >= 3)
            .count();

        let conditions = [
            ("first_shift", self.total_shifts_completed >= 1),
            ("survivor", self.survival_bonuses >= 10),
            ("perfect_shift", shift_survived && shift_violations == 0),
            ("big_earner", shift_earnings >= 500),
            ("almanac_scholar", mastered >= 5),
            ("skill_collector", self.unlocked_skills.len() >= 3),
        ];

        for (id, met) in conditions {
            if met && self.unlock_achievement(id, now.clone()) {
                newly_unlocked.push(id.to_string());
            }
        }

        newly_unlocked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Saves written before the `Achievements` registry adoption stored
    /// `achievements` as a bare array. Loading one of those saves must not
    /// error (which would otherwise wipe the whole `PlayerStats` via the
    /// `unwrap_or_else(|_| PlayerStats::new())` fallback in `Game::new`).
    #[test]
    fn legacy_achievement_array_deserializes() {
        let mut stats = PlayerStats::new();
        stats.init_achievements();
        stats.unlock_achievement("first_shift", "2026-01-01".to_string());
        stats.total_shifts_completed = 3;

        // Reshape the current `{ "achievements": [...] }` registry encoding
        // back into the pre-migration bare-array shape a real legacy save
        // would contain, leaving every other field untouched.
        let mut value = serde_json::to_value(&stats).unwrap();
        let inner = value["achievements"]["achievements"].take();
        value["achievements"] = inner;
        let legacy_json = serde_json::to_string(&value).unwrap();

        let reloaded: PlayerStats = serde_json::from_str(&legacy_json).expect("legacy save loads");
        assert_eq!(reloaded.total_shifts_completed, 3);
        assert!(reloaded.is_achievement_unlocked("first_shift"));
        assert_eq!(reloaded.achievements.len(), stats.achievements.len());
    }

    /// Current saves serialize `achievements` as the registry's own shape
    /// (`{ "achievements": [...] }`); round-tripping must preserve unlocks.
    #[test]
    fn current_achievement_registry_round_trips() {
        let mut stats = PlayerStats::new();
        stats.init_achievements();
        stats.unlock_achievement("first_shift", "2026-01-01".to_string());

        let json = serde_json::to_string(&stats).unwrap();
        let reloaded: PlayerStats = serde_json::from_str(&json).unwrap();

        assert!(reloaded.is_achievement_unlocked("first_shift"));
        assert_eq!(reloaded.achievements.len(), stats.achievements.len());
    }

    /// A missing `achievements` key (very old saves, from before achievements
    /// existed at all) should default to an empty registry rather than
    /// failing deserialization.
    #[test]
    fn missing_achievements_field_defaults_empty() {
        let stats = PlayerStats::new();
        let mut value = serde_json::to_value(&stats).unwrap();
        value.as_object_mut().unwrap().remove("achievements");

        let reloaded: PlayerStats = serde_json::from_value(value).expect("defaults apply");
        assert_eq!(reloaded.achievements.len(), 0);
    }
}

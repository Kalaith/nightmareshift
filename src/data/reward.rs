//! Meta-progression payouts, loaded from `rewardData.json`.
//!
//! The bank balance and lore fragments a run yields are what gate the skill
//! tree and the almanac. Per-shift earnings alone pace the twenty-node tree
//! far slower than the almanac, so the two milestone systems the game already
//! tracks — achievements and completing a full multi-night run — pay into
//! them here rather than being scoreboard entries with nothing behind them.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single payout into the meta-progression currencies.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Payout {
    /// Bank balance, spent in the skill tree.
    #[serde(default)]
    pub bank: u32,
    /// Lore fragments, spent in the almanac.
    #[serde(default)]
    pub lore: u32,
}

impl Payout {
    /// True when this payout does nothing.
    pub fn is_empty(&self) -> bool {
        self.bank == 0 && self.lore == 0
    }
}

/// Payout for surviving every night of a run.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RunCompletionReward {
    #[serde(default)]
    pub bank: u32,
    #[serde(default)]
    pub lore: u32,
    /// Additional bank per night survived, so a longer run pays more.
    #[serde(rename = "bankPerNight", default)]
    pub bank_per_night: u32,
}

impl RunCompletionReward {
    /// The payout for completing a run of `nights` nights.
    pub fn payout(&self, nights: u32) -> Payout {
        Payout {
            bank: self.bank + self.bank_per_night * nights,
            lore: self.lore,
        }
    }
}

/// The rate at which lore fragments are sold back to the depot for bank.
///
/// The two meta-currencies had disjoint sinks: lore bought almanac levels
/// and nothing else, so it went dead once every passenger was mastered,
/// while bank paced the skill tree an order of magnitude slower. Trading one
/// for the other makes the almanac and the skill tree compete for the same
/// resource, so studying the roster deeply and buying broadly into the tree
/// become a choice rather than two unrelated tracks.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LoreExchange {
    /// Fragments spent per trade.
    #[serde(default)]
    pub lore: u32,
    /// Bank received per trade.
    #[serde(default)]
    pub bank: u32,
}

impl LoreExchange {
    /// True when the exchange is configured to actually trade something.
    pub fn is_available(&self) -> bool {
        self.lore > 0 && self.bank > 0
    }
}

/// All meta-progression payouts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewardData {
    /// Keyed by achievement id, paid once when the achievement first unlocks.
    #[serde(default)]
    pub achievements: HashMap<String, Payout>,
    #[serde(rename = "runCompletion", default)]
    pub run_completion: RunCompletionReward,
    #[serde(rename = "loreExchange", default)]
    pub lore_exchange: LoreExchange,
}

impl RewardData {
    /// The payout for first unlocking `achievement_id`.
    pub fn for_achievement(&self, achievement_id: &str) -> Payout {
        self.achievements
            .get(achievement_id)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_rewards;
    use crate::state::PlayerStats;
    use std::collections::HashSet;

    /// Every achievement must be worth something. An achievement with no
    /// reward entry is a scoreboard line the meta-progression never sees.
    #[test]
    fn every_achievement_pays_out() {
        let rewards = load_rewards();
        for achievement in PlayerStats::achievement_definitions() {
            let payout = rewards.for_achievement(&achievement.id);
            assert!(
                !payout.is_empty(),
                "achievement {:?} has no reward",
                achievement.id
            );
        }
    }

    /// Conversely, a reward keyed to an id no achievement uses is dead data
    /// that silently pays nobody — usually a rename or a typo.
    #[test]
    fn every_reward_names_a_real_achievement() {
        let ids: HashSet<String> = PlayerStats::achievement_definitions()
            .into_iter()
            .map(|a| a.id)
            .collect();
        for id in load_rewards().achievements.keys() {
            assert!(ids.contains(id), "reward {id:?} names no achievement");
        }
    }

    /// The exchange is what stops lore going dead once every passenger is
    /// mastered, so it has to be configured and has to be a real trade.
    #[test]
    fn lore_exchanges_for_bank() {
        let rate = load_rewards().lore_exchange;
        assert!(rate.is_available(), "lore exchange is not configured");
    }

    /// Selling the entire almanac budget must not by itself buy the whole
    /// skill tree, or the two systems stop competing and lore just becomes
    /// bank with extra steps.
    #[test]
    fn exchanging_lore_does_not_trivialise_the_tree() {
        let rate = load_rewards().lore_exchange;
        let almanac_budget: u32 = crate::data::loader::load_passengers().len() as u32 * (1 + 3 + 5);
        let bank_if_all_sold = almanac_budget / rate.lore * rate.bank;
        let tree_cost: u32 = crate::data::loader::load_skill_tree()
            .iter()
            .map(|s| s.cost)
            .sum();
        assert!(
            bank_if_all_sold < tree_cost,
            "selling {almanac_budget} lore yields {bank_if_all_sold},              which already covers the {tree_cost} tree"
        );
    }

    /// Surviving a whole run is the game's headline result and must pay.
    #[test]
    fn completing_a_run_pays_out() {
        let rewards = load_rewards();
        let payout = rewards.run_completion.payout(5);
        assert!(!payout.is_empty(), "run completion pays nothing");
        assert!(
            payout.bank > rewards.run_completion.bank,
            "bankPerNight never applied"
        );
    }
}

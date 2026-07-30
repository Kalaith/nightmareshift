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

/// What giving a passenger an item they actually asked for is worth.
///
/// `wantedItems` already gates whether a trade is offered, but every trade
/// resolved identically, so handing over the exact thing a passenger wanted
/// paid no more than handing over junk. This is what makes the choice of
/// which item to give matter.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct WantedTradeBonus {
    /// Need level removed — settling a passenger who got what they wanted.
    #[serde(rename = "needRelief", default)]
    pub need_relief: i32,
    /// Positive reputation entries credited with this passenger.
    #[serde(rename = "reputationBonus", default)]
    pub reputation_bonus: u32,
}

impl WantedTradeBonus {
    /// True when the bonus does anything.
    pub fn is_active(&self) -> bool {
        self.need_relief > 0 || self.reputation_bonus > 0
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
    #[serde(rename = "wantedTrade", default)]
    pub wanted_trade: WantedTradeBonus,
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

impl Payout {
    /// How this payout reads on a card, or `None` when it is worth nothing.
    ///
    /// Achievements fund both halves of the meta-progression -- bank buys
    /// skills, lore buys almanac levels -- and the amounts run from $250 with 3
    /// lore up to $1500 with 8. All of it was authored, paid on unlock, and
    /// shown nowhere, so a player could not tell which goal was worth chasing
    /// first. Big Earner pays the most bank in the game and the card said only
    /// "Locked".
    pub fn describe(&self) -> Option<String> {
        match (self.bank, self.lore) {
            (0, 0) => None,
            (bank, 0) => Some(format!("pays ${bank}")),
            (0, lore) => Some(format!("pays {lore} lore")),
            (bank, lore) => Some(format!("pays ${bank} and {lore} lore")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_rewards;
    use crate::state::PlayerStats;
    use std::collections::HashSet;

    /// Every achievement's payout has to be sayable, since the card now prints
    /// it. A payout of nothing prints nothing, and that must not happen for a
    /// real achievement -- the existing test above covers it being non-zero, and
    /// this covers the sentence actually forming.
    #[test]
    fn every_achievement_payout_reads_as_a_sentence() {
        let rewards = load_rewards();
        for achievement in PlayerStats::achievement_definitions() {
            let payout = rewards.for_achievement(&achievement.id);
            let line = payout.describe().unwrap_or_else(|| {
                panic!("{} has nothing to say about what it pays", achievement.id)
            });
            if payout.bank > 0 {
                assert!(
                    line.contains(&payout.bank.to_string()),
                    "{} pays ${} and says {line:?}",
                    achievement.id,
                    payout.bank
                );
            }
            if payout.lore > 0 {
                assert!(
                    line.contains(&payout.lore.to_string()),
                    "{} pays {} lore and says {line:?}",
                    achievement.id,
                    payout.lore
                );
            }
        }
    }

    /// Both halves are named when both are paid, and a payout of one currency
    /// does not claim the other.
    #[test]
    fn a_payout_names_only_what_it_pays() {
        use crate::data::reward::Payout;

        assert_eq!(Payout { bank: 0, lore: 0 }.describe(), None);

        let bank_only = Payout { bank: 250, lore: 0 }.describe().expect("a line");
        assert!(bank_only.contains("250") && !bank_only.contains("lore"));

        let lore_only = Payout { bank: 0, lore: 4 }.describe().expect("a line");
        assert!(lore_only.contains("lore") && !lore_only.contains('$'));

        let both = Payout { bank: 800, lore: 6 }.describe().expect("a line");
        assert!(both.contains("800") && both.contains('6'));
    }

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

    /// A lore fragment must be worth little enough that trading supplements
    /// driving rather than replacing it.
    ///
    /// This replaces a weaker check that bounded the exchange against what
    /// the almanac *costs* — 144 fragments to master the roster — and passed
    /// comfortably while the mechanic printed money. A player does not earn
    /// 144 fragments; a sixty-shift session measured 1091, and at the old
    /// twenty-four dollars a fragment that surplus was worth $26,184 against
    /// a $19,800 tree. The bound has to be against the fragment's own value,
    /// because income is what the almanac budget failed to predict.
    #[test]
    fn a_lore_fragment_cannot_be_worth_much() {
        let rate = load_rewards().lore_exchange;
        let cheapest_skill = crate::data::loader::load_skill_tree()
            .iter()
            .map(|skill| skill.cost)
            .min()
            .expect("the tree has nodes");

        let per_fragment = rate.bank as f32 / rate.lore.max(1) as f32;
        let fragments_for_cheapest = cheapest_skill as f32 / per_fragment;
        assert!(
            fragments_for_cheapest >= 50.0,
            "{fragments_for_cheapest:.0} fragments buys the cheapest ${cheapest_skill} skill;              lore is standing in for driving"
        );
    }

    /// Giving a passenger what they asked for has to be worth more than
    /// giving them anything else, or `wantedItems` only ever decides whether
    /// an offer appears and never what the player should hand over.
    #[test]
    fn giving_a_wanted_item_is_rewarded() {
        assert!(
            load_rewards().wanted_trade.is_active(),
            "a wanted trade pays nothing"
        );
    }

    /// Every passenger must want at least one item that the player can
    /// actually part with, otherwise their trade offer can never be
    /// satisfied with the thing they asked for.
    #[test]
    fn every_passengers_wants_are_giveable() {
        let catalog = crate::data::loader::load_item_catalog();
        for passenger in crate::data::loader::load_passengers() {
            if !passenger.wants_trade {
                continue;
            }
            let giveable = passenger
                .wanted_items
                .iter()
                .filter(|name| catalog.get(name).can_trade)
                .count();
            assert!(
                giveable > 0,
                "{} wants nothing the player can hand over",
                passenger.name
            );
        }
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

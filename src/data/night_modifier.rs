//! Per-night run modifiers, matching `nightModifierData.json`.
//!
//! A modifier is a named twist an ordinary mid-run night may carry — "Blood
//! Moon: fares +20%, one rule deeper" — rolled once in `begin_night` on the
//! seeded stream and applied through hooks that already exist: the quota,
//! the difficulty that drives the rule draw, the fare calculation, the
//! starting fuel, and the lore payout. Night 1 never rolls one (the opening
//! stays learnable) and The Last Fare never rolls one (that night is
//! authored whole).

use serde::{Deserialize, Serialize};

fn one_f32() -> f32 {
    1.0
}

fn one_weight() -> u32 {
    1
}

/// One authored night twist. Every numeric field defaults to "no effect",
/// so an entry authors only what it changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NightModifier {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Relative draw weight among the modifiers once a night rolls one.
    #[serde(default = "one_weight")]
    pub weight: u32,
    /// Multiplier on every fare earned this night.
    #[serde(default = "one_f32")]
    pub fare_mult: f32,
    /// Multiplier on the night's earnings quota.
    #[serde(default = "one_f32")]
    pub quota_mult: f32,
    /// Added to the night's effective difficulty before the rule draw
    /// (still capped at `SCORING.MAX_DIFFICULTY`).
    #[serde(default)]
    pub difficulty_bonus: u32,
    /// Applied to the starting fuel, clamped to at least a sliver.
    #[serde(default)]
    pub start_fuel_delta: i32,
    /// Added to the shift's lore payout.
    #[serde(default)]
    pub lore_bonus: u32,
}

/// The modifier deck: one roll decides whether tonight carries one at all,
/// a weighted draw decides which.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NightModifierData {
    /// Chance an eligible night carries any modifier.
    pub chance: f32,
    pub modifiers: Vec<NightModifier>,
}

impl NightModifierData {
    /// Roll tonight's modifier on the seeded stream, or nothing.
    pub fn roll(&self, rng: &mut macroquad_toolkit::rng::SeededRng) -> Option<NightModifier> {
        if self.modifiers.is_empty() || !rng.chance(self.chance.clamp(0.0, 1.0)) {
            return None;
        }
        let total: u32 = self.modifiers.iter().map(|m| m.weight.max(1)).sum();
        let mut pick = rng.range_i32(0, total as i32) as u32;
        for modifier in &self.modifiers {
            let weight = modifier.weight.max(1);
            if pick < weight {
                return Some(modifier.clone());
            }
            pick -= weight;
        }
        self.modifiers.last().cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::load_night_modifiers;
    use std::collections::HashSet;

    /// Every authored modifier must be well-formed: named, described, its
    /// draw chance a probability, its multipliers positive (a zero fare or
    /// quota multiplier is a broken economy, not a twist), and its fuel
    /// delta survivable against the 100-point tank.
    #[test]
    fn the_modifier_deck_is_well_formed() {
        let deck = load_night_modifiers();
        assert!(
            (0.0..=1.0).contains(&deck.chance),
            "chance {} is not a probability",
            deck.chance
        );
        assert!(!deck.modifiers.is_empty(), "the deck is empty");

        let mut ids = HashSet::new();
        for modifier in &deck.modifiers {
            assert!(
                ids.insert(modifier.id.clone()),
                "duplicate modifier id {:?}",
                modifier.id
            );
            assert!(!modifier.name.trim().is_empty(), "{}: no name", modifier.id);
            assert!(
                !modifier.description.trim().is_empty(),
                "{}: no description",
                modifier.id
            );
            assert!(
                modifier.fare_mult > 0.0 && modifier.quota_mult > 0.0,
                "{}: a non-positive multiplier",
                modifier.id
            );
            assert!(
                modifier.start_fuel_delta > -90,
                "{}: fuel delta {} leaves no night to drive",
                modifier.id,
                modifier.start_fuel_delta
            );
            assert!(
                modifier.weight > 0,
                "{}: zero weight can never be drawn",
                modifier.id
            );
        }
    }

    /// A modifier must change something, or it is a name on the briefing
    /// and nothing else.
    #[test]
    fn every_modifier_moves_a_lever() {
        for modifier in load_night_modifiers().modifiers {
            let moves = modifier.fare_mult != 1.0
                || modifier.quota_mult != 1.0
                || modifier.difficulty_bonus > 0
                || modifier.start_fuel_delta != 0
                || modifier.lore_bonus > 0;
            assert!(moves, "{}: authors no effect", modifier.id);
        }
    }

    /// The weighted draw always lands on an authored entry.
    #[test]
    fn the_roll_draws_from_the_deck() {
        let deck = load_night_modifiers();
        let mut rng = macroquad_toolkit::rng::SeededRng::new(0x0DDB);
        let mut drawn = 0;
        for _ in 0..200 {
            if let Some(modifier) = deck.roll(&mut rng) {
                drawn += 1;
                assert!(deck.modifiers.iter().any(|m| m.id == modifier.id));
            }
        }
        assert!(
            drawn > 0,
            "200 rolls at chance {} drew nothing",
            deck.chance
        );
    }
}

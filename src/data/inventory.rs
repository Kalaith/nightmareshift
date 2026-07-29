//! Inventory item types matching the item system.

use super::passenger::Rarity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Named item-name pools keyed by supernatural category, loaded from
/// `itemPoolData.json`. Used to generate an item drop when a passenger has no
/// explicit `dropItems`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemPools {
    #[serde(default)]
    pub ghost: Vec<String>,
    #[serde(default)]
    pub vampire: Vec<String>,
    #[serde(default)]
    pub demon: Vec<String>,
    #[serde(default)]
    pub occult: Vec<String>,
    #[serde(default)]
    pub holy: Vec<String>,
    #[serde(default)]
    pub common: Vec<String>,
}

impl ItemPools {
    /// Pick a random item name from the named category, falling back to the
    /// common pool (and finally a hardcoded name) if a pool is empty.
    pub fn pick(&self, category: &str) -> String {
        let pool = match category {
            "ghost" => &self.ghost,
            "vampire" => &self.vampire,
            "demon" => &self.demon,
            "occult" => &self.occult,
            "holy" => &self.holy,
            _ => &self.common,
        };
        let pool = if pool.is_empty() { &self.common } else { pool };
        if pool.is_empty() {
            return "Old Key".to_string();
        }
        pool[macroquad_toolkit::rng::gen_range(0, pool.len())].clone()
    }

    /// Every name any pool can produce, in declaration order. Used by the
    /// catalog-coverage test so a name can never be droppable without also
    /// being defined in `itemData.json`.
    #[cfg(test)]
    pub fn all_names(&self) -> Vec<&str> {
        [
            &self.ghost,
            &self.vampire,
            &self.demon,
            &self.occult,
            &self.holy,
            &self.common,
        ]
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect()
    }
}

/// Type of inventory item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ItemType {
    Protective,
    Cursed,
    Consumable,
    Tradeable,
    #[default]
    Story,
}

/// Type of item effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemEffectType {
    FuelBonus,
    TimeBonus,
    RuleImmunity,
    SupernaturalProtection,
    FuelDrain,
    TimePenalty,
    RuleTrigger,
    ReputationModifier,
}

/// An effect provided by an item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemEffect {
    #[serde(rename = "type")]
    pub effect_type: ItemEffectType,
    pub value: i32,
    #[serde(default)]
    pub duration: Option<u32>,
    #[serde(default)]
    pub condition: Option<String>,
}

/// Type of curse penalty
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursePenalty {
    FuelDrain,
    TimeAcceleration,
    ForcedChoices,
    AttractingDanger,
}

/// Properties of a cursed item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursedProperties {
    #[serde(rename = "penaltyType")]
    pub penalty_type: CursePenalty,
    #[serde(rename = "penaltyValue")]
    pub penalty_value: i32,
    #[serde(rename = "triggersAfter")]
    pub triggers_after: u32,
    #[serde(rename = "canBeRemoved")]
    pub can_be_removed: bool,
    #[serde(rename = "removalCondition")]
    pub removal_condition: Option<String>,
}

/// Type of protection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectionType {
    SupernaturalImmunity,
    RuleForgiveness,
    SafePassage,
    LuckyEncounters,
}

/// Properties of a protective item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectiveProperties {
    #[serde(rename = "protectionType")]
    pub protection_type: ProtectionType,
    #[serde(rename = "protectionStrength")]
    pub protection_strength: u32,
    #[serde(rename = "usesRemaining")]
    pub uses_remaining: Option<u32>,
    #[serde(rename = "protectsAgainst")]
    pub protects_against: Option<Vec<String>>,
}

/// An item in the player's inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub source: String,
    #[serde(rename = "backstoryItem")]
    pub backstory_item: bool,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    #[serde(default)]
    pub rarity: Rarity,
    pub description: String,
    #[serde(default)]
    pub effects: Vec<ItemEffect>,
    pub durability: Option<u32>,
    #[serde(rename = "maxDurability")]
    pub max_durability: Option<u32>,
    #[serde(rename = "acquiredAt")]
    pub acquired_at: f64,
    #[serde(rename = "canUse")]
    pub can_use: bool,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "cursedProperties")]
    pub cursed_properties: Option<CursedProperties>,
    #[serde(rename = "protectiveProperties")]
    pub protective_properties: Option<ProtectiveProperties>,
}

impl InventoryItem {
    /// Check if this item is cursed
    pub fn is_cursed(&self) -> bool {
        self.item_type == ItemType::Cursed || self.cursed_properties.is_some()
    }

    /// Check if curse should trigger based on time possessed
    pub fn should_trigger_curse(&self, current_time: f64) -> bool {
        if let Some(ref curse) = self.cursed_properties {
            let possession_minutes = (current_time - self.acquired_at) / 60.0;
            possession_minutes >= curse.triggers_after as f64
        } else {
            false
        }
    }

    /// Whether this item can be handed to a passenger.
    ///
    /// `canTrade` says whether it is the sort of thing anyone would take;
    /// a curse's `canBeRemoved` says whether it will let itself be given away
    /// at all. The Contract Fragment has the driver's name on it and refuses.
    /// Both were authored and only the first was read, so a binding contract
    /// could be traded off like a spare coin.
    pub fn can_be_given_away(&self) -> bool {
        if !self.can_trade {
            return false;
        }
        self.cursed_properties
            .as_ref()
            .map(|curse| curse.can_be_removed)
            .unwrap_or(true)
    }

    /// Check if item is broken (durability depleted)
    pub fn is_broken(&self) -> bool {
        self.durability.map(|d| d == 0).unwrap_or(false)
    }

    /// Apply deterioration based on time
    pub fn apply_deterioration(&mut self, current_time: f64) {
        if let (Some(durability), Some(_max)) = (self.durability.as_mut(), self.max_durability) {
            let age_minutes = (current_time - self.acquired_at) / 60.0;
            if age_minutes > 10.0 {
                let deterioration = ((age_minutes - 10.0) / 10.0) as u32;
                *durability = durability.saturating_sub(deterioration);
            }
        }
    }
}

/// The item catalog, loaded from `itemData.json`: every droppable item name
/// mapped to the template used to mint an `InventoryItem`.
///
/// Keys are matched case-insensitively (they are stored lowercase), so pool
/// entries and passenger `dropItems` can use display casing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemCatalog {
    templates: HashMap<String, ItemTemplate>,
}

impl ItemCatalog {
    /// Look up a template by item name. Unknown names fall back to an inert
    /// story keepsake rather than failing the drop outright.
    pub fn get(&self, name: &str) -> ItemTemplate {
        self.templates
            .get(&name.to_lowercase())
            .cloned()
            .unwrap_or_else(ItemTemplate::keepsake)
    }

    /// Every defined item name.
    #[cfg(test)]
    pub fn names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// True when the catalog defines this name.
    #[cfg(test)]
    pub fn contains(&self, name: &str) -> bool {
        self.templates.contains_key(&name.to_lowercase())
    }

    /// Create an inventory item from the catalog.
    pub fn create_item(&self, name: &str, source: &str, current_time: f64) -> InventoryItem {
        let template = self.get(name);
        InventoryItem {
            id: format!("{}_{}", name.replace(' ', "_"), current_time as u64),
            name: name.to_string(),
            source: source.to_string(),
            backstory_item: false,
            item_type: template.item_type,
            rarity: template.rarity,
            description: template.description,
            effects: template.effects,
            durability: template.max_durability,
            max_durability: template.max_durability,
            acquired_at: current_time,
            can_use: template.can_use,
            can_trade: template.can_trade,
            cursed_properties: template.cursed_properties,
            protective_properties: template.protective_properties,
        }
    }
}

/// Template for creating inventory items, as authored in `itemData.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTemplate {
    #[serde(rename = "type", default)]
    pub item_type: ItemType,
    #[serde(default)]
    pub rarity: Rarity,
    pub description: String,
    #[serde(rename = "canUse", default)]
    pub can_use: bool,
    #[serde(rename = "canTrade", default)]
    pub can_trade: bool,
    #[serde(rename = "maxDurability", default)]
    pub max_durability: Option<u32>,
    #[serde(rename = "cursedProperties", default)]
    pub cursed_properties: Option<CursedProperties>,
    #[serde(rename = "protectiveProperties", default)]
    pub protective_properties: Option<ProtectiveProperties>,
    #[serde(default)]
    pub effects: Vec<ItemEffect>,
}

impl ItemTemplate {
    /// The inert fallback used when an item name is not in the catalog.
    fn keepsake() -> Self {
        Self {
            item_type: ItemType::Story,
            rarity: Rarity::Common,
            description: "A mysterious object left behind by a passenger".to_string(),
            can_use: false,
            can_trade: false,
            max_durability: Some(100),
            cursed_properties: None,
            protective_properties: None,
            effects: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::loader::{load_item_catalog, load_item_pools, load_passengers};

    /// Every name a pool can produce must exist in the catalog. Without this,
    /// adding a name to `itemPoolData.json` silently mints an inert keepsake.
    #[test]
    fn every_pooled_item_is_in_the_catalog() {
        let pools = load_item_pools();
        let catalog = load_item_catalog();
        let missing: Vec<&str> = pools
            .all_names()
            .into_iter()
            .filter(|name| !catalog.contains(name))
            .collect();
        assert!(
            missing.is_empty(),
            "items missing from catalog: {missing:?}"
        );
    }

    /// Passenger-specific `dropItems` go through the same catalog.
    #[test]
    fn every_passenger_drop_item_is_in_the_catalog() {
        let catalog = load_item_catalog();
        let missing: Vec<String> = load_passengers()
            .iter()
            .flat_map(|p| p.drop_items.iter())
            .filter(|name| !catalog.contains(name))
            .cloned()
            .collect();
        assert!(
            missing.is_empty(),
            "items missing from catalog: {missing:?}"
        );
    }

    /// The point of the catalog: an item a passenger hands you must do
    /// something. Every catalog entry either carries effects, wards you, or
    /// curses you — a droppable item is never pure inventory clutter.
    #[test]
    fn every_catalog_item_does_something() {
        let catalog = load_item_catalog();
        let inert: Vec<&String> = catalog
            .templates
            .iter()
            .filter(|(_, t)| {
                t.effects.is_empty()
                    && t.protective_properties.is_none()
                    && t.cursed_properties.is_none()
            })
            .map(|(name, _)| name)
            .collect();
        assert!(inert.is_empty(), "items with no effect at all: {inert:?}");
    }

    /// Every curse must tell the player how to be rid of it, or the penalty
    /// is something that happens to them with no way to act on it.
    #[test]
    fn every_curse_names_its_way_out() {
        let catalog = load_item_catalog();
        let mut names = catalog.names();
        names.sort();
        for name in names {
            let Some(curse) = catalog.get(&name).cursed_properties else {
                continue;
            };
            if !curse.can_be_removed {
                continue;
            }
            let condition = curse.removal_condition.unwrap_or_default();
            assert!(
                !condition.trim().is_empty(),
                "{name:?} can be removed but does not say how"
            );
        }
    }

    /// A curse that refuses to be given away must not also be marked
    /// tradeable, or the inventory offers a way out that the trade refuses.
    #[test]
    fn an_unremovable_curse_is_not_offered_for_trade() {
        let catalog = load_item_catalog();
        let mut names = catalog.names();
        names.sort();
        for name in names {
            let template = catalog.get(&name);
            let Some(curse) = &template.cursed_properties else {
                continue;
            };
            if curse.can_be_removed {
                continue;
            }
            assert!(
                !template.can_trade,
                "{name:?} cannot be removed but is marked canTrade"
            );
        }
    }

    /// A usable item must actually have effects to apply, or "Use" is a no-op
    /// button on an inventory row.
    #[test]
    fn usable_items_have_effects() {
        let catalog = load_item_catalog();
        let empty: Vec<&String> = catalog
            .templates
            .iter()
            .filter(|(_, t)| t.can_use && t.effects.is_empty())
            .map(|(name, _)| name)
            .collect();
        assert!(empty.is_empty(), "usable items with no effects: {empty:?}");
    }
}

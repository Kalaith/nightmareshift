//! Inventory item types matching the item system.

use super::passenger::Rarity;
use serde::{Deserialize, Serialize};

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
    pub duration: Option<u32>,
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

/// Database of known item templates
pub struct ItemDatabase;

impl ItemDatabase {
    /// Get item data by name
    pub fn get_item_data(name: &str) -> ItemTemplate {
        match name.to_lowercase().as_str() {
            "old locket" => ItemTemplate {
                item_type: ItemType::Cursed,
                rarity: Rarity::Uncommon,
                description: "A tarnished locket that whispers forgotten names".to_string(),
                can_use: false,
                can_trade: true,
                max_durability: Some(100),
                cursed_properties: Some(CursedProperties {
                    penalty_type: CursePenalty::AttractingDanger,
                    penalty_value: 1,
                    triggers_after: 20,
                    can_be_removed: true,
                    removal_condition: Some("Trade with Sister Agnes".to_string()),
                }),
                protective_properties: None,
                effects: Vec::new(),
            },
            "crystal pendant" => ItemTemplate {
                item_type: ItemType::Protective,
                rarity: Rarity::Rare,
                description: "A shimmering crystal that wards off supernatural influence"
                    .to_string(),
                can_use: true,
                can_trade: true,
                max_durability: None,
                cursed_properties: None,
                protective_properties: Some(ProtectiveProperties {
                    protection_type: ProtectionType::SupernaturalImmunity,
                    protection_strength: 2,
                    uses_remaining: Some(3),
                    protects_against: None,
                }),
                effects: vec![ItemEffect {
                    effect_type: ItemEffectType::RuleImmunity,
                    value: 1,
                    duration: Some(30),
                    condition: Some("supernatural_encounter".to_string()),
                }],
            },
            "blessed medallion" => ItemTemplate {
                item_type: ItemType::Protective,
                rarity: Rarity::Rare,
                description: "A holy medallion that repels dark influences".to_string(),
                can_use: true,
                can_trade: false,
                max_durability: None,
                cursed_properties: None,
                protective_properties: Some(ProtectiveProperties {
                    protection_type: ProtectionType::SupernaturalImmunity,
                    protection_strength: 3,
                    uses_remaining: Some(5),
                    protects_against: None,
                }),
                effects: vec![ItemEffect {
                    effect_type: ItemEffectType::SupernaturalProtection,
                    value: 3,
                    duration: None,
                    condition: None,
                }],
            },
            "soul protection ward" => ItemTemplate {
                item_type: ItemType::Protective,
                rarity: Rarity::Legendary,
                description: "A ward crafted by The Collector, protecting your very essence"
                    .to_string(),
                can_use: true,
                can_trade: false,
                max_durability: None,
                cursed_properties: None,
                protective_properties: Some(ProtectiveProperties {
                    protection_type: ProtectionType::SupernaturalImmunity,
                    protection_strength: 5,
                    uses_remaining: Some(1),
                    protects_against: Some(vec!["16".to_string()]), // Death's Taxi Driver
                }),
                effects: Vec::new(),
            },
            "withered flowers" => ItemTemplate {
                item_type: ItemType::Story,
                rarity: Rarity::Common,
                description: "Flowers that never truly die, holding memories of the past"
                    .to_string(),
                can_use: false,
                can_trade: false,
                max_durability: Some(50),
                cursed_properties: None,
                protective_properties: None,
                effects: Vec::new(),
            },
            _ => ItemTemplate {
                item_type: ItemType::Story,
                rarity: Rarity::Common,
                description: "A mysterious object left behind by a passenger".to_string(),
                can_use: false,
                can_trade: false,
                max_durability: Some(100),
                cursed_properties: None,
                protective_properties: None,
                effects: Vec::new(),
            },
        }
    }

    /// Create an inventory item from a template
    pub fn create_item(name: &str, source: &str, current_time: f64) -> InventoryItem {
        let template = Self::get_item_data(name);
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

/// Template for creating inventory items
pub struct ItemTemplate {
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub description: String,
    pub can_use: bool,
    pub can_trade: bool,
    pub max_durability: Option<u32>,
    pub cursed_properties: Option<CursedProperties>,
    pub protective_properties: Option<ProtectiveProperties>,
    pub effects: Vec<ItemEffect>,
}

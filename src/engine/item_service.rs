//! Item service for managing drops, trading, and effects.

use crate::data::*;
use crate::state::*;
use rand::Rng;

/// Item drop result
#[derive(Debug, Clone)]
pub struct ItemDrop {
    pub item: InventoryItem,
}

/// Trade offer from a passenger
#[derive(Debug, Clone)]
pub struct TradeOffer {
    pub passenger_name: String,
    pub offered_item: InventoryItem,
}

/// Item service for managing all item-related logic
pub struct ItemService;

impl ItemService {
    /// Calculate drop chance based on passenger and ride outcome
    pub fn calculate_drop_chance(
        passenger: &Passenger,
        route_type: RouteType,
        backstory_unlocked: bool,
        constants: &ConstantsData,
    ) -> f32 {
        let mut base_chance: f32 = constants.probabilities.item_drop;
        
        // Rarity modifiers
        base_chance *= match passenger.rarity {
            Rarity::Common => 0.5,
            Rarity::Uncommon => 0.75,
            Rarity::Rare => 1.0,
            Rarity::Legendary => 1.5,
        };

        // Scenic routes increase drop chance
        if route_type == RouteType::Scenic {
            base_chance *= 1.3;
        }

        // Backstory unlock significantly increases drop chance
        if backstory_unlocked {
            base_chance *= 1.5;
        }

        // Supernatural passengers more likely to drop items
        if passenger.is_supernatural {
            base_chance *= 1.2;
        }

        base_chance.min(0.80)
    }

    /// Generate an item drop from a passenger
    pub fn generate_drop(
        passenger: &Passenger,
        route_type: RouteType,
        backstory_unlocked: bool,

        current_time: f64,
        constants: &ConstantsData,
    ) -> Option<ItemDrop> {
        let drop_chance = Self::calculate_drop_chance(passenger, route_type, backstory_unlocked, constants);
        
        if rand::random::<f32>() > drop_chance {
            return None;
        }

        // Determine item based on passenger
        let item = Self::select_item_for_passenger(passenger, current_time);

        Some(ItemDrop {
            item,
        })
    }

    /// Select an appropriate item for a passenger to drop
    fn select_item_for_passenger(passenger: &Passenger, current_time: f64) -> InventoryItem {
        // Check if passenger has specific drop items
        if !passenger.drop_items.is_empty() {
            let idx = rand::thread_rng().gen_range(0..passenger.drop_items.len());
            let item_name = &passenger.drop_items[idx];
            return ItemDatabase::create_item(item_name, &passenger.name, current_time);
        }

        // Otherwise generate based on supernatural type
        let item_name = match passenger.supernatural.as_str() {
            "ghost" | "specter" => Self::random_ghost_item(),
            "vampire" => Self::random_vampire_item(),
            "demon" => Self::random_demon_item(),
            "psychic" | "fortune_teller" => Self::random_occult_item(),
            "priest" | "nun" => Self::random_holy_item(),
            _ => Self::random_common_item(),
        };

        ItemDatabase::create_item(&item_name, &passenger.name, current_time)
    }

    fn random_ghost_item() -> String {
        let items = ["Old Locket", "Withered Flowers", "Faded Photograph", "Dusty Mirror"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    fn random_vampire_item() -> String {
        let items = ["Blood Vial", "Ancient Coin", "Velvet Cloak Scrap", "Ornate Ring"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    fn random_demon_item() -> String {
        let items = ["Sulfur Crystal", "Burning Coal", "Contract Fragment", "Cursed Dice"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    fn random_occult_item() -> String {
        let items = ["Crystal Pendant", "Tarot Card", "Incense Bundle", "Rune Stone"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    fn random_holy_item() -> String {
        let items = ["Blessed Medallion", "Holy Water Vial", "Prayer Beads", "Saint's Icon"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    fn random_common_item() -> String {
        let items = ["Forgotten Wallet", "Lost Phone", "Crumpled Note", "Old Key"];
        items[rand::thread_rng().gen_range(0..items.len())].to_string()
    }

    /// Check if a passenger wants to trade
    pub fn check_trade_offer(
        passenger: &Passenger,
        inventory: &[InventoryItem],
        constants: &ConstantsData,
    ) -> Option<TradeOffer> {
        // Only some passengers trade
        if !passenger.wants_trade {
            return None;
        }

        // Check if player has something the passenger wants
        let wanted_item = inventory.iter().find(|item| {
            passenger.wanted_items.contains(&item.name) && item.can_trade
        });

        if wanted_item.is_some() || rand::random::<f32>() < constants.probabilities.trade_offer_chance {
            // Generate a trade offer
            let current_time = 0.0; // Will be filled in by caller
            let offered_item = Self::select_item_for_passenger(passenger, current_time);

            return Some(TradeOffer {
                passenger_name: passenger.name.clone(),
                offered_item,
            });
        }

        None
    }

    /// Apply curse penalties from all cursed items
    pub fn apply_curse_penalties(inventory: &[InventoryItem], state: &mut GameState, current_time: f64) {
        for item in inventory {
            if item.is_cursed() && item.should_trigger_curse(current_time) {
                if let Some(ref curse) = item.cursed_properties {
                    match curse.penalty_type {
                        CursePenalty::FuelDrain => {
                            state.fuel = (state.fuel - curse.penalty_value as f32).max(0.0);
                        }
                        CursePenalty::TimeAcceleration => {
                            state.time_remaining = state.time_remaining.saturating_sub(curse.penalty_value as u32);
                        }
                        CursePenalty::AttractingDanger => {
                            // Increases risk level of next route
                            state.curse_danger_bonus += curse.penalty_value as u32;
                        }
                        CursePenalty::ForcedChoices => {
                            // TODO: Implement forced route choices
                        }
                    }
                }
            }
        }
    }
}

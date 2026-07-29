//! Item service for managing drops, trading, and effects.

use crate::data::*;
use crate::state::*;

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
        item_pools: &ItemPools,
        catalog: &ItemCatalog,
    ) -> Option<ItemDrop> {
        let drop_chance =
            Self::calculate_drop_chance(passenger, route_type, backstory_unlocked, constants);

        if macroquad_toolkit::rng::rand() > drop_chance {
            return None;
        }

        // Determine item based on passenger
        let item = Self::select_item_for_passenger(passenger, current_time, item_pools, catalog);

        Some(ItemDrop { item })
    }

    /// Which item pool a passenger's generic drops come from.
    ///
    /// `itemCategory` is authored per passenger; the keyword scan over the
    /// `supernatural` prose is only a fallback for entries that omit it.
    fn item_category(passenger: &Passenger) -> &str {
        if let Some(category) = passenger.item_category.as_deref() {
            return category;
        }
        let prose = passenger.supernatural.to_lowercase();
        for (keyword, category) in [
            ("ghost", "ghost"),
            ("specter", "ghost"),
            ("drowned", "ghost"),
            ("vampire", "vampire"),
            ("undead", "vampire"),
            ("demon", "demon"),
            ("reaper", "demon"),
            ("psychic", "occult"),
            ("fortune", "occult"),
            ("nun", "holy"),
            ("priest", "holy"),
        ] {
            if prose.contains(keyword) {
                return category;
            }
        }
        "common"
    }

    /// Select an appropriate item for a passenger to drop
    fn select_item_for_passenger(
        passenger: &Passenger,
        current_time: f64,
        item_pools: &ItemPools,
        catalog: &ItemCatalog,
    ) -> InventoryItem {
        // Check if passenger has specific drop items
        if !passenger.drop_items.is_empty() {
            let idx = macroquad_toolkit::rng::gen_range(0, passenger.drop_items.len());
            let item_name = &passenger.drop_items[idx];
            return catalog.create_item(item_name, &passenger.name, current_time);
        }

        // Otherwise generate based on supernatural type
        let item_name = item_pools.pick(Self::item_category(passenger));
        catalog.create_item(&item_name, &passenger.name, current_time)
    }

    /// Check if a passenger wants to trade
    pub fn check_trade_offer(
        passenger: &Passenger,
        inventory: &[InventoryItem],
        constants: &ConstantsData,
        current_time: f64,
        item_pools: &ItemPools,
        catalog: &ItemCatalog,
    ) -> Option<TradeOffer> {
        // Only some passengers trade
        if !passenger.wants_trade {
            return None;
        }

        // Check if player has something the passenger wants
        let wanted_item = inventory
            .iter()
            .find(|item| passenger.wanted_items.contains(&item.name) && item.can_be_given_away());

        if wanted_item.is_some()
            || macroquad_toolkit::rng::chance(constants.probabilities.trade_offer_chance)
        {
            // Generate a trade offer
            let offered_item =
                Self::select_item_for_passenger(passenger, current_time, item_pools, catalog);

            return Some(TradeOffer {
                passenger_name: passenger.name.clone(),
                offered_item,
            });
        }

        None
    }

    /// Apply effects of an item
    /// `current_time` is passed in rather than read from macroquad's clock.
    /// Reaching for the global made the whole effect table impossible to test
    /// outside a window, which is why the reputation effect could sit broken.
    pub fn apply_item_effect(
        effect: &ItemEffect,
        state: &mut GameState,
        reputation_constants: &ReputationConstants,
        current_time: f64,
    ) {
        match effect.effect_type {
            ItemEffectType::FuelBonus => {
                let bonus = effect.value as f32;
                state.fuel = (state.fuel + bonus).min(state.max_fuel);
            }
            ItemEffectType::TimeBonus => {
                state.time_remaining += effect.value as u32;
            }
            ItemEffectType::RuleImmunity => {
                state.rule_immunity_charges += effect.value as u32;
            }
            ItemEffectType::SupernaturalProtection => {
                state.supernatural_protection += effect.value as u32;
            }
            ItemEffectType::FuelDrain => {
                let drain = effect.value as f32;
                state.fuel = (state.fuel - drain).max(0.0);
            }
            ItemEffectType::TimePenalty => {
                let penalty = effect.value as u32;
                state.time_remaining = state.time_remaining.saturating_sub(penalty);
            }
            ItemEffectType::ReputationModifier => {
                // `get_mut` meant this did nothing at all on a first meeting,
                // which is most of them: a reputation entry is not created
                // until a ride with that passenger completes, so the whole
                // point of the withered flowers and the faded photograph was
                // dropped on the floor the first time you offered either.
                if let Some(passenger_id) = state.current_passenger.as_ref().map(|p| p.id) {
                    state.get_passenger_reputation(passenger_id).adjust(
                        effect.value,
                        current_time,
                        reputation_constants,
                    );
                }
            }
            ItemEffectType::RuleTrigger => {
                state.curse_danger_bonus += effect.value.max(1) as u32;
                state.current_dialogue = Some(CurrentDialogue {
                    text: "The item hums against tonight's rules. The next route will be riskier."
                        .to_string(),
                    speaker: DialogueSpeaker::Narrator,
                    timestamp: current_time,
                });
            }
        }
    }

    /// Use an item from inventory
    /// Returns true if the item was successfully used
    pub fn use_item(
        state: &mut GameState,
        idx: usize,
        reputation_constants: &ReputationConstants,
        current_time: f64,
    ) -> bool {
        if idx >= state.inventory.len() {
            return false;
        }

        // We need to clone the item to use it while mutating state
        let item = state.inventory[idx].clone();

        if !item.can_use {
            return false;
        }

        // Apply item effects
        for effect in &item.effects {
            Self::apply_item_effect(effect, state, reputation_constants, current_time);
        }

        // Handle durability/consumable logic
        if item.item_type == ItemType::Consumable {
            state.inventory.remove(idx);
        } else {
            // Decrease durability for other usable items
            if let Some(stored_item) = state.inventory.get_mut(idx) {
                if let Some(durability) = stored_item.durability {
                    if durability > 0 {
                        stored_item.durability = Some(durability - 1);
                        if durability <= 1 {
                            state.inventory.remove(idx);
                        }
                    }
                }
            }
        }

        true
    }

    /// Updated item states (deterioration, curses)
    pub fn update_items(state: &mut GameState, current_time: f64) {
        // Apply curse penalties
        // Need to clone inventory to allow mutation of state
        let inventory_snapshot = state.inventory.clone();
        Self::apply_curse_penalties(&inventory_snapshot, state, current_time);

        // Apply item deterioration
        for item in &mut state.inventory {
            item.apply_deterioration(current_time);
        }

        // Remove broken items
        state.inventory.retain(|item| !item.is_broken());
    }

    /// Apply curse penalties from all cursed items
    pub fn apply_curse_penalties(
        inventory: &[InventoryItem],
        state: &mut GameState,
        current_time: f64,
    ) {
        for item in inventory {
            if item.is_cursed() && item.should_trigger_curse(current_time) {
                if let Some(ref curse) = item.cursed_properties {
                    match curse.penalty_type {
                        CursePenalty::FuelDrain => {
                            state.fuel = (state.fuel - curse.penalty_value as f32).max(0.0);
                        }
                        CursePenalty::TimeAcceleration => {
                            state.time_remaining = state
                                .time_remaining
                                .saturating_sub(curse.penalty_value as u32);
                        }
                        CursePenalty::AttractingDanger => {
                            // Increases risk level of next route
                            state.curse_danger_bonus += curse.penalty_value as u32;
                        }
                        CursePenalty::ForcedChoices => {
                            state.curse_danger_bonus += curse.penalty_value.max(1) as u32;
                            state.current_dialogue = Some(CurrentDialogue {
                                text: "The cursed item narrows your options. The next route feels more dangerous."
                                    .to_string(),
                                speaker: DialogueSpeaker::Narrator,
                                timestamp: current_time,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{
        load_constants, load_item_catalog, load_item_pools, load_passengers, load_skill_tree,
    };
    use crate::engine::RideService;
    use std::collections::HashSet;

    /// Every authored `itemCategory` must name a real, non-empty pool.
    /// A typo here silently demotes a passenger to the common pool.
    #[test]
    fn every_passenger_category_names_a_stocked_pool() {
        let pools = load_item_pools();
        for passenger in load_passengers() {
            let category = ItemService::item_category(&passenger);
            let stocked = match category {
                "ghost" => &pools.ghost,
                "vampire" => &pools.vampire,
                "demon" => &pools.demon,
                "occult" => &pools.occult,
                "holy" => &pools.holy,
                "common" => &pools.common,
                other => panic!("{} maps to unknown pool {other:?}", passenger.name),
            };
            assert!(
                !stocked.is_empty(),
                "{} draws from empty pool {category:?}",
                passenger.name
            );
        }
    }

    /// Every pool must be reachable from at least one passenger, otherwise the
    /// items authored in it can never drop.
    #[test]
    fn every_pool_is_reachable_from_the_roster() {
        let passengers = load_passengers();
        let reached: HashSet<&str> = passengers.iter().map(ItemService::item_category).collect();
        for pool in ["ghost", "vampire", "demon", "occult", "holy", "common"] {
            assert!(reached.contains(pool), "no passenger draws from {pool:?}");
        }
    }

    /// A reputation item has to work the first time you offer one.
    ///
    /// The effect used `passenger_reputation.get_mut`, and a reputation entry
    /// is not created until a ride with that passenger completes — so on a
    /// first meeting, which is most of them, there was no entry and the whole
    /// point of the item was dropped on the floor without a word.
    #[test]
    fn a_reputation_item_lands_on_a_passenger_met_for_the_first_time() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let passenger = load_passengers().into_iter().next().expect("a roster");

        let mut state = GameState::new(0.0, &constants.game_constants);
        state.current_passenger = Some(passenger.clone());
        assert!(
            state.passenger_reputation.is_empty(),
            "this test is only meaningful before any reputation exists"
        );

        let flowers = catalog.create_item("Withered Flowers", &passenger.name, 0.0);
        let effects: Vec<ItemEffect> = flowers
            .effects
            .iter()
            .filter(|effect| effect.effect_type == ItemEffectType::ReputationModifier)
            .cloned()
            .collect();
        assert!(
            !effects.is_empty(),
            "Withered Flowers no longer carries a reputation effect; pick another item"
        );

        for effect in &effects {
            ItemService::apply_item_effect(effect, &mut state, &constants.reputation, 0.0);
        }

        let reputation = state
            .passenger_reputation
            .get(&passenger.id)
            .expect("offering the flowers recorded nothing");
        assert!(reputation.positive_choices > 0);
        assert_ne!(
            reputation.relationship_level,
            RelationshipLevel::Neutral,
            "standing was tallied but the level the fare reads never moved"
        );
    }

    /// Trade wants are matched against inventory item names, so every wanted
    /// name must exist in the catalog and be tradeable — a want for an
    /// untradeable item can never be satisfied.
    #[test]
    fn wanted_items_exist_and_are_tradeable() {
        let catalog = load_item_catalog();
        for passenger in load_passengers() {
            for name in &passenger.wanted_items {
                assert!(
                    catalog.contains(name),
                    "{} wants unknown item {name:?}",
                    passenger.name
                );
                assert!(
                    catalog.get(name).can_trade,
                    "{} wants untradeable item {name:?}",
                    passenger.name
                );
            }
        }
    }

    /// Someone must want to trade, or the whole trade modal is unreachable.
    #[test]
    fn some_passenger_wants_to_trade() {
        assert!(load_passengers().iter().any(|p| p.wants_trade));
    }

    /// Every `ability_unlock` skill must be earnable-into-use: some passenger
    /// carries the matching trait, or the skill is bank balance thrown away.
    #[test]
    fn every_ability_skill_has_a_passenger_that_uses_it() {
        let passengers = load_passengers();
        let traits: HashSet<String> = passengers
            .iter()
            .flat_map(|p| p.traits.iter())
            .map(|t| RideService::trait_skill_id(t))
            .collect();
        for skill in load_skill_tree() {
            if skill.effect.effect_type != "ability_unlock" {
                continue;
            }
            assert!(
                traits.contains(&skill.effect.target),
                "no passenger has a trait for ability skill {:?}",
                skill.id
            );
        }
    }
}

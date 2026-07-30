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
            // Someone holding what this passenger came for is offered their
            // own work, when they have any to give.
            let reward = wanted_item.and(passenger.trade_reward.as_deref());
            let offered_item = match reward.filter(|name| catalog.contains(name)) {
                Some(name) => catalog.create_item(name, &passenger.name, current_time),
                None => {
                    Self::select_item_for_passenger(passenger, current_time, item_pools, catalog)
                }
            };

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

    /// Whether an effect's authored `condition` holds right now.
    ///
    /// One item uses this -- the Crystal Pendant's rule immunity is authored
    /// `"supernatural_encounter"` -- and nothing read it, so the pendant
    /// granted a plain unconditional charge and the catalogue described
    /// something the game did not do.
    ///
    /// The reading is an interpretation and worth naming as one. A
    /// supernatural encounter is a momentary roll during a leg, not a state
    /// anything could be asked about, so "am I in one" is unanswerable at the
    /// moment an item is used. What is answerable, and what the pendant is
    /// for, is whether the thing in the cab is supernatural at all.
    ///
    /// An unrecognised condition holds. Failing open means a typo leaves an
    /// item working rather than silently inert, which is the direction this
    /// project has been burned in before.
    fn condition_met(effect: &ItemEffect, state: &GameState) -> bool {
        let Some(condition) = effect.condition.as_deref() else {
            return true;
        };
        match condition {
            "supernatural_encounter" => state
                .current_passenger
                .as_ref()
                .is_some_and(|passenger| passenger.is_supernatural),
            _ => true,
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

        // An effect whose condition is not met does not fire, and if that
        // leaves nothing to do the item is not spent at all. Consuming a
        // charge for no result would be a trap, and the player is told why
        // rather than left to wonder.
        let applicable: Vec<&ItemEffect> = item
            .effects
            .iter()
            .filter(|effect| Self::condition_met(effect, state))
            .collect();
        if applicable.is_empty() && !item.effects.is_empty() {
            state.current_dialogue = Some(CurrentDialogue {
                text: format!("The {} stays cold. Nothing here answers to it.", item.name),
                speaker: DialogueSpeaker::Narrator,
                timestamp: current_time,
            });
            return false;
        }

        for effect in applicable {
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
        // Which curses come due this frame. Marked as they are collected, so
        // each one is charged once however long the item is carried afterwards.
        let mut coming_due: Vec<(String, CursedProperties)> = Vec::new();
        for item in &mut state.inventory {
            if item.curse_fired || !item.is_cursed() || !item.should_trigger_curse(current_time) {
                continue;
            }
            if let Some(curse) = item.cursed_properties.clone() {
                item.curse_fired = true;
                coming_due.push((item.name.clone(), curse));
            }
        }
        for (name, curse) in coming_due {
            Self::apply_curse(&name, &curse, state, current_time);
        }

        // Apply item deterioration
        for item in &mut state.inventory {
            item.apply_deterioration(current_time);
        }

        // Remove broken items
        state.inventory.retain(|item| !item.is_broken());
    }

    /// Charge one curse's toll, and say so.
    ///
    /// Three of the four penalties used to take their toll in silence: fuel and
    /// the clock simply dropped, and the danger bonus climbed, with nothing on
    /// screen connecting any of it to the thing in the driver's pocket. Only
    /// `ForcedChoices` spoke. A curse the player cannot attribute is
    /// indistinguishable from the game cheating, and the inventory already names
    /// which item is cursed and how to be rid of it -- this is the moment that
    /// warning comes true.
    fn apply_curse(
        item_name: &str,
        curse: &CursedProperties,
        state: &mut GameState,
        current_time: f64,
    ) {
        let told = match curse.penalty_type {
            CursePenalty::FuelDrain => {
                state.fuel = (state.fuel - curse.penalty_value as f32).max(0.0);
                format!(
                    "The {} has been drinking. {}% of the tank, gone.",
                    item_name, curse.penalty_value
                )
            }
            CursePenalty::TimeAcceleration => {
                state.time_remaining = state
                    .time_remaining
                    .saturating_sub(curse.penalty_value as u32);
                format!(
                    "The {} runs the clock forward. {} minutes you will not get back.",
                    item_name, curse.penalty_value
                )
            }
            CursePenalty::AttractingDanger => {
                state.curse_danger_bonus += curse.penalty_value as u32;
                format!("The {} starts drawing something toward the cab.", item_name)
            }
            CursePenalty::ForcedChoices => {
                state.curse_danger_bonus += curse.penalty_value.max(1) as u32;
                format!(
                    "The {} narrows your options. The roads ahead feel worse.",
                    item_name
                )
            }
        };

        state.current_dialogue = Some(CurrentDialogue {
            text: told,
            speaker: DialogueSpeaker::Narrator,
            timestamp: current_time,
        });
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

    /// Every item in the catalogue has to be obtainable somehow.
    ///
    /// This is the test that found the Soul Protection Ward: legendary, the
    /// only thing that turns away Death's Taxi Driver, and reachable from
    /// nowhere in the game. It sat in the item file and in one unit test while
    /// no pool listed it, no passenger dropped it, and no code created it.
    ///
    /// An unobtainable item is worse than a missing one. It reads as content,
    /// it is balanced against, and it is checked for in `ProtectionService`.
    #[test]
    fn every_item_can_be_obtained() {
        let catalog = load_item_catalog();
        let pools = load_item_pools();
        let passengers = load_passengers();

        let mut reachable: HashSet<String> = HashSet::new();
        for pool in ["ghost", "vampire", "demon", "occult", "holy", "common"] {
            for _ in 0..200 {
                reachable.insert(pools.pick(pool).to_lowercase());
            }
        }
        for passenger in &passengers {
            for name in &passenger.drop_items {
                reachable.insert(name.to_lowercase());
            }
            if let Some(reward) = &passenger.trade_reward {
                reachable.insert(reward.to_lowercase());
            }
        }

        let mut names = catalog.names();
        names.sort();
        let orphans: Vec<String> = names
            .into_iter()
            .filter(|name| !reachable.contains(&name.to_lowercase()))
            .collect();
        assert!(
            orphans.is_empty(),
            "nothing in the game can give the player: {orphans:?}"
        );
    }

    /// Carrying what the crafter wants gets his work offered; carrying
    /// nothing he wants does not.
    ///
    /// This covers the wire rather than the data. The offer is built before the
    /// player picks what to hand over, so "already holding something wanted"
    /// is the only condition available at that moment -- and it has to be the
    /// condition, or the ward is either unreachable again or free.
    #[test]
    fn the_crafter_offers_his_work_to_someone_holding_what_he_wants() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let pools = load_item_pools();
        let collector = load_passengers()
            .into_iter()
            .find(|p| p.trade_reward.is_some())
            .expect("a passenger offering a trade reward");
        let reward = collector.trade_reward.clone().expect("the reward");
        let wanted = collector.wanted_items.first().cloned().expect("a want");

        let offer_with = |inventory: Vec<crate::data::InventoryItem>| {
            ItemService::check_trade_offer(
                &collector, &inventory, &constants, 0.0, &pools, &catalog,
            )
            .map(|offer| offer.offered_item.name)
        };

        let holding = vec![catalog.create_item(&wanted, "test", 0.0)];
        assert_eq!(
            offer_with(holding).as_deref(),
            Some(reward.as_str()),
            "holding what he wants did not get his work offered"
        );

        // Something he has no interest in. Any offer at all here is a roll, so
        // the assertion is only that it is never the reward.
        let unwanted = "Burning Coal";
        assert!(
            !collector.wanted_items.iter().any(|w| w == unwanted),
            "pick an item this passenger does not want"
        );
        for _ in 0..200 {
            let carrying = vec![catalog.create_item(unwanted, "test", 0.0)];
            if let Some(offered) = offer_with(carrying) {
                assert_ne!(
                    offered, reward,
                    "his work was offered to someone carrying nothing he wants"
                );
            }
        }
    }

    /// A `tradeReward` naming something the catalogue does not define would
    /// hand the player a placeholder, since `create_item` substitutes rather
    /// than failing.
    #[test]
    fn every_trade_reward_is_a_real_item() {
        let catalog = load_item_catalog();
        let mut authored = 0;
        for passenger in load_passengers() {
            let Some(reward) = &passenger.trade_reward else {
                continue;
            };
            assert!(
                catalog.contains(reward),
                "{} offers {reward:?}, which is not in the catalogue",
                passenger.name
            );
            assert!(
                !passenger.wanted_items.is_empty(),
                "{} offers a reward for a want they do not have",
                passenger.name
            );
            authored += 1;
        }
        assert!(authored > 0, "no passenger offers a trade reward any more");
    }

    /// A curse fires once, not once a frame.
    ///
    /// `update_items` is called from the frame loop and `should_trigger_curse`
    /// is a threshold rather than an event -- possession time past
    /// `triggersAfter` -- so a held curse applied its penalty on every frame
    /// once the clock passed. A one-point fuel drain becomes sixty a second.
    #[test]
    fn a_curse_bites_once_rather_than_every_frame() {
        let constants = load_constants();
        let catalog = load_item_catalog();

        // The Dusty Mirror drains fuel eighty minutes after it is picked up.
        let mut state = GameState::new(0.0, &constants.game_constants);
        state.fuel = 100.0;
        state
            .inventory
            .push(catalog.create_item("Dusty Mirror", "test", 0.0));
        let curse = state.inventory[0]
            .cursed_properties
            .as_ref()
            .expect("the mirror is cursed")
            .clone();
        assert_eq!(curse.penalty_type, crate::data::CursePenalty::FuelDrain);

        // Well past its trigger, then several frames of the game running.
        let past_trigger = (curse.triggers_after as f64 + 1.0) * 60.0;
        ItemService::update_items(&mut state, past_trigger);
        let after_one = state.fuel;
        for frame in 1..=10 {
            ItemService::update_items(&mut state, past_trigger + frame as f64 * 0.016);
        }

        assert!(after_one < 100.0, "the curse never bit at all");
        assert_eq!(
            state.fuel, after_one,
            "the curse bit again on later frames: {} against {after_one}",
            state.fuel
        );
    }

    /// Every curse names itself when it bites.
    ///
    /// Three of the four penalties took their toll in silence: fuel and the
    /// clock simply dropped and the danger bonus climbed, with nothing on screen
    /// connecting any of it to the thing in the driver's pocket. A loss the
    /// player cannot attribute is indistinguishable from the game cheating.
    #[test]
    fn every_curse_says_which_item_took_the_toll() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let mut names: Vec<String> = catalog.names();
        names.sort();

        let mut checked = 0;
        for name in names {
            let item = catalog.create_item(&name, "test", 0.0);
            let Some(curse) = item.cursed_properties.clone() else {
                continue;
            };

            let mut state = GameState::new(0.0, &constants.game_constants);
            state.fuel = 100.0;
            state.inventory.push(item);
            ItemService::update_items(&mut state, (curse.triggers_after as f64 + 1.0) * 60.0);

            let said = state
                .current_dialogue
                .as_ref()
                .map(|dialogue| dialogue.text.clone())
                .unwrap_or_else(|| panic!("{name} took its toll without a word"));
            assert!(
                said.contains(&name),
                "{name:?} bit and the line did not name it: {said:?}"
            );
            checked += 1;
        }
        assert!(checked > 0, "no cursed items in the catalogue");
    }

    /// An item that counts its uses reports them, and spending one lowers the
    /// count the inventory shows.
    ///
    /// Eleven items author a `maxDurability` and `use_item` spends one per use,
    /// removing the item on its last. Nothing displayed it, so a three-use ward
    /// looked identical to one on its final charge and then vanished without
    /// warning.
    #[test]
    fn a_counted_item_reports_and_spends_its_uses() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let passengers = load_passengers();
        let uncanny = passengers
            .iter()
            .find(|p| p.is_supernatural)
            .expect("a supernatural fare");

        // Prayer Beads: four uses, usable, and no condition on the effect.
        let mut state = GameState::new(0.0, &constants.game_constants);
        state.current_passenger = Some(uncanny.clone());
        state
            .inventory
            .push(catalog.create_item("Prayer Beads", "test", 0.0));

        let (before, most) = state.inventory[0]
            .uses_left()
            .expect("the beads count their uses");
        assert_eq!(before, most, "a fresh item did not start full");
        assert!(
            most > 1,
            "pick an item with more than one use to test spending"
        );

        assert!(ItemService::use_item(
            &mut state,
            0,
            &constants.reputation,
            0.0
        ));
        let (after, _) = state.inventory[0]
            .uses_left()
            .expect("the beads are still in hand");
        assert_eq!(after, before - 1, "spending a use did not lower the count");
    }

    /// An item with no durability authored reports none, rather than "0 of 0".
    #[test]
    fn an_uncounted_item_reports_no_uses() {
        let catalog = load_item_catalog();
        let mut names: Vec<String> = catalog.names();
        names.sort();
        let uncounted = names
            .iter()
            .map(|name| catalog.create_item(name, "test", 0.0))
            .find(|item| item.max_durability.is_none())
            .expect("an item without durability");
        assert!(uncounted.uses_left().is_none());
    }

    /// And nor does an item the driver cannot use, whatever durability it
    /// carries. The cursed items author sixty to a hundred, which is a decay
    /// clock rather than a charge count, and reporting it as uses on a locket
    /// nobody can use is worse than saying nothing.
    #[test]
    fn an_unusable_item_reports_no_uses() {
        let catalog = load_item_catalog();
        let mut names: Vec<String> = catalog.names();
        names.sort();
        let unusable = names
            .iter()
            .map(|name| catalog.create_item(name, "test", 0.0))
            .find(|item| !item.can_use && item.max_durability.is_some())
            .expect("an unusable item that still carries a durability");
        assert!(
            unusable.uses_left().is_none(),
            "{} reported uses it has no way to spend",
            unusable.name
        );
    }

    /// The Crystal Pendant's charge is conditional, and the condition has to
    /// bite. It grants rule immunity authored `"supernatural_encounter"`, so
    /// offering it to an ordinary fare should do nothing -- and should not
    /// spend the item doing nothing either.
    #[test]
    fn a_conditional_charge_waits_for_its_condition() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let passengers = load_passengers();
        let mundane = passengers
            .iter()
            .find(|p| !p.is_supernatural)
            .expect("an ordinary fare");
        let uncanny = passengers
            .iter()
            .find(|p| p.is_supernatural)
            .expect("a supernatural fare");

        let use_on = |passenger: &crate::data::Passenger| {
            let mut state = GameState::new(0.0, &constants.game_constants);
            state.current_passenger = Some(passenger.clone());
            state
                .inventory
                .push(catalog.create_item("Crystal Pendant", "test", 0.0));
            let used = ItemService::use_item(&mut state, 0, &constants.reputation, 0.0);
            (used, state.rule_immunity_charges, state.inventory.len())
        };

        let (used, charges, kept) = use_on(mundane);
        assert!(!used, "the pendant was spent on an ordinary passenger");
        assert_eq!(charges, 0, "immunity was granted with no condition met");
        assert_eq!(kept, 1, "the item was consumed for nothing");

        let (used, charges, _) = use_on(uncanny);
        assert!(used, "the pendant did nothing for a supernatural passenger");
        assert!(
            charges > 0,
            "no immunity was granted when it should have been"
        );
    }

    /// An item with no condition on its effects is unaffected by any of this.
    #[test]
    fn an_unconditional_item_still_works_on_anyone() {
        let constants = load_constants();
        let catalog = load_item_catalog();
        let mundane = load_passengers()
            .into_iter()
            .find(|p| !p.is_supernatural)
            .expect("an ordinary fare");

        let mut state = GameState::new(0.0, &constants.game_constants);
        state.current_passenger = Some(mundane);
        state.fuel = 10.0;
        state
            .inventory
            .push(catalog.create_item("Burning Coal", "test", 0.0));

        assert!(ItemService::use_item(
            &mut state,
            0,
            &constants.reputation,
            0.0
        ));
        assert!(state.fuel > 10.0, "the fuel bonus did not apply");
    }

    /// Every authored condition has to be one `condition_met` recognises.
    /// An unknown one holds, so a typo would leave the item working and the
    /// authored gate silently absent -- visible only here.
    #[test]
    fn every_authored_condition_is_recognised() {
        const KNOWN: [&str; 1] = ["supernatural_encounter"];
        let catalog = load_item_catalog();
        let mut names: Vec<String> = catalog.names();
        names.sort();
        let mut checked = 0;
        for name in names {
            for effect in &catalog.get(&name).effects {
                if let Some(condition) = effect.condition.as_deref() {
                    assert!(
                        KNOWN.contains(&condition),
                        "{name} authors unknown effect condition {condition:?}"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "no item authors an effect condition any more");
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

    /// And the other direction: every trait a passenger carries must have a
    /// skill that buys its use.
    ///
    /// The almanac lists a passenger's traits from Lv.1, so a trait with no
    /// skill behind it is advertised to the player and permanently
    /// unreachable. The existing test only checked that no skill was wasted;
    /// nothing checked that no trait was.
    #[test]
    fn every_passenger_trait_has_a_skill_that_uses_it() {
        let purchasable: HashSet<String> = load_skill_tree()
            .into_iter()
            .filter(|skill| skill.effect.effect_type == "ability_unlock")
            .map(|skill| skill.effect.target)
            .collect();
        for passenger in load_passengers() {
            for trait_name in &passenger.traits {
                let id = RideService::trait_skill_id(trait_name);
                assert!(
                    purchasable.contains(&id),
                    "{} carries trait {trait_name:?}, which no skill unlocks",
                    passenger.name
                );
            }
        }
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

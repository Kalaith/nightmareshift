//! The mid-ride event deck: drawing an event, and resolving the player's choice.

use crate::data::event::{EventChoice, EventConsequence, MidRideEvent, RiskTag};
use crate::data::*;
use crate::engine::PassengerStateMachine;
use crate::state::*;

use super::RideService;

impl RideService {
    /// Generate a mid-ride event by drawing from the authored event deck.
    ///
    /// Picks a route-eligible template (weighted), then optionally appends a
    /// passenger-specific "use your ability" choice when the player has both the
    /// almanac knowledge and the matching skill unlocked, and shuffles.
    /// `pub(crate)` so the screenshot harness can seed a real event rather
    /// than hand-building one that could drift from what the game produces.
    pub(crate) fn generate_mid_ride_event(
        state: &GameState,
        data: &GameData,
        stats: &PlayerStats,
        route: RouteType,
    ) -> MidRideEvent {
        let eligible: Vec<&EventTemplate> = data
            .events
            .iter()
            .filter(|e| e.eligible_for(route))
            .collect();

        let (title, description, mut choices) = match Self::pick_weighted_event(&eligible) {
            Some(tpl) => (
                tpl.title.clone(),
                tpl.description.clone(),
                tpl.choices.clone(),
            ),
            None => Self::fallback_event(),
        };

        // Passenger ability choice: strongest option, gated behind almanac + skill.
        if let Some(p) = &state.current_passenger {
            let almanac_unlocked = stats.get_almanac_entry(p.id).knowledge_level >= 1;
            if almanac_unlocked {
                // Any matching trait qualifies, not just the first one listed —
                // a passenger's second and third traits were unreachable before.
                let matched = p
                    .traits
                    .iter()
                    .find(|trait_name| stats.is_skill_unlocked(&Self::trait_skill_id(trait_name)));
                if let Some(trait_name) = matched {
                    choices.push(EventChoice {
                        description: format!("Use your {} to steady the moment", trait_name),
                        risk_type: RiskTag::SpiritualDisturbance,
                        consequence: EventConsequence::Stress(-10),
                        required_trait: Some(trait_name.clone()),
                    });
                }
            }
        }

        macroquad_toolkit::rng::shuffle(&mut choices);

        MidRideEvent {
            title,
            description,
            choices,
        }
    }

    /// The skill-tree id that grants a passenger trait's ability choice.
    /// `"Night Vision"` and the `night_vision` skill are the same thing.
    pub(crate) fn trait_skill_id(trait_name: &str) -> String {
        trait_name.to_lowercase().replace(' ', "_")
    }

    /// Pick one event weighted by its `weight` field.
    fn pick_weighted_event<'a>(events: &[&'a EventTemplate]) -> Option<&'a EventTemplate> {
        if events.is_empty() {
            return None;
        }
        let total: f32 = events.iter().map(|e| e.weight.max(0.0)).sum();
        if total <= 0.0 {
            return macroquad_toolkit::rng::choose(events).copied();
        }
        let mut roll = macroquad_toolkit::rng::rand() * total;
        for e in events {
            roll -= e.weight.max(0.0);
            if roll <= 0.0 {
                return Some(*e);
            }
        }
        events.last().copied()
    }

    /// Generic event used only if the deck is empty (e.g. data failed to load).
    fn fallback_event() -> (String, String, Vec<EventChoice>) {
        (
            "Unsettling Atmosphere".to_string(),
            "The air grows heavy and silent.".to_string(),
            vec![
                EventChoice {
                    description: "Proceed carefully".to_string(),
                    risk_type: RiskTag::DenseFog,
                    consequence: EventConsequence::Fuel(3.0),
                    required_trait: None,
                },
                EventChoice {
                    description: "Speed through it".to_string(),
                    risk_type: RiskTag::StrangeNoises,
                    consequence: EventConsequence::Risk(2),
                    required_trait: None,
                },
                EventChoice {
                    description: "Take a detour".to_string(),
                    risk_type: RiskTag::RoadConstruction,
                    consequence: EventConsequence::Time(8),
                    required_trait: None,
                },
            ],
        )
    }

    /// Resolve a mid-ride event choice, applying its consequence and returning
    /// to route selection for the second leg of the journey.
    pub fn resolve_event_choice(state: &mut GameState, choice_index: usize) {
        if let Some(event) = &state.current_event {
            if let Some(choice) = event.choices.get(choice_index) {
                match choice.consequence {
                    EventConsequence::Fuel(amount) => {
                        state.fuel = (state.fuel - amount).max(0.0);
                    }
                    EventConsequence::Time(amount) => {
                        state.time_remaining = state.time_remaining.saturating_sub(amount);
                    }
                    EventConsequence::Risk(amount) => {
                        Self::apply_event_stress(state, amount * 8);
                    }
                    EventConsequence::Stress(amount) => {
                        Self::apply_event_stress(state, amount);
                    }
                    EventConsequence::None => {}
                }
            }
        }

        // Return to Driving (Pickup) to present route options for the second leg.
        // transition_driving_phase completes the ride after the second leg.
        state.game_phase = GamePhase::Driving;
        state.driving_phase = Some(DrivingPhase::Pickup);
    }

    /// Apply a stress delta from an event choice to the current passenger's need
    /// state, merging any tells the change triggers.
    fn apply_event_stress(state: &mut GameState, amount: i32) {
        if let (Some(mut need), Some(passenger)) = (
            state.current_passenger_need_state.clone(),
            state.current_passenger.clone(),
        ) {
            let now = macroquad::prelude::get_time();
            let triggered =
                PassengerStateMachine::apply_stress_delta(&mut need, &passenger, amount, now);
            state.current_passenger_need_state = Some(need);
            PassengerStateMachine::merge_detected_tells(
                &mut state.detected_tells,
                triggered,
                &passenger,
                state.player_trust,
                now,
                &state.current_guidelines,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::{load_constants, load_passengers, load_skill_tree};

    /// The ability choice is the one place the almanac and the skill tree pay
    /// off together, and the whole link is a string convention: a passenger's
    /// `"Night Vision"` trait finds the `night_vision` skill by lowercasing and
    /// swapping spaces. Nothing declares that correspondence, so renaming a
    /// trait or a skill id on either side deletes the choice in silence -- no
    /// compile error, no failing test, just a strongest option that stops being
    /// offered. Both directions matter, so both are asserted.
    #[test]
    fn every_passenger_trait_names_a_skill_that_can_be_bought() {
        let skills = load_skill_tree();
        for passenger in load_passengers() {
            for trait_name in &passenger.traits {
                let skill_id = RideService::trait_skill_id(trait_name);
                assert!(
                    skills.iter().any(|skill| skill.id == skill_id),
                    "{} has the trait {trait_name:?}, which needs a {skill_id:?} skill \
                     to ever help anyone, and the tree has no such skill",
                    passenger.name
                );
            }
        }
    }

    /// The other direction: an `ability_unlock` skill is bank spent on a trait
    /// nobody has unless some passenger carries it.
    #[test]
    fn every_ability_skill_is_a_trait_some_passenger_has() {
        let passengers = load_passengers();
        for skill in load_skill_tree()
            .iter()
            .filter(|skill| skill.effect.effect_type == "ability_unlock")
        {
            assert!(
                passengers.iter().any(|passenger| {
                    passenger
                        .traits
                        .iter()
                        .any(|name| RideService::trait_skill_id(name) == skill.id)
                }),
                "the {} skill costs {} and no passenger has the matching trait, \
                 so it can never add an ability choice",
                skill.name,
                skill.cost
            );
        }
    }

    /// And the wire itself: studying a passenger and owning their ability has
    /// to actually put the choice on the screen, and neither half alone may.
    #[test]
    fn the_ability_choice_needs_the_almanac_and_the_skill_together() {
        let data = GameData::load();
        let constants = load_constants();
        let mut state = GameState::new(0.0, &constants.game_constants);

        let passenger = load_passengers()
            .into_iter()
            .find(|p| !p.traits.is_empty())
            .expect("a passenger with a trait");
        let trait_name = passenger.traits[0].clone();
        let skill_id = RideService::trait_skill_id(&trait_name);
        let passenger_id = passenger.id;
        state.current_passenger = Some(passenger);

        let offers_ability = |stats: &PlayerStats| {
            (0..40).any(|_| {
                RideService::generate_mid_ride_event(&state, &data, stats, RouteType::Normal)
                    .choices
                    .iter()
                    .any(|choice| choice.required_trait.as_deref() == Some(trait_name.as_str()))
            })
        };

        let mut neither = PlayerStats::new();
        assert!(
            !offers_ability(&neither),
            "offered with no almanac, no skill"
        );

        let mut skill_only = PlayerStats::new();
        skill_only.unlocked_skills.push(skill_id.clone());
        assert!(
            !offers_ability(&skill_only),
            "the skill alone offered the choice -- studying the passenger is \
             supposed to be the other half of the price"
        );

        neither.almanac_progress.insert(
            passenger_id,
            AlmanacEntry {
                passenger_id,
                encountered: true,
                knowledge_level: 1,
            },
        );
        assert!(
            !offers_ability(&neither),
            "the almanac alone offered the choice"
        );

        let mut both = neither.clone();
        both.unlocked_skills.push(skill_id);
        assert!(
            offers_ability(&both),
            "studied the passenger and bought {trait_name:?}, and the choice \
             never appeared"
        );
    }
}

//! What the almanac tells you about the passenger in front of you.
//!
//! `almanacData.json` promises a concrete reward at each knowledge level —
//! Lv.1 "Basic Needs", Lv.2 "Common Tells", Lv.3 "Hidden Rules"/"True Nature".
//! This module turns a passenger plus a knowledge level into those lines, so
//! the ride-request screen and the almanac screen agree on what a level buys
//! and neither can drift from the authored rewards.

use crate::data::{GameData, NeedType, Passenger, TellIntensity};
use crate::engine::GameEngine;
use crate::state::NeedStage;

/// One revealed fact about a passenger.
pub struct DossierLine {
    /// Short caption, e.g. `"Need"`.
    pub label: String,
    /// The revealed value.
    pub value: String,
    /// Knowledge level that unlocked this line (1-3), used for colouring.
    pub level: u32,
}

impl DossierLine {
    fn new(label: &str, value: String, level: u32) -> Self {
        Self {
            label: label.to_string(),
            value,
            level,
        }
    }
}

/// Human-readable name for a need type.
pub fn need_label(need: NeedType) -> &'static str {
    match need {
        NeedType::Hunger => "Hunger",
        NeedType::Fear => "Fear",
        NeedType::Wrath => "Wrath",
        NeedType::Decay => "Decay",
        NeedType::Loneliness => "Loneliness",
        NeedType::Unknown => "Unreadable",
    }
}

/// Plain description of how much a passenger covers what they are.
fn candour_label(deception: f32) -> &'static str {
    match deception {
        d if d <= 0.05 => "Hides nothing; their tells read true",
        d if d <= 0.25 => "Mostly straight; the odd sign slips past",
        d if d <= 0.45 => "Guarded; expect to miss things",
        d if d <= 0.65 => "Covers well; absence of a tell proves nothing",
        _ => "Practised liar; trust the almanac over your eyes",
    }
}

/// How a need stage reads as a moment to act on, rather than as a label.
fn stage_phrase(stage: NeedStage) -> &'static str {
    match stage {
        NeedStage::Calm => "they are still settled",
        NeedStage::Warning => "they turn restless",
        NeedStage::Critical => "they are close to breaking",
        NeedStage::Meltdown => "it is nearly too late",
    }
}

fn intensity_label(intensity: TellIntensity) -> &'static str {
    match intensity {
        TellIntensity::Subtle => "subtle",
        TellIntensity::Moderate => "moderate",
        TellIntensity::Obvious => "obvious",
    }
}

/// Build the facts a player with `knowledge_level` knows about `passenger`.
///
/// Returns an empty list at level 0 — an unstudied passenger reveals nothing
/// beyond what the ride request already shows.
pub fn build(
    passenger: &Passenger,
    knowledge_level: u32,
    data: Option<&GameData>,
) -> Vec<DossierLine> {
    let mut lines = Vec::new();

    // Lv.1 "Observed" — name, description, and basic needs. The player can
    // already see name and description on the request, so the payoff here is
    // knowing what the passenger needs and where it starts to slip.
    if knowledge_level >= 1 {
        if let Some(profile) = &passenger.state_profile {
            lines.push(DossierLine::new(
                "Need",
                format!(
                    "{} - restless past {}, critical at {}",
                    need_label(profile.need_type),
                    profile.thresholds.warning,
                    profile.thresholds.critical
                ),
                1,
            ));
        }
        if !passenger.traits.is_empty() {
            lines.push(DossierLine::new("Traits", passenger.traits.join(", "), 1));
        }
    }

    // Lv.2 "Studied" — the tells to watch for, and what they are known to
    // carry. Catalogued up front rather than only after one fires this ride.
    if knowledge_level >= 2 {
        for tell in passenger.tells.iter().take(3) {
            lines.push(DossierLine::new(
                "Tell",
                format!("{} ({})", tell.description, intensity_label(tell.intensity)),
                2,
            ));
        }
        if !passenger.items.is_empty() {
            lines.push(DossierLine::new("Carries", passenger.items.join(", "), 2));
        }
        // Who they are connected to. The same `relationships` list makes an
        // associate more likely to turn up later in the shift, so knowing it
        // is knowing who the night is about to bring round.
        if let Some(data) = data {
            let names: Vec<&str> = passenger
                .relationships
                .iter()
                .filter_map(|id| data.passengers.iter().find(|p| p.id == *id))
                .map(|p| p.name.as_str())
                .collect();
            if !names.is_empty() {
                lines.push(DossierLine::new("Associates", names.join(", "), 2));
            }
        }
    }

    // Lv.3 "Mastered" — the passenger's own rule, and what they truly are.
    if knowledge_level >= 3 {
        if !passenger.personal_rule.is_empty() {
            lines.push(DossierLine::new(
                "Their rule",
                passenger.personal_rule.clone(),
                3,
            ));
        }
        if !passenger.supernatural.is_empty() {
            lines.push(DossierLine::new(
                "True nature",
                passenger.supernatural.clone(),
                3,
            ));
        }
        // What actually settles them, and when.
        //
        // The engine grants a passenger's exception only for the guideline
        // that owns their `exceptionId`, and only once they have reached the
        // stage that exception authors. Both facts decided whether a ride
        // survived and neither was reachable from anywhere in the game — a
        // player could study a passenger to Mastered and still be guessing
        // which forbidden action would calm them. This is the line the
        // almanac exists to sell.
        if let (Some(data), Some(profile)) = (data, passenger.state_profile.as_ref()) {
            if let Some(exception_id) = profile.exception_id.as_deref() {
                if let Some((guideline, exception)) = data
                    .guidelines
                    .iter()
                    .flat_map(|guideline| {
                        guideline
                            .exceptions
                            .iter()
                            .map(move |exception| (guideline, exception))
                    })
                    .find(|(_, exception)| exception.id == exception_id)
                {
                    lines.push(DossierLine::new(
                        "Relief",
                        format!(
                            "break \"{}\" once {}",
                            guideline.title,
                            stage_phrase(GameEngine::required_stage(exception))
                        ),
                        3,
                    ));
                }
            }
        }
        // How much to trust what you see them do. `deceptionLevel` now scales
        // tell detection, so knowing it is knowing whether an absent tell
        // means calm or means well hidden.
        lines.push(DossierLine::new(
            "Candour",
            candour_label(passenger.deception_level).to_string(),
            3,
        ));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_passengers;

    /// Each level must add something for every passenger, or paying lore
    /// fragments to reach it buys nothing.
    #[test]
    fn every_level_reveals_more_for_every_passenger() {
        let data = GameData::load();
        for passenger in load_passengers() {
            let mut previous = build(&passenger, 0, Some(&data)).len();
            for level in 1..=3 {
                let count = build(&passenger, level, Some(&data)).len();
                assert!(
                    count > previous,
                    "{} reveals nothing new at knowledge level {level}",
                    passenger.name
                );
                previous = count;
            }
        }
    }

    /// What a level promises is what that level delivers.
    ///
    /// `almanacData.json` authors a reward list per level, and since the almanac
    /// card prints it -- "Studied reveals Route Preferences, Common Tells..." --
    /// the list is now a promise made to the player in the UI. It was wrong in
    /// both directions before this test existed. Level 1 claimed "Name" and
    /// "Description", both of which an encountered passenger shows for free at
    /// level 0, and omitted Traits, which it does buy. Level 3 claimed "Hidden
    /// Rules", which no almanac level touches -- hidden rules belong to the
    /// shift, not to a passenger, so a per-passenger mastery could not reveal
    /// them even in principle -- and omitted Relief and Candour.
    ///
    /// The mapping is spelled out rather than inferred, because the two sides
    /// are deliberately worded differently: the player reads "Basic Needs", the
    /// dossier labels the line "Need".
    #[test]
    fn every_promised_reward_is_delivered_at_that_level() {
        use crate::data::loader::load_almanac;

        // Reward name -> the dossier label that carries it.
        const DELIVERED_BY: [(&str, &str); 9] = [
            ("Basic Needs", "Need"),
            ("Traits", "Traits"),
            ("Common Tells", "Tell"),
            ("Carried Items", "Carries"),
            ("Associates", "Associates"),
            ("Their Rule", "Their rule"),
            ("True Nature", "True nature"),
            ("Relief", "Relief"),
            ("Candour", "Candour"),
        ];
        // Delivered somewhere other than the dossier: route preferences appear
        // on the almanac card and the driving screen's route cards, and the
        // backstory on the almanac card.
        const ELSEWHERE: [&str; 2] = ["Route Preferences", "Backstory"];

        let data = GameData::load();
        let passengers = load_passengers();
        let almanac = load_almanac();

        for level in 1..=3u32 {
            let entry = almanac.get_level(level).expect("a level");
            for reward in &entry.rewards {
                if ELSEWHERE.contains(&reward.as_str()) {
                    continue;
                }
                let label = DELIVERED_BY
                    .iter()
                    .find(|(name, _)| name == reward)
                    .map(|(_, label)| *label)
                    .unwrap_or_else(|| {
                        panic!("level {level} promises {reward:?}, which nothing delivers")
                    });

                // Some lines need underlying data the passenger may not have,
                // so it is enough that one passenger gains this label exactly
                // here -- and none gains it a level earlier.
                let gained_here = passengers.iter().any(|passenger| {
                    let has = |at| {
                        build(passenger, at, Some(&data))
                            .iter()
                            .any(|line| line.label == label)
                    };
                    has(level) && !has(level - 1)
                });
                assert!(
                    gained_here,
                    "level {level} promises {reward:?} but no passenger gains {label:?} there"
                );
            }
        }
    }

    /// And nothing a level delivers goes unmentioned, or the card understates
    /// what the lore buys.
    #[test]
    fn every_delivered_line_is_promised() {
        use crate::data::loader::load_almanac;
        use std::collections::HashSet;

        // The dossier labels, as the reward lists spell them.
        const PROMISED_AS: [(&str, &str); 9] = [
            ("Need", "Basic Needs"),
            ("Traits", "Traits"),
            ("Tell", "Common Tells"),
            ("Carries", "Carried Items"),
            ("Associates", "Associates"),
            ("Their rule", "Their Rule"),
            ("True nature", "True Nature"),
            ("Relief", "Relief"),
            ("Candour", "Candour"),
        ];

        let data = GameData::load();
        let almanac = load_almanac();

        for level in 1..=3u32 {
            let promised: HashSet<&str> = almanac
                .get_level(level)
                .expect("a level")
                .rewards
                .iter()
                .map(String::as_str)
                .collect();

            for passenger in load_passengers() {
                let before: HashSet<String> = build(&passenger, level - 1, Some(&data))
                    .into_iter()
                    .map(|line| line.label)
                    .collect();
                for line in build(&passenger, level, Some(&data)) {
                    if before.contains(&line.label) {
                        continue;
                    }
                    let name = PROMISED_AS
                        .iter()
                        .find(|(label, _)| *label == line.label)
                        .map(|(_, name)| *name)
                        .unwrap_or_else(|| {
                            panic!("dossier line {:?} maps to no reward name", line.label)
                        });
                    assert!(
                        promised.contains(name),
                        "level {level} delivers {:?} to {} and never says so",
                        line.label,
                        passenger.name
                    );
                }
            }
        }
    }

    /// Every passenger with a need profile must have a way to be settled,
    /// and Mastered must name it.
    ///
    /// A profile authors an `exceptionId`; the engine grants relief only for
    /// the guideline owning that id. An id that matches no guideline is a
    /// passenger who cannot be soothed at all — and until this line existed
    /// there was nowhere in the game that would have shown it.
    #[test]
    fn mastering_a_passenger_names_what_settles_them() {
        let data = GameData::load();
        let mut named = 0;
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let Some(exception_id) = profile.exception_id.as_deref() else {
                continue;
            };
            assert!(
                data.guidelines.iter().any(|guideline| guideline
                    .exceptions
                    .iter()
                    .any(|exception| exception.id == exception_id)),
                "{} needs exception {exception_id:?}, which no guideline owns",
                passenger.name
            );

            let lines = build(&passenger, 3, Some(&data));
            assert!(
                lines.iter().any(|line| line.label == "Relief"),
                "{} can be mastered without learning what settles them",
                passenger.name
            );
            named += 1;
        }
        assert!(named > 0, "no passenger authors an exception any more");
    }

    /// The stage the almanac quotes must be the stage the engine gates on.
    ///
    /// Both now come from `GameEngine::required_stage`, but the point of the
    /// test is that they must keep coming from the same place: an almanac
    /// that promises relief at a stage the engine will not grant it at is
    /// worse than an almanac that stays quiet.
    #[test]
    fn the_quoted_relief_stage_is_the_one_the_engine_gates_on() {
        let data = GameData::load();
        for passenger in load_passengers() {
            let Some(exception_id) = passenger
                .state_profile
                .as_ref()
                .and_then(|profile| profile.exception_id.as_deref())
            else {
                continue;
            };
            let Some(exception) = data
                .guidelines
                .iter()
                .flat_map(|guideline| guideline.exceptions.iter())
                .find(|exception| exception.id == exception_id)
            else {
                continue;
            };

            let expected = stage_phrase(GameEngine::required_stage(exception));
            let relief = build(&passenger, 3, Some(&data))
                .into_iter()
                .find(|line| line.label == "Relief")
                .expect("a relief line");
            assert!(
                relief.value.ends_with(expected),
                "{} is told {:?}, but the engine unlocks at {expected:?}",
                passenger.name,
                relief.value
            );
        }
    }

    /// The numbers the dossier prints must be the numbers the simulation
    /// runs on.
    ///
    /// Lv.1 tells the player a passenger turns restless past one level and
    /// critical at another. Those readings come from `stateProfile`, and
    /// `PassengerNeedState::calculate_stage` decides the actual stage from
    /// the same block — but nothing checked they agreed, and an almanac that
    /// quotes a threshold the state machine does not use is worse than one
    /// that says nothing.
    #[test]
    fn the_quoted_thresholds_are_the_ones_the_state_machine_uses() {
        use crate::state::{NeedStage, PassengerNeedState};

        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let thresholds = &profile.thresholds;

            let line = build(&passenger, 1, None)
                .into_iter()
                .find(|l| l.label == "Need")
                .unwrap_or_else(|| panic!("{} shows no need line at Lv.1", passenger.name));

            // The two figures the line quotes.
            assert!(
                line.value.contains(&thresholds.warning.to_string()),
                "{}: line {:?} does not quote the warning threshold {}",
                passenger.name,
                line.value,
                thresholds.warning
            );
            assert!(
                line.value.contains(&thresholds.critical.to_string()),
                "{}: line {:?} does not quote the critical threshold {}",
                passenger.name,
                line.value,
                thresholds.critical
            );

            // And the state machine must actually turn at them.
            assert_eq!(
                PassengerNeedState::calculate_stage(thresholds.warning, thresholds),
                NeedStage::Warning,
                "{} does not reach Warning at its quoted threshold",
                passenger.name
            );
            assert_eq!(
                PassengerNeedState::calculate_stage(thresholds.warning - 1, thresholds),
                NeedStage::Calm,
                "{} is already restless below its quoted threshold",
                passenger.name
            );
            assert_eq!(
                PassengerNeedState::calculate_stage(thresholds.critical, thresholds),
                NeedStage::Critical,
                "{} does not reach Critical at its quoted threshold",
                passenger.name
            );
        }
    }

    /// The need type named at Lv.1 must be the one the passenger actually
    /// carries, not a label that drifted from the profile.
    #[test]
    fn the_quoted_need_is_the_carried_need() {
        for passenger in load_passengers() {
            let Some(profile) = &passenger.state_profile else {
                continue;
            };
            let line = build(&passenger, 1, None)
                .into_iter()
                .find(|l| l.label == "Need")
                .expect("a need line");
            assert!(
                line.value.starts_with(need_label(profile.need_type)),
                "{}: line {:?} does not name {:?}",
                passenger.name,
                line.value,
                profile.need_type
            );
        }
    }

    /// Every tell the dossier lists at Lv.2 must be one the passenger can
    /// actually show, and every tell they can show at warning or critical
    /// must be listed. A dossier that names a tell the state machine never
    /// surfaces teaches the player to watch for nothing.
    #[test]
    fn the_listed_tells_are_the_ones_that_can_surface() {
        for passenger in load_passengers() {
            let listed: Vec<String> = build(&passenger, 2, None)
                .into_iter()
                .filter(|l| l.label == "Tell")
                .map(|l| l.value)
                .collect();

            for tell in passenger.tells.iter().take(3) {
                assert!(
                    listed.iter().any(|l| l.contains(&tell.description)),
                    "{} can show {:?} and the dossier does not list it",
                    passenger.name,
                    tell.description
                );
            }
            for entry in &listed {
                assert!(
                    passenger
                        .tells
                        .iter()
                        .any(|tell| entry.contains(&tell.description)),
                    "{} lists {entry:?}, which is not one of their tells",
                    passenger.name
                );
            }
        }
    }

    /// Level 0 is the unstudied state and must stay silent.
    #[test]
    fn unstudied_passengers_reveal_nothing() {
        for passenger in load_passengers() {
            assert!(build(&passenger, 0, None).is_empty(), "{}", passenger.name);
        }
    }

    /// The Lv.3 "Hidden Rules" reward is the passenger's `personalRule`, so
    /// every passenger needs one authored.
    #[test]
    fn every_passenger_has_a_personal_rule() {
        for passenger in load_passengers() {
            assert!(
                !passenger.personal_rule.trim().is_empty(),
                "{} has no personalRule",
                passenger.name
            );
        }
    }

    /// The Lv.1 "Basic Needs" reward reads the state profile, so every
    /// passenger needs one.
    #[test]
    fn every_passenger_has_a_state_profile() {
        for passenger in load_passengers() {
            assert!(
                passenger.state_profile.is_some(),
                "{} has no stateProfile",
                passenger.name
            );
        }
    }
}

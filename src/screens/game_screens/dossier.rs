//! What the almanac tells you about the passenger in front of you.
//!
//! `almanacData.json` promises a concrete reward at each knowledge level —
//! Lv.1 "Basic Needs", Lv.2 "Common Tells", Lv.3 "Hidden Rules"/"True Nature".
//! This module turns a passenger plus a knowledge level into those lines, so
//! the ride-request screen and the almanac screen agree on what a level buys
//! and neither can drift from the authored rewards.

use crate::data::{GameData, NeedType, Passenger, TellIntensity};

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
                    "{} — restless past {}, critical at {}",
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

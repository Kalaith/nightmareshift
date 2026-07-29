//! Location data matching locationData.json.

use serde::{Deserialize, Serialize};

fn default_fare_modifier() -> f32 {
    1.0
}

/// A location in the city where passengers can be picked up or dropped off
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub description: String,
    pub atmosphere: String,
    /// Route-risk contribution when this location is a passenger's pickup.
    #[serde(rename = "riskLevel")]
    pub risk_level: u32,
    /// Fare multiplier applied when this location is a passenger's destination.
    /// Remote or dangerous drop-offs pay a premium; safe civic ones pay less.
    #[serde(rename = "fareModifier", default = "default_fare_modifier")]
    pub fare_modifier: f32,
}

#[cfg(test)]
mod tests {
    use crate::data::loader::{load_constants, load_locations, load_passengers};

    /// Every location must describe itself. The atmosphere is shown on the
    /// ride request next to the risk it explains, so a blank one leaves the
    /// player a bare risk number with no reason attached.
    #[test]
    fn every_location_has_an_atmosphere() {
        for location in load_locations() {
            assert!(
                !location.atmosphere.trim().is_empty(),
                "{} has no atmosphere",
                location.name
            );
        }
    }

    /// Risk levels must sit inside the range the risk constants describe, or
    /// the colouring on the ride request means nothing.
    #[test]
    fn location_risk_sits_within_the_authored_scale() {
        let risk = load_constants().risk;
        for location in load_locations() {
            assert!(
                location.risk_level >= risk.safe,
                "{} is below the safe floor",
                location.name
            );
            assert!(
                location.risk_level <= risk.max_risk_level + 1,
                "{} at risk {} exceeds the authored scale",
                location.name,
                location.risk_level
            );
        }
    }

    /// Every pickup and destination a passenger names must be a real
    /// location, or the ride request shows a route to nowhere and the
    /// pickup's risk silently defaults.
    #[test]
    fn every_passenger_route_names_real_locations() {
        let locations = load_locations();
        let known: Vec<&str> = locations.iter().map(|l| l.name.as_str()).collect();
        for passenger in load_passengers() {
            assert!(
                known.contains(&passenger.pickup.as_str()),
                "{} is picked up at unknown location {:?}",
                passenger.name,
                passenger.pickup
            );
            assert!(
                known.contains(&passenger.destination.as_str()),
                "{} is bound for unknown location {:?}",
                passenger.name,
                passenger.destination
            );
        }
    }
}

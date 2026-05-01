use serde::{Deserialize, Serialize};

/// Tags representing potential risks on a route
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskTag {
    HighTraffic,
    PolicePatrol,
    SpiritualDisturbance,
    SlipperyRoads,
    RoadConstruction,
    DenseFog,
    GangActivity,
    Potholes,
    FlashFloods,
    StrangeNoises,
}

impl RiskTag {
    /// Get a user-friendly name for the risk
    pub fn name(&self) -> &'static str {
        match self {
            RiskTag::HighTraffic => "High Traffic",
            RiskTag::PolicePatrol => "Police Patrol",
            RiskTag::SpiritualDisturbance => "Spiritual Disturbance",
            RiskTag::SlipperyRoads => "Slippery Roads",
            RiskTag::RoadConstruction => "Road Construction",
            RiskTag::DenseFog => "Dense Fog",
            RiskTag::GangActivity => "Gang Activity",
            RiskTag::Potholes => "Severe Potholes",
            RiskTag::FlashFloods => "Flash Floods",
            RiskTag::StrangeNoises => "Strange Noises",
        }
    }

    /// Get a description for the risk
    pub fn description(&self) -> &'static str {
        match self {
            RiskTag::HighTraffic => "Delays likely.",
            RiskTag::PolicePatrol => "Watch your speed.",
            RiskTag::SpiritualDisturbance => "Entities active.",
            RiskTag::SlipperyRoads => "Hard to control.",
            RiskTag::RoadConstruction => "Detours ahead.",
            RiskTag::DenseFog => "Low visibility.",
            RiskTag::GangActivity => "Avoid stopping.",
            RiskTag::Potholes => "Suspension damage.",
            RiskTag::FlashFloods => "Hydroplane risk.",
            RiskTag::StrangeNoises => "Unsettling sounds.",
        }
    }
}

/// A consequence for an event choice (simplified compared to rules::Consequence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventConsequence {
    Fuel(f32),
    Time(u32),
    Risk(i32),
    Stress(i32),
    None,
}

/// A choice within a mid-ride event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChoice {
    pub description: String,
    pub risk_type: RiskTag,
    pub consequence: EventConsequence,
    pub required_trait: Option<String>,
}

/// A mid-ride event that occurs during travel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidRideEvent {
    pub title: String,
    pub description: String,
    pub choices: Vec<EventChoice>,
}

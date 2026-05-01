//! Location data matching locationData.json.

use serde::{Deserialize, Serialize};

/// A location in the city where passengers can be picked up or dropped off
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub description: String,
    pub atmosphere: String,
    #[serde(rename = "riskLevel")]
    pub risk_level: u32,
}

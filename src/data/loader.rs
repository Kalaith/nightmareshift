//! Data loading from embedded JSON files.

use super::*;

/// All game data loaded from JSON files
pub struct GameData {
    pub passengers: Vec<Passenger>,
    pub rules: Vec<Rule>,
    pub locations: Vec<Location>,
    pub constants: ConstantsData,
    pub skills: Vec<Skill>,
    pub almanac: AlmanacData,
    pub guidelines: Vec<Guideline>,
}

impl GameData {
    /// Load all game data from embedded JSON files
    pub fn load() -> Self {
        Self {
            passengers: load_passengers(),
            rules: load_rules(),
            locations: load_locations(),
            constants: load_constants(),
            skills: load_skill_tree(),
            almanac: load_almanac(),
            guidelines: load_guidelines(),
        }
    }

    /// Find a location by name
    pub fn get_location(&self, name: &str) -> Option<&Location> {
        self.locations.iter().find(|l| l.name == name)
    }
}

/// Load passengers from embedded JSON
pub fn load_passengers() -> Vec<Passenger> {
    let json = include_str!("../../assets/passengerData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse passengers: {}", e);
        Vec::new()
    })
}

/// Load rules from embedded JSON
pub fn load_rules() -> Vec<Rule> {
    let json = include_str!("../../assets/shiftRulesData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse rules: {}", e);
        Vec::new()
    })
}

/// Load locations from embedded JSON
pub fn load_locations() -> Vec<Location> {
    let json = include_str!("../../assets/locationData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse locations: {}", e);
        Vec::new()
    })
}

/// Load constants from embedded JSON
pub fn load_constants() -> ConstantsData {
    let json = include_str!("../../assets/constants.json");
    serde_json::from_str(json).expect("Failed to parse constants.json - this is a development error")
}

/// Load skill tree from embedded JSON (JSON is an array directly)
pub fn load_skill_tree() -> Vec<Skill> {
    let json = include_str!("../../assets/skillTreeData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse skill tree: {}", e);
        Vec::new()
    })
}

/// Load almanac data from embedded JSON
pub fn load_almanac() -> AlmanacData {
    let json = include_str!("../../assets/almanacData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse almanac: {}", e);
        AlmanacData {
            levels: std::collections::HashMap::new(),
            lore_costs: LoreCosts { level_1: 1, level_2: 3, level_3: 5 },
        }
    })
}

/// Load guidelines from embedded JSON
pub fn load_guidelines() -> Vec<Guideline> {
    let json = include_str!("../../assets/guidelineData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse guidelines: {}", e);
        Vec::new()
    })
}

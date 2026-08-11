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
    pub localization: Localization,
    pub events: Vec<EventTemplate>,
    pub item_pools: ItemPools,
    pub items: ItemCatalog,
    pub rewards: RewardData,
    pub night_modifiers: NightModifierData,
    pub epilogues: Vec<Epilogue>,
    /// Content files that failed to parse and fell back empty. The night
    /// can still run without them — thinner — and the menu says so, since
    /// a stderr line reaches nobody on the web build.
    pub load_errors: Vec<String>,
}

impl GameData {
    /// Load all game data from embedded JSON files.
    ///
    /// Errs only when the game cannot run at all: `constants.json` and the
    /// localization file are structural, and there is nothing sensible to
    /// fall back on. Content files that fail fall back empty and are
    /// reported in `load_errors` instead — a missing roster is a broken
    /// night, not a broken program.
    pub fn load() -> Result<Self, String> {
        let mut data = Self {
            passengers: load_passengers(),
            rules: load_rules(),
            locations: load_locations(),
            constants: try_load_constants()?,
            skills: load_skill_tree(),
            almanac: load_almanac(),
            guidelines: load_guidelines(),
            localization: try_load_localization()?,
            events: load_events(),
            item_pools: load_item_pools(),
            items: load_item_catalog(),
            rewards: load_rewards(),
            night_modifiers: load_night_modifiers(),
            epilogues: load_epilogues(),
            load_errors: Vec::new(),
        };
        for (file, empty) in [
            ("passengerData.json", data.passengers.is_empty()),
            ("shiftRulesData.json", data.rules.is_empty()),
            ("locationData.json", data.locations.is_empty()),
            ("skillTreeData.json", data.skills.is_empty()),
            ("guidelineData.json", data.guidelines.is_empty()),
            ("eventData.json", data.events.is_empty()),
        ] {
            if empty {
                data.load_errors
                    .push(format!("{file} failed to load; its content is missing"));
            }
        }
        Ok(data)
    }

    /// Find a location by name
    pub fn get_location(&self, name: &str) -> Option<&Location> {
        self.locations.iter().find(|l| l.name == name)
    }
}

/// Load passengers from embedded JSON
pub fn load_passengers() -> Vec<Passenger> {
    let json = macroquad_toolkit::include_json_str!("../../assets/passengerData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse passengers: {}", e);
        Vec::new()
    })
}

/// Load rules from embedded JSON
pub fn load_rules() -> Vec<Rule> {
    let json = macroquad_toolkit::include_json_str!("../../assets/shiftRulesData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse rules: {}", e);
        Vec::new()
    })
}

/// Load locations from embedded JSON
pub fn load_locations() -> Vec<Location> {
    let json = macroquad_toolkit::include_json_str!("../../assets/locationData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse locations: {}", e);
        Vec::new()
    })
}

/// Load constants, or say exactly which part of the file is wrong.
pub fn try_load_constants() -> Result<ConstantsData, String> {
    let json = macroquad_toolkit::include_json_str!("../../assets/constants.json");
    serde_json::from_str(json).map_err(|e| format!("constants.json: {e}"))
}

/// Load constants from embedded JSON.
///
/// Panics on a parse failure, which is the right behavior for the tests
/// that call it — the production path is `GameData::load`, which carries
/// the error to a screen instead.
#[cfg(test)]
pub fn load_constants() -> ConstantsData {
    try_load_constants().expect("constants.json parses")
}

/// Load skill tree from embedded JSON (JSON is an array directly)
pub fn load_skill_tree() -> Vec<Skill> {
    let json = macroquad_toolkit::include_json_str!("../../assets/skillTreeData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse skill tree: {}", e);
        Vec::new()
    })
}

/// Load the ending epilogues from embedded JSON. An unparseable file falls
/// back empty — endings show their title and subtitle, just no paragraph.
pub fn load_epilogues() -> Vec<Epilogue> {
    let json = macroquad_toolkit::include_json_str!("../../assets/epilogueData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse epilogues: {}", e);
        Vec::new()
    })
}

/// Load the night-modifier deck from embedded JSON. An unparseable file
/// falls back to an empty deck — no modifier ever rolls, the campaign
/// still runs.
pub fn load_night_modifiers() -> NightModifierData {
    let json = macroquad_toolkit::include_json_str!("../../assets/nightModifierData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse night modifiers: {}", e);
        NightModifierData {
            chance: 0.0,
            modifiers: Vec::new(),
        }
    })
}

/// Load almanac data from embedded JSON
pub fn load_almanac() -> AlmanacData {
    let json = macroquad_toolkit::include_json_str!("../../assets/almanacData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse almanac: {}", e);
        AlmanacData {
            levels: std::collections::HashMap::new(),
            lore_costs: LoreCosts {
                level_1: 1,
                level_2: 3,
                level_3: 5,
            },
        }
    })
}

/// Load guidelines from embedded JSON
pub fn load_guidelines() -> Vec<Guideline> {
    let json = macroquad_toolkit::include_json_str!("../../assets/guidelineData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse guidelines: {}", e);
        Vec::new()
    })
}

/// Load the mid-ride event deck from embedded JSON
pub fn load_events() -> Vec<EventTemplate> {
    let json = macroquad_toolkit::include_json_str!("../../assets/eventData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse events: {}", e);
        Vec::new()
    })
}

/// Load item name pools from embedded JSON
pub fn load_item_pools() -> ItemPools {
    let json = macroquad_toolkit::include_json_str!("../../assets/itemPoolData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse item pools: {}", e);
        ItemPools::default()
    })
}

/// Load the item catalog from embedded JSON. Every name any pool or passenger
/// can drop is defined here, so a dropped item always carries real effects
/// rather than being an inert keepsake.
pub fn load_item_catalog() -> ItemCatalog {
    let json = macroquad_toolkit::include_json_str!("../../assets/itemData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse item catalog: {}", e);
        ItemCatalog::default()
    })
}

/// Load meta-progression payouts from embedded JSON.
pub fn load_rewards() -> RewardData {
    let json = macroquad_toolkit::include_json_str!("../../assets/rewardData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse rewards: {}", e);
        RewardData::default()
    })
}

/// Load localization from embedded JSON, with glyphs the bundled font cannot
/// draw removed.
///
/// The UI strings are authored with emoji — a fuel pump on the fuel readout,
/// a shield on the survival skills, a skull on the game-over title. The font
/// has none of them, so they rendered as replacement boxes on the status bar,
/// the waiting screen, the skill tree, the trade offer and the ride summary.
/// One private helper in `ui::components` stripped them for the four status
/// bar fields and nothing else did, so the fix depended on each caller
/// remembering. Cleaning them out of the text once, here, means no caller can
/// forget — and the emoji stay in the JSON for a font that can draw them.
pub fn try_load_localization() -> Result<Localization, String> {
    let json = macroquad_toolkit::include_json_str!("../../assets/localization/en.json");
    let mut value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("localization/en.json: {e}"))?;
    strip_undrawable_glyphs(&mut value);
    serde_json::from_value(value).map_err(|e| format!("localization/en.json shape: {e}"))
}

/// Panicking wrapper for the tests; production goes through
/// `GameData::load`, which shows the error instead.
#[cfg(test)]
pub fn load_localization() -> Localization {
    try_load_localization().expect("localization/en.json parses")
}

/// Remove characters the bundled font cannot draw from every string.
///
/// The rule is plain ASCII. A first attempt cut at U+2500 on the reasoning
/// that Latin text and punctuation sit below it — and left the status bar
/// clock still showing a box, because the alarm clock is U+23F0 and the
/// stopwatch U+23F1. Every non-ASCII code point in `en.json` is a pictograph
/// (see the test below, which fails if that stops being true), so there is no
/// accented prose to protect and no reason for a subtler boundary.
///
/// Only strings that actually lost a character are trimmed, so the gap a
/// stripped prefix leaves does not show as a stray indent while deliberate
/// spacing — the leaderboard detail line is indented on purpose — survives.
fn strip_undrawable_glyphs(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => {
            let cleaned: String = text.chars().filter(char::is_ascii).collect();
            if cleaned.len() != text.len() {
                *text = cleaned.trim().to_string();
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_undrawable_glyphs),
        serde_json::Value::Object(map) => {
            map.values_mut().for_each(strip_undrawable_glyphs);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;

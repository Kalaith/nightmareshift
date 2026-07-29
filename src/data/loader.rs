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
            localization: load_localization(),
            events: load_events(),
            item_pools: load_item_pools(),
            items: load_item_catalog(),
            rewards: load_rewards(),
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
    serde_json::from_str(json)
        .expect("Failed to parse constants.json - this is a development error")
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
    let json = include_str!("../../assets/guidelineData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse guidelines: {}", e);
        Vec::new()
    })
}

/// Load the mid-ride event deck from embedded JSON
pub fn load_events() -> Vec<EventTemplate> {
    let json = include_str!("../../assets/eventData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse events: {}", e);
        Vec::new()
    })
}

/// Load item name pools from embedded JSON
pub fn load_item_pools() -> ItemPools {
    let json = include_str!("../../assets/itemPoolData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse item pools: {}", e);
        ItemPools::default()
    })
}

/// Load the item catalog from embedded JSON. Every name any pool or passenger
/// can drop is defined here, so a dropped item always carries real effects
/// rather than being an inert keepsake.
pub fn load_item_catalog() -> ItemCatalog {
    let json = include_str!("../../assets/itemData.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse item catalog: {}", e);
        ItemCatalog::default()
    })
}

/// Load meta-progression payouts from embedded JSON.
pub fn load_rewards() -> RewardData {
    let json = include_str!("../../assets/rewardData.json");
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
pub fn load_localization() -> Localization {
    let json = include_str!("../../assets/localization/en.json");
    let mut value: serde_json::Value = serde_json::from_str(json)
        .expect("Failed to parse localization/en.json - this is critical");
    strip_undrawable_glyphs(&mut value);
    serde_json::from_value(value).expect("localization/en.json does not match Localization")
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
mod tests {
    use super::*;

    /// No localized string may reach the UI carrying a glyph the font cannot
    /// draw. Each one renders as a replacement box, which is what the fuel
    /// readout, the skill tree categories and the trade title were showing.
    #[test]
    fn localization_carries_no_undrawable_glyphs() {
        let json = include_str!("../../assets/localization/en.json");
        let mut value: serde_json::Value = serde_json::from_str(json).expect("valid json");
        strip_undrawable_glyphs(&mut value);

        fn walk(value: &serde_json::Value, path: &str) {
            match value {
                serde_json::Value::String(text) => {
                    let bad: Vec<char> = text.chars().filter(|ch| !ch.is_ascii()).collect();
                    assert!(bad.is_empty(), "{path} still carries {bad:?}");
                }
                serde_json::Value::Array(items) => {
                    for (i, item) in items.iter().enumerate() {
                        walk(item, &format!("{path}[{i}]"));
                    }
                }
                serde_json::Value::Object(map) => {
                    for (key, item) in map {
                        walk(item, &format!("{path}.{key}"));
                    }
                }
                _ => {}
            }
        }
        walk(&value, "");
    }

    /// Every non-ASCII character authored in `en.json` must be a pictograph
    /// the font cannot draw. If prose ever arrives with an accent or a curly
    /// quote in it, stripping would quietly mangle the word, so that should
    /// be a decision rather than a silent loss.
    #[test]
    fn every_non_ascii_character_is_a_pictograph() {
        // Alarm clock, stopwatch, warning sign, fuel pump, check mark, and the
        // variation selector that follows several of them.
        const KNOWN: [u32; 6] = [0x23F0, 0x23F1, 0x26A0, 0x26FD, 0x2713, 0xFE0F];
        let json = include_str!("../../assets/localization/en.json");
        for ch in json.chars().filter(|ch| !ch.is_ascii()) {
            let code = ch as u32;
            let pictographic = KNOWN.contains(&code) || (0x1F000..=0x1FAFF).contains(&code);
            assert!(
                pictographic,
                "U+{code:04X} is not a known pictograph; stripping it would mangle text"
            );
        }
    }

    /// Stripping must not eat the text around the glyph, and must leave
    /// strings that never had one exactly as authored.
    #[test]
    fn stripping_preserves_the_words_and_deliberate_spacing() {
        let mut stripped = serde_json::json!("\u{26FD} Fuel: {}% - {}");
        strip_undrawable_glyphs(&mut stripped);
        assert_eq!(stripped, serde_json::json!("Fuel: {}% - {}"));

        let mut indented = serde_json::json!("  {} passengers | Difficulty {}");
        strip_undrawable_glyphs(&mut indented);
        assert_eq!(
            indented,
            serde_json::json!("  {} passengers | Difficulty {}"),
            "deliberate indentation was reformatted"
        );
    }
}

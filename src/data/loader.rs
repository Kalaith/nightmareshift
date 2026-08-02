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

/// Load constants, or say exactly which part of the file is wrong.
pub fn try_load_constants() -> Result<ConstantsData, String> {
    let json = include_str!("../../assets/constants.json");
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
pub fn try_load_localization() -> Result<Localization, String> {
    let json = include_str!("../../assets/localization/en.json");
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

    /// Text written in Rust gets no stripping pass either.
    ///
    /// The two tests either side of this one walk JSON, which is where the
    /// glyph problem was found and fixed. It was still living in the source:
    /// an em dash in the almanac's need line and in both of the guideline
    /// screen's almanac hints, a bullet in its decision log, an arrow between
    /// a passenger's pickup and destination, and four emoji on the route
    /// preference labels. The last of those never reached the screen only
    /// because its single caller filtered non-ASCII characters itself before
    /// drawing — which is the arrangement the loader's stripping pass exists
    /// to replace, since it makes correctness depend on each caller
    /// remembering.
    ///
    /// Only string literals are checked. Comments are prose for people and
    /// several here use em dashes deliberately.
    #[test]
    fn displayed_source_text_carries_no_undrawable_glyphs() {
        // Files whose literals reach a font, plus the data types that build
        // strings for them. Listed rather than globbed because `include_str!`
        // needs a literal path; a new screen should be added here.
        const SOURCES: [(&str, &str); 8] = [
            ("ui/components.rs", include_str!("../ui/components.rs")),
            ("data/passenger.rs", include_str!("passenger.rs")),
            ("data/event.rs", include_str!("event.rs")),
            (
                "screens/game_screens/dossier.rs",
                include_str!("../screens/game_screens/dossier.rs"),
            ),
            (
                "screens/game_screens/guidelines.rs",
                include_str!("../screens/game_screens/guidelines.rs"),
            ),
            (
                "screens/game_screens/dropoff.rs",
                include_str!("../screens/game_screens/dropoff.rs"),
            ),
            (
                "screens/game_screens/interaction.rs",
                include_str!("../screens/game_screens/interaction.rs"),
            ),
            (
                "screens/game_screens/modals.rs",
                include_str!("../screens/game_screens/modals.rs"),
            ),
        ];

        for (name, source) in SOURCES {
            for (number, line) in source.lines().enumerate() {
                let code = line.trim_start();
                if code.starts_with("//") {
                    continue;
                }
                let mut in_string = false;
                let mut escaped = false;
                for ch in line.chars() {
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    match ch {
                        '\\' if in_string => escaped = true,
                        '"' => in_string = !in_string,
                        _ if in_string && !ch.is_ascii() => panic!(
                            "{name}:{} draws {ch:?}, which the bundled font renders as a box: {}",
                            number + 1,
                            code.trim()
                        ),
                        _ => {}
                    }
                }
            }
        }
    }

    /// The content files get no stripping pass, so what is authored there is
    /// what is drawn.
    ///
    /// `en.json` is cleaned at load, which made it easy to assume the problem
    /// was solved everywhere. It was not: three of the vampire's and the
    /// hunted man's escalation lines carried an em dash, so the sentence that
    /// tells the player their passenger is about to lose control broke into a
    /// replacement box at exactly the dramatic beat. The sulfur crystal's
    /// description carried another, which nothing revealed until item
    /// descriptions were displayed at all.
    ///
    /// Stripping these the way localization is stripped would be worse than
    /// fixing them: an em dash joins two clauses, and deleting it runs the
    /// words together. Plain ASCII is authored here on purpose.
    #[test]
    fn displayed_content_carries_no_undrawable_glyphs() {
        // `emoji` on a passenger and `icon` on a skill are parsed into their
        // structs and read by nothing — the menus draw vector shapes keyed by
        // name instead. They are exempt because they never reach a font, not
        // because they are safe to display.
        const UNREAD_KEYS: [&str; 2] = ["emoji", "icon"];

        fn walk(value: &serde_json::Value, path: &str, file: &str) {
            match value {
                serde_json::Value::String(text) => {
                    let bad: Vec<char> = text.chars().filter(|ch| !ch.is_ascii()).collect();
                    assert!(
                        bad.is_empty(),
                        "{file}{path} carries {bad:?}, which the bundled font draws as boxes"
                    );
                }
                serde_json::Value::Array(items) => {
                    for (i, item) in items.iter().enumerate() {
                        walk(item, &format!("{path}[{i}]"), file);
                    }
                }
                serde_json::Value::Object(map) => {
                    for (key, item) in map {
                        if UNREAD_KEYS.contains(&key.as_str()) {
                            continue;
                        }
                        walk(item, &format!("{path}.{key}"), file);
                    }
                }
                _ => {}
            }
        }

        for (file, json) in [
            (
                "passengerData.json",
                include_str!("../../assets/passengerData.json"),
            ),
            ("itemData.json", include_str!("../../assets/itemData.json")),
            (
                "locationData.json",
                include_str!("../../assets/locationData.json"),
            ),
            (
                "skillTreeData.json",
                include_str!("../../assets/skillTreeData.json"),
            ),
            (
                "almanacData.json",
                include_str!("../../assets/almanacData.json"),
            ),
            (
                "guidelineData.json",
                include_str!("../../assets/guidelineData.json"),
            ),
            (
                "eventData.json",
                include_str!("../../assets/eventData.json"),
            ),
            (
                "shiftRulesData.json",
                include_str!("../../assets/shiftRulesData.json"),
            ),
        ] {
            let value: serde_json::Value =
                serde_json::from_str(json).unwrap_or_else(|e| panic!("{file} is not valid: {e}"));
            walk(&value, "", file);
        }
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

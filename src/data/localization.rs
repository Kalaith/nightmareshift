use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Localization {
    pub meta: LocalizationMeta,
    pub ui: UiStrings,
    pub system: SystemStrings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocalizationMeta {
    pub language: String,
    pub code: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UiStrings {
    pub common: CommonStrings,
    #[serde(rename = "mainMenu")]
    pub main_menu: MainMenuStrings,
    pub briefing: BriefingStrings,
    pub meta: MetaUiStrings,
    pub game: GameStrings,
    #[serde(rename = "gameOver")]
    pub game_over: GameOverStrings,
    pub success: SuccessStrings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommonStrings {
    pub currency: String,
    pub percent: String,
    #[serde(rename = "timeFormat")]
    pub time_format: String,
    #[serde(rename = "backButton")]
    pub back_button: String,
    #[serde(rename = "continue")]
    pub continue_text: String,
    #[serde(rename = "continueSpace")]
    pub continue_space: String,
    #[serde(rename = "acceptSpace")]
    pub accept_space: String,
    #[serde(rename = "declineEsc")]
    pub decline_esc: String,
    pub loading: String,
    #[serde(rename = "tryAgain")]
    pub try_again: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MainMenuStrings {
    pub title: String,
    pub subtitle: String,
    #[serde(rename = "startSpace")]
    pub start_space: String,
    pub stats: String,
    pub achievements: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BriefingStrings {
    pub title: String,
    #[serde(rename = "rulesTitle")]
    pub rules_title: String,
    #[serde(rename = "weatherTitle")]
    pub weather_title: String,
    #[serde(rename = "beginSpace")]
    pub begin_space: String,
    /// "NIGHT {} OF {}" progress label for the current run.
    #[serde(rename = "nightLabel", default)]
    pub night_label: String,
    /// Per-night framing lines; index by (night - 1), clamped to the last entry.
    #[serde(default)]
    pub premise: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetaUiStrings {
    #[serde(rename = "skillTree")]
    pub skill_tree: SkillTreeStrings,
    pub almanac: AlmanacStrings,
    pub leaderboard: LeaderboardStrings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillTreeStrings {
    pub title: String,
    pub button: String,
    #[serde(rename = "bankBalance")]
    pub bank_balance: String,
    pub categories: SkillCategories,
    pub unlocked: String,
    pub purchase: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillCategories {
    pub survival: String,
    pub occult: String,
    pub efficiency: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AlmanacStrings {
    pub title: String,
    pub button: String,
    pub fragments: String,
    #[serde(rename = "unknownLevel")]
    pub unknown_level: String,
    #[serde(rename = "upgradeCost")]
    pub upgrade_cost: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LeaderboardStrings {
    pub title: String,
    pub button: String,
    #[serde(rename = "topRuns")]
    pub top_runs: String,
    #[serde(rename = "noRuns")]
    pub no_runs: String,
    pub achievements: String,
    #[serde(rename = "scoreEntry")]
    pub score_entry: String,
    pub details: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameStrings {
    #[serde(rename = "statusBar")]
    pub status_bar: StatusBarStrings,
    pub waiting: WaitingStrings,
    pub driving: DrivingStrings,
    pub inventory: InventoryStrings,
    pub rules: RulesStrings,
    pub guidelines: GuidelinesStrings,
    pub trade: TradeStrings,
    pub completion: CompletionStrings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StatusBarStrings {
    pub fuel: String,
    pub earnings: String,
    pub time: String,
    pub rides: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WaitingStrings {
    pub looking: String,
    #[serde(rename = "fuelLabel")]
    pub fuel_label: String,
    #[serde(rename = "fuelStatus")]
    pub fuel_status: FuelStatusStrings,
    #[serde(rename = "inventoryHint")]
    pub inventory_hint: String,
    #[serde(rename = "refuelTitle")]
    pub refuel_title: String,
    #[serde(rename = "fullTank")]
    pub full_tank: String,
    pub partial: String,
    #[serde(rename = "findPassenger")]
    pub find_passenger: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FuelStatusStrings {
    pub critical: String,
    pub low: String,
    pub medium: String,
    pub good: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DrivingStrings {
    pub pickup: String,
    pub destination: String,
    pub blocked: String,
    #[serde(rename = "nightWarning")]
    pub night_warning: String,
    #[serde(rename = "weatherWarning")]
    pub weather_warning: String,
    pub routes: RouteDescriptions,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteDescriptions {
    pub normal: String,
    #[serde(rename = "normalDesc")]
    pub normal_desc: String,
    pub shortcut: String,
    #[serde(rename = "shortcutDesc")]
    pub shortcut_desc: String,
    pub scenic: String,
    #[serde(rename = "scenicDesc")]
    pub scenic_desc: String,
    pub police: String,
    #[serde(rename = "policeDesc")]
    pub police_desc: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InventoryStrings {
    pub title: String,
    pub hint: String,
    pub count: String,
    pub empty: String,
    pub source: String,
    #[serde(rename = "clickToUse")]
    pub click_to_use: String,
    pub more: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RulesStrings {
    pub title: String,
    pub hint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GuidelinesStrings {
    pub title: String,
    pub timer: String,
    pub label: String,
    #[serde(rename = "tellsLabel")]
    pub tells_label: String,
    #[serde(rename = "noTells")]
    pub no_tells: String,
    pub follow: String,
    #[serde(rename = "break")]
    pub break_guideline: String,
    pub intensity: IntensityStrings,
}

// Map "break" in json to field break_guideline
#[derive(Debug, Deserialize, Clone)]
pub struct IntensityStrings {
    pub subtle: String,
    pub moderate: String,
    pub obvious: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TradeStrings {
    pub title: String,
    #[serde(rename = "wantsToTrade")]
    pub wants_to_trade: String,
    pub offering: String,
    #[serde(rename = "anyItem")]
    pub any_item: String,
    pub select: String,
    pub empty: String,
    pub decline: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CompletionStrings {
    pub title: String,
    pub fare: String,
    pub items: String,
    pub backstory: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameOverStrings {
    pub title: String,
    pub stats: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SuccessStrings {
    #[serde(rename = "totalEarnings")]
    pub total_earnings: String,
    #[serde(rename = "rides")]
    pub rides_completed: String,
    #[serde(rename = "bonus")]
    pub survival_bonus: String,
    #[serde(rename = "finalScore")]
    pub final_score: String,
    #[serde(rename = "nightTitle")]
    pub night_title: String,
    #[serde(rename = "nightSubtitle")]
    pub night_subtitle: String,
    #[serde(rename = "runTitle")]
    pub run_title: String,
    #[serde(rename = "runSubtitle")]
    pub run_subtitle: String,
    #[serde(rename = "nextNight")]
    pub next_night: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SystemStrings {
    pub loading: String,
    pub saving: String,
    pub error: String,
}

impl GuidelinesStrings {
    // Custom deserializer helper if needed, but serde rename is enough
    // Actually wait, 'break' is a Rust keyword. I handled it with rename above in structs but I need to ensure serde looks for "break"
}

// Re-check GuidelineStrings definition
/*
#[derive(Debug, Deserialize, Clone)]
pub struct GuidelinesStrings {
    ...
    #[serde(rename = "break")]
    pub break_guideline: String,
    ...
}
*/

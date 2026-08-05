//! UI component modules.

pub mod backgrounds;
pub mod components;
pub mod core;
mod prewarm;

pub use backgrounds::*;
pub use components::*;
pub use core::*;
pub use prewarm::*;

/// Actions triggered by UI interactions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    None,
    StartGame,
    AcceptRide,
    DeclineRide,
    SelectRoute(usize),
    SelectEventChoice(usize), // New action for mid-ride events
    Continue,
    ReturnToMenu,
    TryAgain,
    /// Advance from the results screen to the next night of the run.
    NextNight,
    EndShift,
    RefuelFull,
    RefuelPartial,
    ToggleRules,
    ToggleInventory,
    TogglePauseMenu, // ESC pause menu
    UseItem(usize),
    PerformRuleAction(String),
    /// Open the seed-entry modal on the main menu.
    OpenSeedEntry,
    /// Begin a run on today's shared daily seed.
    StartDailyRun,
    /// Begin a run on a player-entered seed.
    StartSeededRun(u64),
    // Meta-progression screens
    OpenSkillTree,
    OpenAlmanac,
    OpenLeaderboard,
    OpenHelpOptions,
    CycleTextScale,
    ToggleHighContrast,
    ToggleReducedMotion,
    CycleBrightness,
    ToggleCaptions,
    ToggleFullscreen,
    CycleMasterVolume,
    CycleAmbienceVolume,
    CycleMusicVolume,
    CycleEffectsVolume,
    DeleteSave,
    PurchaseSkill(String),
    UpgradeAlmanacKnowledge(u32),
    /// Sell surplus lore fragments back for bank balance.
    ExchangeLoreForBank,
    // Trading
    AcceptTrade(usize),
    DeclineTrade,
    // Guideline decisions
    FollowGuideline,
    BreakGuideline,
}

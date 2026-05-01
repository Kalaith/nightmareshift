//! UI component modules.

pub mod components;
pub mod core;

pub use components::*;
pub use core::*;

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
    EndShift,
    RefuelFull,
    RefuelPartial,
    ToggleRules,
    ToggleInventory,
    TogglePauseMenu, // ESC pause menu
    UseItem(usize),
    PerformRuleAction(String),
    // Meta-progression screens
    OpenSkillTree,
    OpenAlmanac,
    OpenLeaderboard,
    DeleteSave,
    PurchaseSkill(String),
    UpgradeAlmanacKnowledge(u32),
    // Trading
    AcceptTrade(usize),
    DeclineTrade,
    // Guideline decisions
    FollowGuideline,
    BreakGuideline,
}

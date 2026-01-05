//! UI component modules.

pub mod core;
pub mod components;

pub use core::*;
pub use components::*;
pub use macroquad_toolkit::ui::*; // Export toolkit UI

/// Actions triggered by UI interactions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    None,
    StartGame,
    AcceptRide,
    DeclineRide,
    SelectRoute(usize),
    Continue,
    ReturnToMenu,
    TryAgain,
    RefuelFull,
    RefuelPartial,
    ToggleRules,
    ToggleInventory,
    UseItem(usize),
    // Meta-progression screens
    OpenSkillTree,
    OpenAlmanac,
    OpenLeaderboard,
    PurchaseSkill(String),
    UpgradeAlmanacKnowledge(u32),
    // Trading
    AcceptTrade(usize),
    DeclineTrade,
    // Guideline decisions
    FollowGuideline,
    BreakGuideline,
}

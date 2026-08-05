//! Screen rendering modules.
//!
//! Screens are separated by purpose to keep file sizes manageable.
//! - `menu_screens`: Loading, Main Menu, Briefing, Game Over, Success
//! - `game_screens`: Waiting, Driving, Interaction, DropOff, Guidelines
//! - `meta_screens`: Skill Tree, Almanac, Leaderboard

pub mod game_screens;
pub mod menu_screens;
pub mod meta_screens;

/// Main screen enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Loading,
    MainMenu,
    Briefing,
    Game,
    GameOver,
    Success,
    SkillTree,
    Almanac,
    Leaderboard,
    HelpOptions,
}

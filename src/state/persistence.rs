//! Save/Load persistence system.

use super::PlayerStats;
#[cfg(target_arch = "wasm32")]
use macroquad_toolkit::persistence::{delete_slot, load_from_slot, save_to_slot, slot_exists};
#[cfg(not(target_arch = "wasm32"))]
use macroquad_toolkit::persistence::{file_exists, get_app_data_path, load_json, save_json};
use serde::{Deserialize, Serialize};

/// Save file name
const GAME_NAME: &str = "nightmare_shift";
#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE: &str = "nightmare_shift_save.json";
#[cfg(target_arch = "wasm32")]
const SAVE_SLOT: &str = "autosave";

/// Save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub player_stats: PlayerStats,
}

impl SaveData {
    /// Current save format version
    pub const VERSION: u32 = 1;

    /// Create new save data from player stats
    pub fn new(player_stats: PlayerStats) -> Self {
        Self {
            version: Self::VERSION,
            player_stats,
        }
    }
}

/// Persistence system for save/load
pub struct Persistence;

impl Persistence {
    /// Get the save file path
    #[cfg(not(target_arch = "wasm32"))]
    fn get_save_path() -> std::path::PathBuf {
        get_app_data_path(GAME_NAME, SAVE_FILE)
            .unwrap_or_else(|| std::path::PathBuf::from(SAVE_FILE))
    }

    /// Save player stats to file
    pub fn save(player_stats: &PlayerStats) -> Result<(), String> {
        let save_data = SaveData::new(player_stats.clone());
        #[cfg(not(target_arch = "wasm32"))]
        {
            save_json(Self::get_save_path(), &save_data)
        }
        #[cfg(target_arch = "wasm32")]
        {
            save_to_slot(GAME_NAME, SAVE_SLOT, &save_data)
        }
    }

    /// Load player stats from file
    pub fn load() -> Result<PlayerStats, String> {
        #[cfg(not(target_arch = "wasm32"))]
        let save_data: SaveData = load_json(Self::get_save_path())?;
        #[cfg(target_arch = "wasm32")]
        let save_data: SaveData = load_from_slot(GAME_NAME, SAVE_SLOT)?;

        // Version check for future migrations
        if save_data.version > SaveData::VERSION {
            return Err("Save file is from a newer version".to_string());
        }

        Ok(save_data.player_stats)
    }

    /// Check if a save file exists
    pub fn save_exists() -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            file_exists(Self::get_save_path())
        }
        #[cfg(target_arch = "wasm32")]
        {
            slot_exists(GAME_NAME, SAVE_SLOT)
        }
    }

    /// Delete the save file
    pub fn delete_save() -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = Self::get_save_path();
            if path.exists() {
                std::fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            delete_slot(GAME_NAME, SAVE_SLOT)
        }
    }
}

//! Save/Load persistence system.

use serde::{Deserialize, Serialize};
use super::PlayerStats;
use macroquad_toolkit::persistence::{save_json, load_json, get_app_data_path, file_exists};

/// Save file name
const SAVE_FILE: &str = "nightmare_shift_save.json";

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
    fn get_save_path() -> std::path::PathBuf {
         get_app_data_path("nightmare_shift", SAVE_FILE)
            .unwrap_or_else(|| std::path::PathBuf::from(SAVE_FILE))
    }

    /// Save player stats to file
    pub fn save(player_stats: &PlayerStats) -> Result<(), String> {
        let save_data = SaveData::new(player_stats.clone());
        save_json(Self::get_save_path(), &save_data)
    }

    /// Load player stats from file
    pub fn load() -> Result<PlayerStats, String> {
        let save_data: SaveData = load_json(Self::get_save_path())?;

        // Version check for future migrations
        if save_data.version > SaveData::VERSION {
            return Err("Save file is from a newer version".to_string());
        }

        Ok(save_data.player_stats)
    }

    /// Check if a save file exists
    pub fn save_exists() -> bool {
        file_exists(Self::get_save_path())
    }

    /// Delete the save file
    pub fn delete_save() -> Result<(), String> {
        let path = Self::get_save_path();
        if path.exists() {
             std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

//! Save/Load persistence system.

use serde::{Deserialize, Serialize};
use super::PlayerStats;

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
    #[cfg(not(target_arch = "wasm32"))]
    fn get_save_path() -> Option<std::path::PathBuf> {
        // Try to get user data directory
        if let Some(data_dir) = dirs::data_local_dir() {
            let game_dir = data_dir.join("nightmare_shift");
            if !game_dir.exists() {
                let _ = std::fs::create_dir_all(&game_dir);
            }
            Some(game_dir.join(SAVE_FILE))
        } else {
            // Fallback to current directory
            Some(std::path::PathBuf::from(SAVE_FILE))
        }
    }

    /// Save player stats to file (desktop only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(player_stats: &PlayerStats) -> Result<(), String> {
        let save_data = SaveData::new(player_stats.clone());
        let json = serde_json::to_string_pretty(&save_data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        let path = Self::get_save_path()
            .ok_or_else(|| "Could not determine save path".to_string())?;

        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write save file: {}", e))?;

        Ok(())
    }

    /// Load player stats from file (desktop only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Result<PlayerStats, String> {
        let path = Self::get_save_path()
            .ok_or_else(|| "Could not determine save path".to_string())?;

        if !path.exists() {
            return Err("No save file found".to_string());
        }

        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;

        let save_data: SaveData = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse save file: {}", e))?;

        // Version check for future migrations
        if save_data.version > SaveData::VERSION {
            return Err("Save file is from a newer version".to_string());
        }

        Ok(save_data.player_stats)
    }

    /// Check if a save file exists (desktop only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_exists() -> bool {
        Self::get_save_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Delete the save file (desktop only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn delete_save() -> Result<(), String> {
        let path = Self::get_save_path()
            .ok_or_else(|| "Could not determine save path".to_string())?;

        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete save file: {}", e))?;
        }

        Ok(())
    }

    // WASM stubs - use localStorage in browser
    #[cfg(target_arch = "wasm32")]
    pub fn save(_player_stats: &PlayerStats) -> Result<(), String> {
        // TODO: Implement localStorage saving for WASM
        Err("WASM saving not yet implemented".to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Result<PlayerStats, String> {
        Err("WASM loading not yet implemented".to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_exists() -> bool {
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub fn delete_save() -> Result<(), String> {
        Err("WASM delete not yet implemented".to_string())
    }
}

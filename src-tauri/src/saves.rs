use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::engine::game_state::GameState;
use crate::ai::client::ApiMessage;

const MAX_SLOTS: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub slot: usize,
    pub name: String,
    pub saved_at: String, // Unix epoch seconds as string
    pub player_name: String,
    pub scene: String,
    pub game_state: GameState,
    pub ai_history: Vec<ApiMessage>,
}

/// Lightweight metadata for listing saves without loading full GameState.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub slot: usize,
    pub name: String,
    pub saved_at: String,
    pub player_name: String,
    pub scene: String,
}

fn slot_path(app_data_dir: &PathBuf, slot: usize) -> PathBuf {
    app_data_dir.join("saves").join(format!("slot_{}.json", slot))
}

pub fn save_game(
    slot: usize,
    name: String,
    saved_at: String,
    game_state: &GameState,
    ai_history: &[ApiMessage],
    app_data_dir: &PathBuf,
) -> Result<(), String> {
    if slot >= MAX_SLOTS {
        return Err(format!("Invalid slot: {}", slot));
    }

    let path = slot_path(app_data_dir, slot);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let save = SaveSlot {
        slot,
        name,
        saved_at,
        player_name: game_state.player.name.clone(),
        scene: game_state.scene.clone(),
        game_state: game_state.clone(),
        ai_history: ai_history.to_vec(),
    };

    let text = serde_json::to_string_pretty(&save).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

pub fn load_game(slot: usize, app_data_dir: &PathBuf) -> Option<SaveSlot> {
    if slot >= MAX_SLOTS {
        return None;
    }

    let text = fs::read_to_string(slot_path(app_data_dir, slot)).ok()?;
    serde_json::from_str::<SaveSlot>(&text).ok()
}

/// Returns a Vec of exactly MAX_SLOTS entries. Missing or corrupt slots are None.
pub fn list_saves(app_data_dir: &PathBuf) -> Vec<Option<SaveMetadata>> {
    (0..MAX_SLOTS)
        .map(|slot| {
            let text = fs::read_to_string(slot_path(app_data_dir, slot)).ok()?;
            let s: SaveSlot = serde_json::from_str(&text).ok()?;
            Some(SaveMetadata {
                slot: s.slot,
                name: s.name,
                saved_at: s.saved_at,
                player_name: s.player_name,
                scene: s.scene,
            })
        })
        .collect()
}

pub fn delete_save(slot: usize, app_data_dir: &PathBuf) -> Result<(), String> {
    if slot >= MAX_SLOTS {
        return Err(format!("Invalid slot: {}", slot));
    }

    let path = slot_path(app_data_dir, slot);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())
    } else {
        Ok(()) // idempotent
    }
}

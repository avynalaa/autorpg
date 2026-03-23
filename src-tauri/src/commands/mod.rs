pub mod executor;

use std::sync::Mutex;
use std::path::PathBuf;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::engine::{
    game_state::{GameState, EntrySource},
    dice,
};
use crate::ai::{client::AiClient, client::ApiMessage, parser, SYSTEM_PROMPT};
use crate::commands::executor::apply_command;
use crate::settings;
use crate::saves::{self, SaveMetadata};

pub struct AppState {
    pub game: Mutex<GameState>,
    pub ai_history: Mutex<Vec<ApiMessage>>,
    pub api_key: Mutex<String>,
    pub model: Mutex<String>,
    pub endpoint: Mutex<String>,
    pub api_format: Mutex<String>,
    pub system_prompt: Mutex<String>,
    pub history_mode: Mutex<String>,
    pub history_limit: Mutex<u32>,
    pub max_tokens: Mutex<u32>,
    pub app_data_dir: PathBuf,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let s = settings::load(&app_data_dir);
        Self {
            game: Mutex::new(GameState::new()),
            ai_history: Mutex::new(Vec::new()),
            api_key: Mutex::new(s.api_key),
            model: Mutex::new(s.model),
            endpoint: Mutex::new(s.endpoint),
            api_format: Mutex::new(s.api_format),
            system_prompt: Mutex::new(s.system_prompt),
            history_mode: Mutex::new(s.history_mode),
            history_limit: Mutex::new(s.history_limit),
            max_tokens: Mutex::new(s.max_tokens),
            app_data_dir,
        }
    }

    fn save_settings(&self) {
        let s = settings::PersistedSettings {
            api_key: self.api_key.lock().unwrap().clone(),
            model: self.model.lock().unwrap().clone(),
            endpoint: self.endpoint.lock().unwrap().clone(),
            api_format: self.api_format.lock().unwrap().clone(),
            system_prompt: self.system_prompt.lock().unwrap().clone(),
            history_mode: self.history_mode.lock().unwrap().clone(),
            history_limit: *self.history_limit.lock().unwrap(),
            max_tokens: *self.max_tokens.lock().unwrap(),
        };
        settings::save(&self.app_data_dir, &s);
    }
}

// ── Settings ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_api_key(key: String, state: State<AppState>) {
    *state.api_key.lock().unwrap() = key;
    state.save_settings();
}

#[tauri::command]
pub fn set_model(model: String, state: State<AppState>) {
    *state.model.lock().unwrap() = model;
    state.save_settings();
}

#[tauri::command]
pub fn set_endpoint(url: String, state: State<AppState>) {
    *state.endpoint.lock().unwrap() = url;
    state.save_settings();
}

#[tauri::command]
pub fn set_api_format(format: String, state: State<AppState>) {
    *state.api_format.lock().unwrap() = format;
    state.save_settings();
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> serde_json::Value {
    serde_json::json!({
        "endpoint": *state.endpoint.lock().unwrap(),
        "api_format": *state.api_format.lock().unwrap(),
        "model": *state.model.lock().unwrap(),
        "history_mode": *state.history_mode.lock().unwrap(),
        "history_limit": *state.history_limit.lock().unwrap(),
        "max_tokens": *state.max_tokens.lock().unwrap(),
    })
}

#[tauri::command]
pub async fn fetch_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let api_key  = state.api_key.lock().unwrap().clone();
    let endpoint = state.endpoint.lock().unwrap().clone();
    let model    = state.model.lock().unwrap().clone();
    let format   = state.api_format.lock().unwrap().clone();
    let client = AiClient::new(api_key, endpoint, model, format);
    client.fetch_models().await
}

#[tauri::command]
pub fn set_system_prompt(prompt: String, state: State<AppState>) {
    *state.system_prompt.lock().unwrap() = prompt;
    state.save_settings();
}

#[tauri::command]
pub fn get_system_prompt(state: State<AppState>) -> String {
    state.system_prompt.lock().unwrap().clone()
}

#[tauri::command]
pub fn reset_system_prompt(state: State<AppState>) {
    *state.system_prompt.lock().unwrap() = SYSTEM_PROMPT.to_string();
    state.save_settings();
}

#[tauri::command]
pub fn set_history_mode(mode: String, state: State<AppState>) {
    *state.history_mode.lock().unwrap() = mode;
    state.save_settings();
}

#[tauri::command]
pub fn set_history_limit(limit: u32, state: State<AppState>) {
    *state.history_limit.lock().unwrap() = limit;
    state.save_settings();
}

#[tauri::command]
pub fn set_max_tokens(tokens: u32, state: State<AppState>) {
    *state.max_tokens.lock().unwrap() = tokens;
    state.save_settings();
}

// ── Game State ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_game_state(state: State<AppState>) -> GameState {
    state.game.lock().unwrap().clone()
}

#[tauri::command]
pub fn reset_game(state: State<AppState>) {
    let mut gs = GameState::new();
    gs.phase = crate::engine::game_state::GamePhase::CharacterCreation;
    *state.game.lock().unwrap() = gs;
    *state.ai_history.lock().unwrap() = Vec::new();
}

#[tauri::command]
pub fn set_player_name(name: String, state: State<AppState>) {
    state.game.lock().unwrap().player.name = name;
}

#[tauri::command]
pub fn set_player_stats(
    hp_max: i32, mp_max: i32,
    str_: i32, dex: i32, con: i32, int_: i32, wis: i32, cha: i32,
    state: State<AppState>
) {
    let mut gs = state.game.lock().unwrap();
    gs.player.hp_max = hp_max;
    gs.player.hp = hp_max;
    gs.player.mp_max = mp_max;
    gs.player.mp = mp_max;
    let scores = &mut gs.player.ability_scores;
    scores.strength = str_;
    scores.dexterity = dex;
    scores.constitution = con;
    scores.intelligence = int_;
    scores.wisdom = wis;
    scores.charisma = cha;
}

// ── Dice ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn roll_dice(notation: String) -> dice::DiceRoll {
    dice::roll(&notation)
}

#[tauri::command]
pub fn roll_dice_labeled(notation: String, label: String) -> dice::DiceRoll {
    dice::roll_with_label(&notation, &label)
}

// ── AI Interaction ──────────────────────────────────────────────────────────

/// Returns a slice of history to send to the AI, based on mode and limit.
/// Does NOT mutate the original vec.
fn truncate_for_send<'a>(
    history: &'a [ApiMessage],
    mode: &str,
    limit: u32,
) -> &'a [ApiMessage] {
    match mode {
        "tokens" => {
            // Estimate: chars / 4 per message. Walk from end, keep pairs.
            let limit = limit as usize;
            let mut total_chars = 0usize;
            let mut keep_from = history.len();
            // Iterate in reverse pairs (assistant, user)
            let mut i = history.len();
            while i >= 2 {
                let chars = history[i - 1].content.chars().count()
                    + history[i - 2].content.chars().count();
                let tokens_estimate = chars / 4;
                if total_chars + tokens_estimate > limit {
                    break;
                }
                total_chars += tokens_estimate;
                keep_from = i - 2;
                i -= 2;
            }
            &history[keep_from..]
        }
        _ => {
            // "turns" mode: keep last (limit * 2) messages
            let max_msgs = (limit as usize) * 2;
            if history.len() > max_msgs {
                &history[history.len() - max_msgs..]
            } else {
                history
            }
        }
    }
}

#[derive(Serialize)]
pub struct AiResponse {
    pub narrative: String,
    pub commands_applied: Vec<String>,
    pub game_state: GameState,
}

#[tauri::command]
pub async fn send_to_ai(
    player_input: String,
    state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    let api_key  = state.api_key.lock().unwrap().clone();
    if api_key.is_empty() {
        return Err("API key not set. Open Settings to configure your endpoint and API key.".to_string());
    }

    let model         = state.model.lock().unwrap().clone();
    let endpoint      = state.endpoint.lock().unwrap().clone();
    let api_format    = state.api_format.lock().unwrap().clone();
    let system_prompt = state.system_prompt.lock().unwrap().clone();
    let history_mode  = state.history_mode.lock().unwrap().clone();
    let history_limit = *state.history_limit.lock().unwrap();
    let max_tokens    = *state.max_tokens.lock().unwrap();
    let full_history  = state.ai_history.lock().unwrap().clone();

    // Build compact context + player input
    let context = {
        let gs = state.game.lock().unwrap();
        gs.build_ai_context()
    };
    let full_message = format!("{}\n\nPlayer: {}", context, player_input);

    // Call AI
    let client = AiClient::new(api_key, endpoint, model, api_format);
    let send_history = truncate_for_send(&full_history, &history_mode, history_limit);
    let raw_response = client.send(&system_prompt, send_history, &full_message, max_tokens).await?;

    // Parse commands from response
    let parsed = parser::parse_ai_response(&raw_response);

    // Apply commands to game state
    let mut cmd_log: Vec<String> = Vec::new();
    {
        let mut gs = state.game.lock().unwrap();

        // Clear previous choices before applying new response
        gs.clear_choices();

        // Log player input
        gs.add_story_entry(EntrySource::Player, &player_input);

        // Apply commands
        for cmd in &parsed.commands {
            if let Some(msg) = apply_command(&mut gs, cmd) {
                cmd_log.push(msg.clone());
                gs.add_story_entry(EntrySource::System, &msg);
            }
        }

        // Log AI narrative
        gs.add_story_entry(EntrySource::Narrator, &parsed.clean_text);
    }

    // Update conversation history (storage cap: 200 turns = 400 messages)
    {
        let mut hist = state.ai_history.lock().unwrap();
        hist.push(ApiMessage { role: "user".to_string(), content: full_message });
        hist.push(ApiMessage { role: "assistant".to_string(), content: raw_response.clone() });
        while hist.len() > 400 {
            hist.drain(0..2);
        }
    }

    let game_state = state.game.lock().unwrap().clone();
    Ok(AiResponse {
        narrative: parsed.clean_text,
        commands_applied: cmd_log,
        game_state,
    })
}

#[tauri::command]
pub fn clear_ai_history(state: State<AppState>) {
    state.ai_history.lock().unwrap().clear();
}

// ── Save / Load ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct LoadResult {
    pub game_state: GameState,
}

#[tauri::command]
pub fn save_game_cmd(
    slot: usize,
    name: String,
    state: State<AppState>,
) -> Result<(), String> {
    let gs = state.game.lock().unwrap().clone();
    let hist = state.ai_history.lock().unwrap().clone();
    // Generate timestamp as Unix epoch seconds string
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    saves::save_game(slot, name, ts, &gs, &hist, &state.app_data_dir)
}

#[tauri::command]
pub fn load_game_cmd(
    slot: usize,
    state: State<AppState>,
) -> Result<LoadResult, String> {
    let save = saves::load_game(slot, &state.app_data_dir)
        .ok_or_else(|| format!("No save found in slot {}", slot))?;
    *state.game.lock().unwrap() = save.game_state.clone();
    *state.ai_history.lock().unwrap() = save.ai_history;
    Ok(LoadResult { game_state: save.game_state })
}

#[tauri::command]
pub fn list_saves_cmd(state: State<AppState>) -> Vec<Option<SaveMetadata>> {
    saves::list_saves(&state.app_data_dir)
}

#[tauri::command]
pub fn delete_save_cmd(slot: usize, state: State<AppState>) -> Result<(), String> {
    saves::delete_save(slot, &state.app_data_dir)
}

// ── Manual game state mutations (for UI controls) ───────────────────────────

#[tauri::command]
pub fn remove_inventory_item(slot_idx: usize, state: State<AppState>) -> GameState {
    state.game.lock().unwrap().inventory.remove_item(slot_idx);
    state.game.lock().unwrap().clone()
}

#[tauri::command]
pub fn equip_item(slot_idx: usize, state: State<AppState>) -> Result<GameState, String> {
    let mut gs = state.game.lock().unwrap();
    gs.inventory.equip_item(slot_idx)?;
    Ok(gs.clone())
}

#[tauri::command]
pub fn player_attack(target_id: String, state: State<AppState>) -> Result<GameState, String> {
    let mut gs = state.game.lock().unwrap();
    let player_id = gs.player.id.clone();
    let result = gs.combat.resolve_attack(&player_id, &target_id);
    // Sync player HP from combat
    if let Some(c) = gs.combat.combatants.iter().find(|c| c.id == player_id) {
        gs.player.hp = c.hp;
    }
    gs.add_story_entry(EntrySource::System, &result.description);
    gs.combat.end_turn();
    Ok(gs.clone())
}

#[derive(Deserialize)]
pub struct NewCharacter {
    pub name: String,
    pub race: String,
    pub class: String,
    pub backstory: String,
}

#[tauri::command]
pub fn create_character(data: NewCharacter, state: State<AppState>) -> GameState {
    use crate::engine::character::{CharacterRace, CharacterClass};
    let mut gs = state.game.lock().unwrap();
    gs.player.name = data.name;
    gs.player.backstory = data.backstory;
    gs.player.race = match data.race.to_lowercase().as_str() {
        "elf"      => CharacterRace::Elf,
        "dwarf"    => CharacterRace::Dwarf,
        "halfling" => CharacterRace::Halfling,
        "orc"      => CharacterRace::Orc,
        _          => CharacterRace::Human,
    };
    gs.player.class = match data.class.to_lowercase().as_str() {
        "mage"    => CharacterClass::Mage,
        "rogue"   => CharacterClass::Rogue,
        "cleric"  => CharacterClass::Cleric,
        "ranger"  => CharacterClass::Ranger,
        "bard"    => CharacterClass::Bard,
        _         => CharacterClass::Warrior,
    };
    gs.phase = crate::engine::game_state::GamePhase::Playing;
    gs.clone()
}

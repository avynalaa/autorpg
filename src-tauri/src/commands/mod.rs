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
    pub last_player_input: Mutex<String>,
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
            last_player_input: Mutex::new(String::new()),
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
            system_prompt_customized: true,
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
    let s = settings::PersistedSettings {
        api_key: state.api_key.lock().unwrap().clone(),
        model: state.model.lock().unwrap().clone(),
        endpoint: state.endpoint.lock().unwrap().clone(),
        api_format: state.api_format.lock().unwrap().clone(),
        system_prompt: SYSTEM_PROMPT.to_string(),
        system_prompt_customized: false,
        history_mode: state.history_mode.lock().unwrap().clone(),
        history_limit: *state.history_limit.lock().unwrap(),
        max_tokens: *state.max_tokens.lock().unwrap(),
    };
    settings::save(&state.app_data_dir, &s);
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

fn sync_post_combat_state(gs: &mut GameState) {
    use crate::engine::combat::CombatState;
    use crate::engine::game_state::GamePhase;

    match gs.combat.state {
        CombatState::Victory | CombatState::Fled | CombatState::OutOfCombat => {
            gs.phase = GamePhase::Playing;
        }
        CombatState::Defeat => {
            gs.phase = GamePhase::GameOver;
        }
        _ => {
            gs.phase = GamePhase::Combat;
        }
    }
}

fn resolve_enemy_opening_turn(gs: &mut GameState) {
    use crate::engine::combat::CombatState;

    if gs.combat.state != CombatState::EnemyTurn {
        return;
    }

    if let Some(enemy_id) = gs.combat.turn_order.get(gs.combat.current_turn_idx).cloned() {
        if let Some(enemy_result) = gs.combat.enemy_auto_turn(&enemy_id) {
            gs.add_story_entry(EntrySource::System, &enemy_result.description);
        }
    }

    if let Some(c) = gs.combat.combatants.iter().find(|c| c.id == gs.player.id) {
        gs.player.hp = c.hp;
    }

    sync_post_combat_state(gs);
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

    // Store for potential regenerate
    *state.last_player_input.lock().unwrap() = player_input.clone();

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

        // Post-process npc_posture: apply relation floor, inject combat choices, auto-start if hostile
        let posture_level = |p: &str| -> i32 { match p { "hostile" => 3, "tense" => 2, "reluctant" => 1, _ => 0 } };
        let npc_ids: Vec<String> = parsed.commands.iter()
            .filter(|c| c.verb == "npc_posture")
            .filter_map(|c| c.args.get(0).cloned())
            .collect();

        for id in &npc_ids {
            let raw_posture = gs.flags.get(&format!("npc_{}_posture_raw", id))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "calm".to_string());
            let name = gs.flags.get(&format!("npc_{}_name", id))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| id.replace('_', " "));

            // Relation floor: hostile NPCs can't be talked below tense
            let relation_score = gs.relations.relations.get(id).map(|r| r.score).unwrap_or(0);
            let floor = if relation_score < -75 { "hostile" } else if relation_score < -50 { "tense" } else { "" };
            let effective = if !floor.is_empty() && posture_level(floor) > posture_level(&raw_posture) {
                floor.to_string()
            } else {
                raw_posture.clone()
            };

            gs.set_flag(&format!("npc_{}_posture", id), serde_json::Value::String(effective.clone()));

            // Retrieve AI-provided stats; fall back to placeholder if AI forgot npc_stats
            let hp  = gs.flags.get(&format!("npc_{}_hp",  id)).and_then(|v| v.as_i64()).unwrap_or(20) as i32;
            let atk = gs.flags.get(&format!("npc_{}_atk", id)).and_then(|v| v.as_i64()).unwrap_or(5)  as i32;
            let def = gs.flags.get(&format!("npc_{}_def", id)).and_then(|v| v.as_i64()).unwrap_or(12) as i32;

            match effective.as_str() {
                "hostile" if gs.phase != crate::engine::game_state::GamePhase::Combat => {
                    use crate::engine::combat::Combatant;
                    let player_c = Combatant::from_character(&gs.player);
                    let enemy = Combatant::new_enemy(id, &name, hp, atk, def);
                    gs.combat.start(vec![player_c, enemy]);
                    gs.phase = crate::engine::game_state::GamePhase::Combat;
                    let msg = format!("{} turns hostile — combat begins.", name);
                    cmd_log.push(msg.clone());
                    gs.add_story_entry(EntrySource::System, &msg);
                    resolve_enemy_opening_turn(&mut gs);
                }
                "tense" => {
                    gs.choices.push(crate::engine::game_state::PlayerChoice {
                        id: format!("escalate_{}", id),
                        text: format!("Escalate the confrontation with {}", name),
                        style: crate::engine::game_state::ChoiceStyle::Danger,
                        tags: vec!["combat_push".to_string()],
                    });
                }
                "reluctant" => {
                    gs.choices.push(crate::engine::game_state::PlayerChoice {
                        id: format!("force_fight_{}", id),
                        text: format!("Force the fight with {}", name),
                        style: crate::engine::game_state::ChoiceStyle::Danger,
                        tags: vec!["combat_push".to_string()],
                    });
                }
                _ => {}
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

#[tauri::command]
pub fn go_to_menu(state: State<AppState>) -> GameState {
    *state.game.lock().unwrap() = GameState::new();
    *state.ai_history.lock().unwrap() = Vec::new();
    *state.last_player_input.lock().unwrap() = String::new();
    state.game.lock().unwrap().clone()
}

#[tauri::command]
pub async fn regenerate_last(
    state: State<'_, AppState>,
) -> Result<AiResponse, String> {
    let api_key = state.api_key.lock().unwrap().clone();
    if api_key.is_empty() {
        return Err("API key not set.".to_string());
    }

    let player_input = {
        let cached = state.last_player_input.lock().unwrap().clone();
        if !cached.is_empty() {
            cached
        } else {
            // Fall back to the last Player entry in the story log (e.g. after loading a save)
            let gs = state.game.lock().unwrap();
            gs.story_log.iter().rev()
                .find(|e| e.source == crate::engine::game_state::EntrySource::Player)
                .map(|e| e.text.clone())
                .unwrap_or_default()
        }
    };
    if player_input.is_empty() {
        return Err("Nothing to regenerate.".to_string());
    }

    // Roll back history by one turn
    {
        let mut hist = state.ai_history.lock().unwrap();
        let new_len = hist.len().saturating_sub(2);
        hist.truncate(new_len);
    }

    // Roll back story log
    {
        let mut gs = state.game.lock().unwrap();
        gs.trim_last_turn();
        gs.clear_choices();
    }

    let model         = state.model.lock().unwrap().clone();
    let endpoint      = state.endpoint.lock().unwrap().clone();
    let api_format    = state.api_format.lock().unwrap().clone();
    let system_prompt = state.system_prompt.lock().unwrap().clone();
    let history_mode  = state.history_mode.lock().unwrap().clone();
    let history_limit = *state.history_limit.lock().unwrap();
    let max_tokens    = *state.max_tokens.lock().unwrap();
    let full_history  = state.ai_history.lock().unwrap().clone();

    let context = {
        let gs = state.game.lock().unwrap();
        gs.build_ai_context()
    };
    let full_message = format!("{}\n\nPlayer: {}", context, player_input);

    let client = AiClient::new(api_key, endpoint, model, api_format);
    let send_history = truncate_for_send(&full_history, &history_mode, history_limit);
    let raw_response = client.send(&system_prompt, send_history, &full_message, max_tokens).await?;

    let parsed = parser::parse_ai_response(&raw_response);

    let mut cmd_log: Vec<String> = Vec::new();
    {
        let mut gs = state.game.lock().unwrap();
        gs.add_story_entry(EntrySource::Player, &player_input);
        for cmd in &parsed.commands {
            if let Some(msg) = apply_command(&mut gs, cmd) {
                cmd_log.push(msg.clone());
                gs.add_story_entry(EntrySource::System, &msg);
            }
        }
        gs.add_story_entry(EntrySource::Narrator, &parsed.clean_text);
    }

    {
        let mut hist = state.ai_history.lock().unwrap();
        hist.push(ApiMessage { role: "user".to_string(), content: full_message });
        hist.push(ApiMessage { role: "assistant".to_string(), content: raw_response });
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
pub fn force_npc_combat(npc_id: String, state: State<AppState>) -> GameState {
    use crate::engine::combat::Combatant;
    let mut gs = state.game.lock().unwrap();
    let name = gs.flags.get(&format!("npc_{}_name", npc_id))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| npc_id.replace('_', " "));
    let hp  = gs.flags.get(&format!("npc_{}_hp",  npc_id)).and_then(|v| v.as_i64()).unwrap_or(20) as i32;
    let atk = gs.flags.get(&format!("npc_{}_atk", npc_id)).and_then(|v| v.as_i64()).unwrap_or(5)  as i32;
    let def = gs.flags.get(&format!("npc_{}_def", npc_id)).and_then(|v| v.as_i64()).unwrap_or(12) as i32;
    let player_c = Combatant::from_character(&gs.player);
    let enemy = Combatant::new_enemy(&npc_id, &name, hp, atk, def);
    gs.combat.start(vec![player_c, enemy]);
    gs.phase = crate::engine::game_state::GamePhase::Combat;
    gs.clear_choices();
    gs.add_story_entry(EntrySource::System, &format!("You force the fight with {} — combat begins.", name));
    resolve_enemy_opening_turn(&mut gs);
    gs.clone()
}

#[tauri::command]
pub fn force_start_combat(
    enemy_name: String,
    hp: i32,
    atk: i32,
    def: i32,
    state: State<AppState>,
) -> GameState {
    use crate::engine::combat::Combatant;
    let mut gs = state.game.lock().unwrap();
    let enemy_id = enemy_name.to_lowercase().replace(' ', "_");
    let player_combatant = Combatant::from_character(&gs.player);
    let enemy = Combatant::new_enemy(&enemy_id, &enemy_name, hp, atk, def);
    gs.combat.start(vec![player_combatant, enemy]);
    gs.phase = crate::engine::game_state::GamePhase::Combat;
    gs.add_story_entry(EntrySource::System, &format!("Combat started: {}", enemy_name));
    resolve_enemy_opening_turn(&mut gs);
    gs.clone()
}

#[tauri::command]
pub fn player_attack(target_id: String, state: State<AppState>) -> Result<GameState, String> {
    let mut gs = state.game.lock().unwrap();
    let player_id = gs.player.id.clone();
    let result = gs.combat.resolve_attack(&player_id, &target_id);
    gs.add_story_entry(EntrySource::System, &result.description);
    gs.combat.end_turn();

    if let Some(c) = gs.combat.combatants.iter().find(|c| c.id == player_id) {
        gs.player.hp = c.hp;
    }

    resolve_enemy_opening_turn(&mut gs);
    sync_post_combat_state(&mut gs);

    Ok(gs.clone())
}

#[derive(Deserialize)]
pub struct NewCharacter {
    pub name: String,
    pub last_name: Option<String>,
    pub age: Option<u32>,
    pub gender: String,
    pub sex: String,
    pub appearance: String,
    pub personality: String,
    pub race: String,
    pub class: String,
    pub backstory: String,
    pub ability_scores: Option<crate::engine::character::AbilityScores>,
}

#[tauri::command]
pub fn create_character(data: NewCharacter, state: State<AppState>) -> GameState {
    use crate::engine::character::{CharacterRace, CharacterClass};
    let mut gs = state.game.lock().unwrap();
    
    // Identity
    gs.player.name = data.name;
    gs.player.last_name = data.last_name;
    gs.player.age = data.age;
    gs.player.gender = data.gender;
    gs.player.sex = data.sex;
    
    // Persona
    gs.player.appearance = data.appearance;
    gs.player.personality = data.personality;
    gs.player.backstory = data.backstory;
    
    // Race
    gs.player.race = match data.race.to_lowercase().as_str() {
        "elf"      => CharacterRace::Elf,
        "dwarf"    => CharacterRace::Dwarf,
        "halfling" => CharacterRace::Halfling,
        "orc"      => CharacterRace::Orc,
        _          => CharacterRace::Human,
    };
    
    // Class
    gs.player.class = match data.class.to_lowercase().as_str() {
        "mage"    => CharacterClass::Mage,
        "rogue"   => CharacterClass::Rogue,
        "cleric"  => CharacterClass::Cleric,
        "ranger"  => CharacterClass::Ranger,
        "bard"    => CharacterClass::Bard,
        _         => CharacterClass::Warrior,
    };
    
    // Ability scores — use provided or class preset
    gs.player.ability_scores = data.ability_scores.unwrap_or_else(|| gs.player.class.preset_scores());
    
    // Calculate HP/MP based on class and CON/INT/WIS/CHA
    let con_mod = gs.player.ability_scores.con_mod();
    gs.player.hp_max = gs.player.class.base_hp() + con_mod;
    gs.player.hp = gs.player.hp_max;
    gs.player.mp_max = gs.player.class.base_mp(&gs.player.ability_scores);
    gs.player.mp = gs.player.mp_max;
    
    gs.phase = crate::engine::game_state::GamePhase::Playing;
    gs.clone()
}

mod engine;
mod ai;
mod commands;
mod settings;
mod saves;

use commands::{AppState, *};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data dir");
            app.manage(AppState::new(app_data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Settings
            set_api_key,
            set_model,
            set_endpoint,
            set_api_format,
            get_settings,
            fetch_models,
            set_system_prompt,
            get_system_prompt,
            reset_system_prompt,
            set_history_mode,
            set_history_limit,
            set_max_tokens,
            // Game state
            get_game_state,
            reset_game,
            set_player_name,
            set_player_stats,
            create_character,
            // AI
            send_to_ai,
            clear_ai_history,
            regenerate_last,
            go_to_menu,
            // Save / Load
            save_game_cmd,
            load_game_cmd,
            list_saves_cmd,
            delete_save_cmd,
            // Dice
            roll_dice,
            roll_dice_labeled,
            // Inventory
            remove_inventory_item,
            equip_item,
            // Combat
            player_attack,
            force_npc_combat,
            force_start_combat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

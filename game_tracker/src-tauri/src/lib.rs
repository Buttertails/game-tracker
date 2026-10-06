pub mod api;
pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use api::IgdbClient;
use commands::AppState;
use db::Database;
use std::sync::Mutex;
use tauri::Manager;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    let client_id = match option_env!("IGDB_CLIENT_ID") {
        Some(s) => s.to_string(),
        None => std::env::var("IGDB_CLIENT_ID").expect("IGDB_CLIENT_ID not set"),
    };

    let client_secret = match option_env!("IGDB_CLIENT_SECRET") {
        Some(s) => s.to_string(),
        None => std::env::var("IGDB_CLIENT_SECRET").expect("IGDB_CLIENT_SECRET not set"),
    };

    let igdb_client = IgdbClient::new(client_id, client_secret);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("game_shelf.db");

            let db = Database::new(db_path.to_str().unwrap()).unwrap();

            let shelf_service = services::shelf_service::ShelfService::new(&db);
            if shelf_service.list_all().unwrap().is_empty() {
                shelf_service.create("Solo").unwrap();
            }

            app.manage(AppState {
                db: Mutex::new(db),
                igdb_client,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Shelf
            commands::get_shelves,
            commands::create_shelf,
            commands::rename_shelf,
            commands::delete_shelf,
            // Game search & entry
            commands::search_games,
            commands::add_game_from_search,
            commands::add_game_manually,
            // Game entry detail
            commands::get_game_entry,
            commands::get_entries_by_shelf,
            commands::delete_game_entry,
            commands::update_ownership_status,
            commands::update_entry_metadata,
            // Status transitions
            commands::move_to_in_progress,
            commands::move_to_completed,
            commands::move_to_backlog,
            commands::undo_completion_to_in_progress,
            commands::undo_completion_to_backlog,
            // Smart Fill
            commands::get_smart_fill_suggestions,
            commands::accept_smart_fill,
            commands::reroll_suggestion,
            // Notes
            commands::add_note,
            commands::delete_note,
            commands::get_notes,
            // Tags
            commands::add_tag,
            commands::remove_tag,
            // Launch
            commands::update_launch_path,
            commands::launch_game,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}

pub mod api;
pub mod commands;
pub mod db;
pub mod models;
pub mod services;

use api::RawgClient;
use commands::AppState;
use db::Database;
use std::env;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("game_backlog.db");
    let db = Database::new(db_path.to_str().unwrap()).unwrap();

    {
        let service = services::category_service::CategoryService::new(&db);
        service.initialize_presets().unwrap();
    }

    dotenvy::dotenv().ok();
    let api_key = std::env::var("RAWG_API_KEY").expect("RAWG_API_KEY not set");
    let rawg_client = RawgClient::new(api_key);

    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(db),
            rawg_client,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_categories,
            commands::create_category,
            commands::delete_category,
            commands::search_games,
            commands::add_game_from_search,
            commands::add_game_manually,
            commands::move_to_category,
            commands::delete_game_entry,
            commands::update_launch_path,
            commands::launch_game,
            commands::update_last_played,
            commands::add_tag,
            commands::remove_tag,
            commands::update_source,
            commands::get_game_entry,
            commands::get_entries_by_category,
        ])
        .run(tauri::generate_context!())
        .expect("Error while runnig tauri application");
}

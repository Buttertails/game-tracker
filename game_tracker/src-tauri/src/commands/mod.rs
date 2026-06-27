use crate::api::RawgClient;
use crate::db::Database;
use crate::models::error::AppError;
use crate::models::{Category, GameEntry, GameEntryDetail, RawgGameData, SearchResult};
use crate::services::category_service::CategoryService;
use crate::services::game_entry_service::GameEntryService;
use crate::services::launch_service::GameLaunchService;
use crate::services::metadata_service::SourceService;
use crate::services::tag_service::TagService;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
    pub rawg_client: RawgClient,
}

#[tauri::command]
pub fn get_categories(state: State<'_, AppState>) -> Result<Vec<Category>, AppError> {
    let db = state.db.lock().unwrap();
    let service = CategoryService::new(&db);
    service.list_all()
}

#[tauri::command]
pub fn create_category(state: State<'_, AppState>, name: String) -> Result<Category, AppError> {
    let db = state.db.lock().unwrap();
    let service = CategoryService::new(&db);

    service.create(&name)
}

#[tauri::command]
pub fn delete_category(state: State<'_, AppState>, id: i64, confirm: bool) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = CategoryService::new(&db);

    service.delete(id, confirm)
}

#[tauri::command]
pub async fn search_games(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, AppError> {
    state.rawg_client.search_games(&query).await
}

#[tauri::command]
pub fn add_game_from_search(
    state: State<'_, AppState>,
    rawg_data: RawgGameData,
    category_id: i64,
    tag_list: Vec<String>,
    source: Option<&str>,
) -> Result<GameEntryDetail, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.add_from_search(&rawg_data, category_id, &tag_list, source)
}

#[tauri::command]
pub fn add_game_manually(
    state: State<'_, AppState>,
    name: String,
    category_id: i64,
    tag_list: Vec<String>,
    source: Option<&str>,
) -> Result<GameEntryDetail, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.add_manually(&name, category_id, &tag_list, source)
}

#[tauri::command]
pub fn move_to_category(
    state: State<'_, AppState>,
    entry_id: i64,
    dest_category_id: i64,
) -> Result<GameEntry, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.move_to_category(entry_id, dest_category_id)
}

#[tauri::command]
pub fn delete_game_entry(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.delete(entry_id)
}

#[tauri::command]
pub fn update_launch_path(
    state: State<'_, AppState>,
    entry_id: i64,
    path: Option<String>,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameLaunchService::new(&db);

    service.set_path(entry_id, path.as_deref())
}

#[tauri::command]
pub fn launch_game(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameLaunchService::new(&db);

    service.launch(entry_id)
}

#[tauri::command]
pub fn update_last_played(
    state: State<'_, AppState>,
    entry_id: i64,
    date: Option<String>,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = SourceService::new(&db);

    service.update_last_played(entry_id, date.as_deref())
}

#[tauri::command]
pub fn add_tag(state: State<'_, AppState>, entry_id: i64, tag: String) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = TagService::new(&db);

    service.add(entry_id, &tag)
}

#[tauri::command]
pub fn remove_tag(state: State<'_, AppState>, entry_id: i64, tag: String) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = TagService::new(&db);

    service.remove(entry_id, &tag)
}

#[tauri::command]
pub fn update_source(
    state: State<'_, AppState>,
    entry_id: i64,
    source: Option<String>,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = SourceService::new(&db);

    service.update_source(entry_id, source.as_deref())
}

#[tauri::command]
pub fn get_game_entry(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<GameEntryDetail, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.get_detail(entry_id)
}

#[tauri::command]
pub fn get_entries_by_category(
    state: State<'_, AppState>,
    category_id: i64,
) -> Result<Vec<GameEntry>, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);

    service.list_by_category(category_id)
}

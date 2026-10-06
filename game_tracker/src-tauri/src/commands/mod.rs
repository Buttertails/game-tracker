use std::sync::Mutex;

use crate::api::IgdbClient;
use crate::db::Database;
use crate::models::error::AppError;
use crate::models::{
    GameEntryDetail, ManualEntryInput, OwnershipStatus, SearchResult, Shelf, ShelfEntries, ShelfSummary, SmartFillSuggestion, TimestampedNote,
};
use crate::services::game_entry_service::GameEntryService;
use crate::services::launch_service::GameLaunchService;
use crate::services::note_service::NoteService;
use crate::services::shelf_service::ShelfService;
use crate::services::smart_fill_service::SmartFillService;
use crate::services::status_transition_service::StatusTransitionService;
use crate::services::tag_service::TagService;
use tauri::State;
use chrono::{DateTime, Utc};

pub struct AppState {
    pub db: Mutex<Database>,
    pub igdb_client: IgdbClient,
}

#[tauri::command]
pub fn get_shelves(state: State<'_, AppState>) -> Result<Vec<ShelfSummary>, AppError> {
    let db = state.db.lock().unwrap();
    let service = ShelfService::new(&db);
    service.list_all()
}

#[tauri::command]
pub fn create_shelf(state: State<'_, AppState>, name: String) -> Result<Shelf, AppError> {
    let db = state.db.lock().unwrap();
    let service = ShelfService::new(&db);
    service.create(&name)
}

#[tauri::command]
pub fn rename_shelf(
    state: State<'_, AppState>,
    shelf_id: i64,
    new_name: String,
) -> Result<Shelf, AppError> {
    let db = state.db.lock().unwrap();
    let service = ShelfService::new(&db);
    service.rename(shelf_id, &new_name)
}

#[tauri::command]
pub fn delete_shelf(
    state: State<'_, AppState>,
    shelf_id: i64,
    confirmed: bool,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = ShelfService::new(&db);
    service.delete(shelf_id, confirmed)
}

#[tauri::command]
pub async fn search_games(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, AppError> {
    state.igdb_client.search_games(&query).await
}

#[tauri::command]
pub fn add_game_from_search(
    state: State<'_, AppState>,
    search_result: SearchResult,
    shelf_id: i64,
    source: Option<String>,
    launch_path: Option<String>,
    ownership_status: OwnershipStatus,
    tag_list: Vec<String>,
) -> Result<GameEntryDetail, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.add_from_search(
        &search_result,
        shelf_id,
        source.as_deref(),
        launch_path.as_deref(),
        ownership_status,
        &tag_list,
    )
}

#[tauri::command]
pub fn add_game_manually(
    state: State<'_, AppState>,
    input: ManualEntryInput,
    tag_list: Vec<String>,
) -> Result<GameEntryDetail, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.add_manually(input, &tag_list)
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
pub fn get_entries_by_shelf(
    state: State<'_, AppState>,
    shelf_id: i64,
) -> Result<ShelfEntries, AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.list_by_shelf(shelf_id)
}

#[tauri::command]
pub fn delete_game_entry(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.delete(entry_id)
}

#[tauri::command]
pub fn update_ownership_status(
    state: State<'_, AppState>,
    entry_id: i64,
    ownership_status: OwnershipStatus,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.update_ownership_status(entry_id, ownership_status)
}

#[tauri::command]
pub fn move_to_in_progress(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = StatusTransitionService::new(&db);
    service.move_to_in_progress(entry_id)
}

#[tauri::command]
pub fn move_to_completed(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = StatusTransitionService::new(&db);
    service.move_to_completed(entry_id)
}

#[tauri::command]
pub fn move_to_backlog(state: State<'_, AppState>, entry_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = StatusTransitionService::new(&db);
    service.move_to_backlog(entry_id)
}

#[tauri::command]
pub fn undo_completion_to_in_progress(
    state: State<'_, AppState>,
    entry_id: i64,
    confirmed: bool,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = StatusTransitionService::new(&db);
    service.undo_to_in_progress(entry_id, confirmed)
}

#[tauri::command]
pub fn undo_completion_to_backlog(
    state: State<'_, AppState>,
    entry_id: i64,
    confirmed: bool,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = StatusTransitionService::new(&db);
    service.undo_to_backlog(entry_id, confirmed)
}

#[tauri::command]
pub fn get_smart_fill_suggestions(
    state: State<'_, AppState>,
    shelf_id: i64,
) -> Result<Vec<SmartFillSuggestion>, AppError> {
    let db = state.db.lock().unwrap();
    let service = SmartFillService::new(&db);
    service.get_suggestions(shelf_id)
}

#[tauri::command]
pub fn accept_smart_fill(state: State<'_, AppState>, entry_ids: Vec<i64>) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = SmartFillService::new(&db);
    service.accept(&entry_ids)
}

#[tauri::command]
pub fn reroll_suggestion(
    state: State<'_, AppState>,
    shelf_id: i64,
    entry_to_replace_id: i64,
    kept_suggestion_ids: Vec<i64>,
) -> Result<SmartFillSuggestion, AppError> {
    let db = state.db.lock().unwrap();
    let service = SmartFillService::new(&db);
    service.reroll(shelf_id, entry_to_replace_id, &kept_suggestion_ids)
}

#[tauri::command]
pub fn add_note(
    state: State<'_, AppState>,
    entry_id: i64,
    text: String,
) -> Result<TimestampedNote, AppError> {
    let db = state.db.lock().unwrap();
    let service = NoteService::new(&db);
    service.add(entry_id, &text)
}

#[tauri::command]
pub fn delete_note(state: State<'_, AppState>, note_id: i64) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = NoteService::new(&db);
    service.delete(note_id)
}

#[tauri::command]
pub fn get_notes(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<Vec<TimestampedNote>, AppError> {
    let db = state.db.lock().unwrap();
    let service = NoteService::new(&db);
    service.list_by_entry(entry_id)
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
pub fn update_entry_metadata(
    state: State<'_, AppState>,
    entry_id: i64,
    genre: Option<String>,
    avg_playtime_hours: Option<i64>,
    release_date: Option<DateTime<Utc>>,
    source: Option<String>,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    let service = GameEntryService::new(&db);
    service.update_entry_metadata(
        entry_id,
        genre.as_deref(),
        avg_playtime_hours,
        release_date,
        source.as_deref(),
    )
}

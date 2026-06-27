use game_tracker_lib::{
    db::Database,
    models::error::AppError,
    services::{
        category_service::CategoryService, game_entry_service::GameEntryService,
        launch_service::GameLaunchService,
    },
};
use proptest::prelude::*;
use tempfile::NamedTempFile;

fn valid_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,30}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn setup() -> Database {
    Database::new_in_memory().unwrap()
}

proptest! {
    // Property 17: Launch path round-trip storage
    #[test]
    fn launch_path_round_trip(name in valid_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let entry_service = GameEntryService::new(&db);
        let entry = entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        let launch_service = GameLaunchService::new(&db);

        // Create a real temp file so validate_path passes
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Set the path
        launch_service.set_path(entry.entry_id, Some(path)).unwrap();

        // Retrieve and verify round-trip
        let detail = entry_service.get_detail(entry.entry_id).unwrap();
        prop_assert_eq!(detail.launch_path, Some(path.to_string()));
    }

    // Clearing the launch path removes it
    #[test]
    fn clearing_launch_path(name in valid_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let entry_service = GameEntryService::new(&db);
        let entry = entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        let launch_service = GameLaunchService::new(&db);

        // Set a path first
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        launch_service.set_path(entry.entry_id, Some(path)).unwrap();

        // Clear it
        launch_service.set_path(entry.entry_id, None).unwrap();

        // Verify it's gone
        let detail = entry_service.get_detail(entry.entry_id).unwrap();
        prop_assert!(detail.launch_path.is_none());
    }

    // Invalid path is rejected
    #[test]
    fn invalid_path_rejected(name in valid_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let entry_service = GameEntryService::new(&db);
        let entry = entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        let launch_service = GameLaunchService::new(&db);

        let result = launch_service.set_path(entry.entry_id, Some("C:\\nonexistent\\fake_game.exe"));
        prop_assert!(matches!(result, Err(AppError::LaunchPathNotFound(_))));
    }

    // No launch path returns error
    #[test]
    fn launch_without_path_errors(name in valid_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let entry_service = GameEntryService::new(&db);
        let entry = entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        let launch_service = GameLaunchService::new(&db);

        let result = launch_service.launch(entry.entry_id);
        prop_assert!(matches!(result, Err(AppError::NoLaunchPath)));
    }
}

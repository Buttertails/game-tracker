use game_tracker_lib::db::Database;
use game_tracker_lib::models::error::AppError;
use game_tracker_lib::models::RawgGameData;
use game_tracker_lib::services::category_service::CategoryService;
use game_tracker_lib::services::game_entry_service::GameEntryService;
use proptest::prelude::*;

fn valid_game_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,200}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn rawg_game_data() -> impl Strategy<Value = RawgGameData> {
    (
        1..100000i64,
        "[a-zA-Z0-9 _-]{1,50}",
        proptest::option::of("[0-9]{4}-[0-9]{2}-[0-9]{2}"),
        proptest::option::of(0.0f64..5.0),
        proptest::option::of(0i32..100),
    )
        .prop_map(|(id, name, released, rating, metacritic)| RawgGameData {
            id,
            name,
            released,
            rating,
            metacritic,
            platforms: None,
            genres: None,
            background_image: None,
            esrb_rating: None,
            extra: serde_json::Value::Object(serde_json::Map::new()),
        })
}

fn whitespace_only() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::char::range(' ', ' '), 1..10)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

fn setup() -> Database {
    let db = Database::new_in_memory().unwrap();
    db
}

proptest! {
    #[test]
    fn valid_names_are_accepted(name in valid_game_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let category_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let result = service.add_manually(&name, category_id, &[], None);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn empty_names_are_rejected(name in whitespace_only()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let category_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let result = service.add_manually(&name, category_id, &[], None);
        prop_assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[test]
    fn add_game_from_search_stores_complete(data in rawg_game_data()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let category_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let result = service.add_from_search(&data, category_id, &[], None);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn duplicate_names_are_rejected(name in valid_game_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let category_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        service.add_manually(&name, category_id, &[], None).unwrap();

        let result = service.check_duplicate_by_name(&name).unwrap();
        prop_assert!(result.is_some());
    }

    #[test]
    fn moving_entry_preserves_metadata(
        name in valid_game_name(),
        source in proptest::option::of("[a-zA-Z ]{1,50}")
    ) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_a = cats[0].category_id;
        let cat_b = cats[1].category_id;

        let service = GameEntryService::new(&db);
        let tags = vec!["RPG".to_string(), "Indie".to_string()];
        let entry = service.add_manually(&name, cat_a, &tags, source.as_deref()).unwrap();

        let before = service.get_detail(entry.entry_id).unwrap();
        service.move_to_category(entry.entry_id, cat_b).unwrap();
        let after = service.get_detail(entry.entry_id).unwrap();

        prop_assert_eq!(&before.name, &after.name);
        prop_assert_eq!(&before.addition_date, &after.addition_date);
        prop_assert_eq!(&before.tags, &after.tags);
        prop_assert_eq!(&before.source, &after.source);
        prop_assert_eq!(&before.last_played, &after.last_played);
        prop_assert_eq!(&before.launch_path, &after.launch_path);
        prop_assert_eq!(&before.stored_api_data, &after.stored_api_data);
        prop_assert_eq!(after.category_id, cat_b);
    }

    #[test]
    fn move_to_same_category_is_noop(name in valid_game_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let entry = service.add_manually(&name, cat_id, &[], None).unwrap();

        let result = service.move_to_category(entry.entry_id, cat_id).unwrap();

        prop_assert_eq!(result.category_id, cat_id);
    }

    #[test]
    fn delete_entry_removes_it(name in valid_game_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let entry = service.add_manually(&name, cat_id, &[], None).unwrap();
        service.delete(entry.entry_id).unwrap();

        let result = service.get_detail(entry.entry_id);
        prop_assert!(result.is_err());
    }

    #[test]
    fn manual_entry_has_no_api_data(name in valid_game_name()) {
        let db = setup();
        let cat_service = CategoryService::new(&db);
        cat_service.initialize_presets().unwrap();

        let cats = cat_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        let service = GameEntryService::new(&db);
        let entry = service.add_manually(&name, cat_id, &[], None).unwrap();

        prop_assert!(entry.stored_api_data.is_none());
    }
}

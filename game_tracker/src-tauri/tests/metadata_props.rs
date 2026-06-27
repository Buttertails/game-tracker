use game_tracker_lib::{
    db::Database,
    models::error::AppError::{self},
    services::{
        category_service::CategoryService, game_entry_service::GameEntryService,
        metadata_service::SourceService,
    },
};

use chrono::{Local, NaiveDate};
use proptest::prelude::*;

fn valid_source() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,100}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn valid_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,30}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn whitespace_only() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::char::range(' ', ' '), 1..10)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

fn large_source_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{101,150}"
}

fn valid_past_date() -> impl Strategy<Value = String> {
    (2000i32..=2025, 1u32..=12, 1u32..=28)
        .prop_map(|(y, m, d)| format!("{:04}-{:02}-{:02}", y, m, d))
}

fn future_date() -> impl Strategy<Value = String> {
    (2027i32..=2030, 1u32..=12, 1u32..=28)
        .prop_map(|(y, m, d)| format!("{:04}-{:02}-{:02}", y, m, d))
}

fn setup() -> Database {
    let db = Database::new_in_memory().unwrap();
    db
}

proptest! {
    #[test]
    fn valid_source_is_added(source in valid_source(), name in valid_name()) {
        // Check that a valid source is added successfully

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // add source to entry and check validity
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None);
        prop_assert!(&entry.is_ok());
        let entry_unwrap = entry.unwrap();

        // update source and check validity
        let update_result = metadata_service.update_source(entry_unwrap.entry_id, Some(&source));
        prop_assert_eq!(update_result.unwrap(), ());

        // Check if entry source was updated
        let updated_entry = game_entry_service.get_detail(entry_unwrap.entry_id).unwrap();
        prop_assert_ne!(entry_unwrap.source, updated_entry.source);
    }

    #[test]
    fn empty_source_clears(valid_source in valid_source(), empty_source in whitespace_only(), name in valid_name()) {
        // Check that a whitespace-only source clears the field

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // add source to entry and check validity
        let entry = game_entry_service.add_manually(&name, cat_id, &[], Some(&valid_source));
        prop_assert!(entry.is_ok());
        let entry_unwrap = entry.unwrap();

        // update source and check validity
        let update_result = metadata_service.update_source(entry_unwrap.entry_id, Some(&empty_source));
        prop_assert_eq!(update_result.unwrap(), ());

        // fetch entry and check source is None
        let update_entry = game_entry_service.get_detail(entry_unwrap.entry_id).unwrap();
        prop_assert!(matches!(update_entry.source, None));
    }

    #[test]
    fn large_source_is_rejected(large_source in large_source_name(), name in valid_name()) {
        // Check that a source larger than character limit is rejected

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // add source to entry and check validity
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None);
        prop_assert!(entry.is_ok());
        let entry_unwrap = entry.unwrap();

        // update source and check validity
        let update_result = metadata_service.update_source(entry_unwrap.entry_id, Some(&large_source));
        prop_assert!(matches!(update_result, Err(AppError::ValidationError(_))));
    }

    #[test]
    fn valid_past_date_is_added(valid_date in valid_past_date(), name in valid_name()) {
        // Check that a valid date is added successfully

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // update last played date and check validity
        let update_result = metadata_service.update_last_played(entry.entry_id, Some(&valid_date));
        prop_assert_eq!(update_result.unwrap(), ());

        // Check if entry source was updated
        let updated_entry = game_entry_service.get_detail(entry.entry_id).unwrap();
        prop_assert_ne!(entry.last_played, updated_entry.last_played);
    }

    #[test]
    fn future_date_is_rejected(future_date in future_date(), name in valid_name()) {
        // Check that a future date is rejected

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // update last played date and check validity
        let update_result = metadata_service.update_last_played(entry.entry_id, Some(&future_date));
        prop_assert!(matches!(update_result, Err(AppError::InvalidDate(_))));
    }

    #[test]
    fn empty_last_played_date_clears(valid_date in valid_past_date(), empty_date in whitespace_only(), name in valid_name()) {
        // Check that a whitespace-only date clears the field

        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let metadata_service = SourceService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry with valid date and check validity
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();
        metadata_service.update_last_played(entry.entry_id, Some(&valid_date)).unwrap();
        let after_set = game_entry_service.get_detail(entry.entry_id).unwrap();
        prop_assert!(after_set.last_played.is_some());

        // update last played date with empty date and check validity
        metadata_service.update_last_played(entry.entry_id, Some(&empty_date)).unwrap();

        // Check if entry last played was updated
        let after_clear = game_entry_service.get_detail(entry.entry_id).unwrap();
        prop_assert!(after_clear.last_played.is_none());
    }
}

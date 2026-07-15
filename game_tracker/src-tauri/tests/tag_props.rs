use game_tracker_lib::{
    db::Database,
    models::error::AppError::{self},
    services::{
        category_service::CategoryService, game_entry_service::GameEntryService,
        tag_service::TagService,
    },
};
use proptest::prelude::*;

fn valid_tag() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,30}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn valid_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,30}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
}

fn whitespace_only() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::char::range(' ', ' '), 1..10)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

fn large_tag_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{31,50}"
}

fn setup() -> Database {
    let db = Database::new_in_memory().unwrap();
    db
}

proptest! {
    #[test]
    fn valid_tags_are_accepted(tag in valid_tag(), name in valid_name()) {
        // Check that a valid tag is added successfully
        // Setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let service = TagService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // add tag
        let result = service.add(entry.entry_id, &tag);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn empty_tags_are_rejected(tag in whitespace_only(), name in valid_name()) {
        // Check an empty tag is rejected

        // setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let service = TagService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // add tag
        let result = service.add(entry.entry_id, &tag);
        prop_assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[test]
    fn large_tags_are_rejected(tag in large_tag_name(), name in valid_name()) {
        // Check a tag larger than character limit is rejected

        // setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let service = TagService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // add tag
        let result = service.add(entry.entry_id, &tag);
        prop_assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[test]
    fn duplicate_tags_are_rejected(tag in valid_tag(), name in valid_name()) {
        // Check duplicate tags are not added

        // setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let service = TagService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // add tag and verify sucessful addition
        let add_result = service.add(entry.entry_id, &tag);
        prop_assert!(add_result.is_ok());

        // attempt to add duplicate tag and verify rejection
        let upper = tag.to_uppercase();
        let duplicate_result = service.add(entry.entry_id, &upper);
        prop_assert!(matches!(duplicate_result, Err(AppError::DuplicateTag(_))));
    }

    #[test]
    fn tag_is_removed_from_entry(tag in valid_tag(), name in valid_name()) {
        // Check when a tag is removed it is removed from the game entry

        // setup in-memory database connection
        let db = setup();

        // setup services
        let game_entry_service = GameEntryService::new(&db);
        let category_service = CategoryService::new(&db);
        let service = TagService::new(&db);

        // setup categories
        category_service.initialize_presets().unwrap();
        let cats = category_service.list_all().unwrap();
        let cat_id = cats[0].category_id;

        // setup entry
        let mut entry = game_entry_service.add_manually(&name, cat_id, &[], None).unwrap();

        // add tag and verify sucessful addition
        let add_result = service.add(entry.entry_id, &tag);
        prop_assert!(add_result.is_ok());

        // delete tag and verify successful deletion
        let delete_result = service.remove(entry.entry_id, &tag);
        prop_assert!(delete_result.is_ok());

        // verify tag is removed from entry tag list
        entry = game_entry_service.get_detail(entry.entry_id).unwrap();
        let tag_list = entry.tags;
        prop_assert_eq!(tag_list.iter().find(|&t| *t == tag), None);
    }
}

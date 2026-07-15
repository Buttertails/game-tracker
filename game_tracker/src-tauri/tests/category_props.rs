use game_tracker_lib::db::Database;
use game_tracker_lib::models::error::AppError;
use game_tracker_lib::services::category_service::CategoryService;
use proptest::prelude::*;

// Generates random valid category names (1-50 chars after trim, at least non-whitespace)
fn valid_category_name() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 _-]{1,50}".prop_filter("must not be all whitespace", |s| !s.trim().is_empty())
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
    fn valid_names_are_accepted(name in valid_category_name()) {
        let db = setup();
        let service = CategoryService::new(&db);
        let result = service.create(&name);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn empty_names_are_rejected(name in whitespace_only()) {
        let db = setup();
        let service = CategoryService::new(&db);
        let result = service.create(&name);
        prop_assert!(matches!(result, Err(AppError::ValidationError(_))));
    }

    #[test]
    fn duplicate_names_are_rejected(name in valid_category_name()) {
        let db = setup();
        let service = CategoryService::new(&db);

        service.create(&name).unwrap();

        let upper = name.to_uppercase();
        let result = service.create(&upper);
        prop_assert!(matches!(result, Err(AppError::DuplicateCategory(_))));
    }

    #[test]
    fn delete_empty_category_removes_it(name in valid_category_name()) {
        let db = setup();
        let service = CategoryService::new(&db);

        let cat = service.create(&name).unwrap();
        let result = service.delete(cat.category_id, false);
        prop_assert!(result.is_ok());

        let all = service.list_all().unwrap();
        prop_assert!(all.iter().all(|c| c.category_id != cat.category_id));
    }
}

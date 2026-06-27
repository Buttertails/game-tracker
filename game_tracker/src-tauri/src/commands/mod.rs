use std::fs::Metadata;

use crate::api::RawgClient;
use crate::db::Database;
use crate::services::category_service::CategoryService;
use crate::services::game_entry_service::GameEntryService;
use crate::services::launch_service::GameLaunchService;
struct AppState {
    db: Database,
    rawg_client: RawgClient,
    category_service: CategoryService,
    game_entry_service: GameEntryService,
    launch_service: GameLaunchService,
    metadata_service: SourceService,
    tag_service: TagService,
}

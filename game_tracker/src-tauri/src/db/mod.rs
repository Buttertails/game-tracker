pub mod game_entries;
pub mod notes;
pub mod shelves;
pub mod tags;

use rusqlite::Connection;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS shelves (
                shelf_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS game_entries (
                entry_id INTEGER PRIMARY KEY AUTOINCREMENT,
                shelf_id INTEGER NOT NULL,
                igdb_id: INTEGER,
                name TEXT NOT NULL,
                status INTEGER NOT NULL,
                genre TEXT,
                avg_playtime_hours INTEGER,
                release_date INTEGER,
                source TEXT,
                launch_path TEXT,
                ownership_status INTEGER,
                started_at TEXT,
                completed_at TEXT,
                addition_date TEXT NOT NULL DEFAULT (datetime('now')),
                last_played TEXT,
                background_image TEXT,
                stored_api_data TEXT,
                api_data_version INTEGER NOT NULL DEFAULT 2,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),

                FOREIGN KEY (shelf_id) REFERENCES shelves(shelf_id) ON DELETE CASCADE,
                FOREIGN KEY (status) REFERENCES statuses(status_id) ON DELETE RESTRICT,
                FOREIGN KEY (ownership_status) REFERENCES ownership_statuses(ownership_status_id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS game_notes (
                note_id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (entry_id) REFERENCES game_entries(entry_id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS game_tags (
                tag_id INTEGER PRIMARY KEY AUTOINCREMENT, 
                entry_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                FOREIGN KEY (entry_id) REFERENCES game_entries(entry_id) ON DELETE CASCADE,
                UNIQUE(entry_id, tag COLLATE NOCASE)
            );

            CREATE TABLE IF NOT EXISTS statuses (
                status_id INTEGER PRIMARY KEY AUTOINCREMENT,
                status_type TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS ownership_statuses (
                ownership_status_id INTEGER PRIMARY KEY AUTOINCREMENT,
                ownership_status_type TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_game_entries_shelf ON game_entries(shelf_id);
            CREATE INDEX IF NOT EXISTS idx_game_entries_shelf_status ON game_entries(shelf_id, status);
            CREATE INDEX IF NOT EXISTS idx_game_entries_name ON game_entries(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_game_notes_entry ON game_notes(entry_id);
            CREATE INDEX IF NOT EXISTS idx_game_tags_entry ON game_tags(entry_id);

            INSERT INTO statuses(status_type) SELECT 'backlog' WHERE NOT EXISTS( SELECT 1 FROM statuses WHERE status_type = 'backlog');
            INSERT INTO statuses(status_type) SELECT 'in_progress' WHERE NOT EXISTS (SELECT 1 FROM statuses WHERE status_type = 'in_progress');
            INSERT INTO statuses(status_type) SELECT 'completed' WHERE NOT EXISTS (SELECT 1 FROM statuses WHERE status_type = 'completed');

            INSERT INTO ownership_statuses(ownership_status_type) SELECT 'installed' WHERE NOT EXISTS (SELECT 1 FROM ownership_statuses WHERE ownership_status_type = 'installed');
            INSERT INTO ownership_statuses(ownership_status_type) SELECT 'not_installed' WHERE NOT EXISTS (SELECT 1 FROM ownership_statuses WHERE ownership_status_type = 'not_installed');
            INSERT INTO ownership_statuses(ownership_status_type) SELECT 'wishlisted' WHERE NOT EXISTS (SELECT 1 FROM ownership_statuses WHERE ownership_status_type = 'wishlisted');
            "
        )
    }
}

-- Test data for Game Shelf Tracker
-- Run this after creating the schema to populate with sample data

-- Shelves
INSERT INTO shelves (name) VALUES ('Solo');
INSERT INTO shelves (name) VALUES ('Me + Roommate');
INSERT INTO shelves (name) VALUES ('Friend Group');

-- Game entries on Solo shelf (shelf_id = 1)
-- Backlog entries (status = 1)
INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'Hollow Knight', 1, 'Metroidvania', 2, 2017, 3, 'Steam', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'Celeste', 1, 'Platformer', 1, 2018, 3, 'Steam', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'Final Fantasy VII', 1, 'RPG', 3, 1997, 1, 'Emulated - PS1', 2);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'Persona 5 Royal', 1, 'RPG', 3, 2020, 3, 'Steam', 2);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'Hades', 1, 'Action', 2, 2020, 3, 'Steam', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, launch_path, ownership_status)
VALUES (1, 'Portal 2', 1, 'Puzzle', 1, 2011, 2, 'Steam', 'C:\Program Files\Steam\steamapps\common\Portal 2\portal2.exe', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (1, 'The Witness', 1, 'Puzzle', 2, 2016, 3, 'Epic Games', 3);

-- No metadata (should be excluded from Smart Fill)
INSERT INTO game_entries (shelf_id, name, status, ownership_status)
VALUES (1, 'Some Indie Game', 1, 2);

-- In Progress entries (status = 2)
INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at)
VALUES (1, 'Elden Ring', 2, 'Action RPG', 3, 2022, 3, 'Steam', 1, '2025-01-15 10:00:00');

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at)
VALUES (1, 'Tetris Effect', 2, 'Puzzle', 1, 2018, 3, 'Epic Games', 1, '2025-02-01 14:30:00');

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at)
VALUES (1, 'Civilization VI', 2, 'Strategy', 3, 2016, 3, 'Steam', 1, '2025-02-10 09:00:00');

-- Completed entries (status = 3)
INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at, completed_at)
VALUES (1, 'Outer Wilds', 3, 'Adventure', 2, 2019, 3, 'Steam', 1, '2024-11-01 08:00:00', '2024-11-20 22:00:00');

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at, completed_at)
VALUES (1, 'Super Metroid', 3, 'Metroidvania', 1, 1994, 1, 'Emulated - SNES', 1, '2024-12-01 10:00:00', '2024-12-05 18:00:00');

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at, completed_at)
VALUES (1, 'Dark Souls', 3, 'Action RPG', 3, 2011, 2, 'Steam', 1, '2024-09-15 12:00:00', '2024-10-30 20:00:00');

-- Game entries on Me + Roommate shelf (shelf_id = 2)
INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at)
VALUES (2, 'Elden Ring', 2, 'Action RPG', 3, 2022, 3, 'Steam', 1, '2025-01-20 19:00:00');

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (2, 'It Takes Two', 1, 'Action Adventure', 2, 2021, 3, 'Steam', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (2, 'Divinity Original Sin 2', 1, 'RPG', 3, 2017, 3, 'Steam', 1);

-- Game entries on Friend Group shelf (shelf_id = 3)
INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status)
VALUES (3, 'Lethal Company', 1, 'Horror', 2, 2023, 3, 'Steam', 1);

INSERT INTO game_entries (shelf_id, name, status, genre, length_category, release_year, era, source, ownership_status, started_at)
VALUES (3, 'Deep Rock Galactic', 2, 'FPS', 2, 2020, 3, 'Steam', 1, '2025-01-05 20:00:00');

-- Notes (for entries with IDs that will be 9=Elden Ring Solo, 10=Tetris Effect)
INSERT INTO game_notes (entry_id, text, created_at) VALUES (9, 'Just beat Margit. Took about 10 tries. This game is brutal.', '2025-01-16 22:00:00');
INSERT INTO game_notes (entry_id, text, created_at) VALUES (9, 'Found the Academy. The map is massive.', '2025-01-20 21:00:00');
INSERT INTO game_notes (entry_id, text, created_at) VALUES (9, 'Beat Rennala. Magic build is working well.', '2025-01-28 23:30:00');
INSERT INTO game_notes (entry_id, text, created_at) VALUES (10, 'Zone mode is incredibly relaxing.', '2025-02-02 22:00:00');

-- Tags
INSERT INTO game_tags (entry_id, tag) VALUES (9, 'Soulslike');
INSERT INTO game_tags (entry_id, tag) VALUES (9, 'Open World');
INSERT INTO game_tags (entry_id, tag) VALUES (9, 'Multiplayer');
INSERT INTO game_tags (entry_id, tag) VALUES (1, 'Indie');
INSERT INTO game_tags (entry_id, tag) VALUES (1, 'Metroidvania');
INSERT INTO game_tags (entry_id, tag) VALUES (2, 'Indie');
INSERT INTO game_tags (entry_id, tag) VALUES (2, 'Precision Platformer');
INSERT INTO game_tags (entry_id, tag) VALUES (5, 'Roguelite');
INSERT INTO game_tags (entry_id, tag) VALUES (12, 'GOTY');
INSERT INTO game_tags (entry_id, tag) VALUES (14, 'Soulslike');

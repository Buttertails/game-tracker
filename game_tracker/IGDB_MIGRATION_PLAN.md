# IGDB Migration Plan

Consolidated plan for switching the game data source from RAWG to IGDB and
cleaning up the data models. Work bottom-up: models first, then DB, then API
client, then service/commands, then frontend.

## Background / key facts about IGDB

- Endpoint: `POST https://api.igdb.com/v4/games` (not GET with query params).
- Auth: two headers, `Client-ID` and `Authorization: Bearer <token>`, where the
  token is a Twitch OAuth **app access token** (IGDB is owned by Twitch). Tokens
  last ~60 days and must be cached/refreshed.
- Body: an "Apicalypse" plain-text query. You only get fields you explicitly name.
- Nested expansion resolves related data in the SAME call:
  `cover.image_id`, `genres.name`, `platforms.name`. No per-result requests.
- Cover: IGDB returns `cover.image_id` (a bare id), NOT a URL. Build the URL:
  `https://images.igdb.com/igdb/image/upload/t_cover_big/{image_id}.jpg`
- `first_release_date` is a Unix timestamp (i64), not a date string.
- `rating` / `aggregated_rating` are on a 0-100 scale (RAWG rating was 0-5).
- There is NO `playtime` field. Time-to-beat lives at a separate endpoint
  (`/v4/game_time_to_beat`). See "Length category" below.

## Decisions

1. **Delete `RawgGameData` and all `Rawg*` structs.** They were transport structs
   that leaked into the service layer. Replace with a private `IgdbGameRaw` in the
   api module that is deserialized then mapped and thrown away.
2. **Keep `SearchResult` distinct from `GameEntry`.** They model different life
   stages: `SearchResult` = "a game that exists in the world" (identity = external
   `igdb_id`, transient, no shelf/status). `GameEntry` = "a game I placed on a
   shelf and track" (identity = `entry_id`, persisted, has status/ownership/notes).
   Merging them creates Option-soup.
3. **Keep `GameEntry` (list) vs `GameEntryDetail` (detail) split.** The blob
   (`stored_api_data`) is the justification: don't load it for a 50-game grid, do
   load it for one detail view.
4. **`stored_api_data` = Option B.** Store the minimal IGDB object fetched at
   search time. Do NOT pad the query with unused fields now. When the enhanced
   detail view is built later, add fields to the query then and re-fetch by id.
5. **Persist `igdb_id` as a real column** on `game_entries`. This is the true
   backup mechanism: as long as the id is stored, anything IGDB has can be
   re-fetched later. (Currently NO external id is stored at all.)
6. **Stamp `api_data_version`** so the future lazy-refresh can tell thin blobs
   from fat ones without inspecting individual keys.

## Store raw values, derive categories on demand

Normalization decision: store the source-of-truth value and derive the
categorization when needed (Smart Fill + frontend). Removes a whole class of
drift bugs (e.g. release_year edited but era column left stale).

- **Era:** derived purely from `release_year` via `derive_era`. Store
  `release_year` only; **drop the `era` column** (and the `eras` lookup table +
  FK). Smart Fill already derives era live at scoring time and never trusted the
  stored column, so nothing is lost.
- **Length:** derived from average playtime via `derive_length_category`. Store
  `avg_playtime_hours` (raw); **drop the `length_category` column** (and the
  `length_categories` lookup table + FK). Derive category on demand.

### Playtime source: two requests per search (CHOSEN)

IGDB core `/v4/games` has no playtime field. Time-to-beat is a separate endpoint.
So each search is **2 requests total** (batched, NOT per-game / not N+1):

1. `POST /v4/games` (search) -> ~20 results, each with `igdb_id`.
2. `POST /v4/game_time_to_beat` for the whole batch of ids:
   `fields game_id, normally, completely; where game_id = (id1,...,idN); limit 20;`
3. Join playtime onto each result by `game_id`.

Gotchas:
- Values are in **seconds**; convert to hours (`/ 3600`) before storing, since
  `derive_length_category` takes hours.
- Coverage is incomplete (older/obscure games often missing) -> playtime is
  genuinely `Option`. When absent, length is underivable; user sets it manually
  (existing ⚠ badge + Smart Fill eligibility filter already handle this).

## Future work (NOT now): enhanced detail view + lazy refresh

- Add an `enrich_entry(entry_id)` fn on `GameEntryService`: fetch by `igdb_id`,
  overwrite blob, bump `api_data_version`.
- Detail command calls it conditionally: `if stored_version < REQUIRED_VERSION`.
- Must degrade gracefully if IGDB is unreachable (open view from typed columns,
  skip enhanced section; reuse `AppError::ApiUnavailable`).
- Same fn also powers a manual "refresh metadata" button for free.

---

## Change checklist (bottom-up)

### 1. Models (`src-tauri/src/models/mod.rs`)  [DONE]
- [x] Finalize `SearchResult` (neutral, carries everything for an entry + raw blob,
      including `avg_playtime_hours`).
- [x] Delete `RawgGameData`, `RawgPlatform`, `RawgPlatformInner`, `RawgGenre`,
      `RawgEsrbRating`, and the broken `IGDBGameData` struct.
- [x] Add `igdb_id: Option<i64>` to `GameEntry` and `GameEntryDetail`.
- [ ] Change `GameEntry`/`GameEntryDetail` to store raw values: replace
      `length_category: Option<LengthCategory>` with `avg_playtime_hours: Option<i64>`;
      keep `release_year`; remove any stored `era`. Derive both on demand.

### 2. DB (`src-tauri/src/db/`)
- [ ] Add columns: `igdb_id INTEGER`, `api_data_version INTEGER`,
      `avg_playtime_hours INTEGER`. Drop derived columns `era` and
      `length_category` (+ their lookup tables/FKs: `eras`, `length_categories`).
- [ ] Migration is `CREATE TABLE IF NOT EXISTS`, so add guarded
      `ALTER TABLE ... ADD COLUMN` migrations for existing DBs (SQLite can't
      easily drop columns/FKs in-place — for the removed `era`/`length_category`
      columns, either leave them unused or do a table-rebuild migration).
- [ ] Update `insert_entry` signature + INSERT: drop `era`/`length_category`,
      add `igdb_id`, `api_data_version`, `avg_playtime_hours`.
- [ ] Update row-mapping in `get_entry` / `get_entry_detail` / list fns.

### 3. API client (`src-tauri/src/api/`)
- [ ] Replace `RawgClient` with `IgdbClient` holding `client_id`, `access_token`
      (cached), `client`.
- [ ] Twitch token flow: POST `https://id.twitch.tv/oauth2/token`, cache + refresh.
- [ ] Private `IgdbGameRaw` (+ `IgdbCover`, `IgdbNamed`) deserialize structs.
- [ ] `search_games` -> POST with Apicalypse body, map `IgdbGameRaw -> SearchResult`
      (convert timestamp->year, build cover URL, flatten nested names).

### 4. Service + commands
- [ ] `add_from_search` takes `&SearchResult`, becomes near-passthrough
      (fields pre-derived in api layer). Persist `igdb_id` + `api_data_version`.
- [ ] `search_games` command returns `Vec<SearchResult>`.
- [ ] `AppState.rawg_client` -> `igdb_client`; `lib.rs` env vars
      (`TWITCH_CLIENT_ID`, `TWITCH_CLIENT_SECRET` instead of `RAWG_API_KEY`).
- [ ] Update `.github/workflows/release.yml` secrets.

### 5. Frontend (`src/routes/+page.svelte`)
- [ ] Rename `RawgGameData` interface -> `SearchResult`; update field names
      (`background_image` still works if api builds the URL; `platforms`/`genres`
      shapes change to string arrays).
- [ ] `add_game_from_search` invoke payload key `rawgData` -> `searchResult`.

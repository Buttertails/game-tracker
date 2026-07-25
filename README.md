# Lil Shelf

A desktop app that helps you actually finish your game backlog by limiting your active games to 3 at a time.

## Philosophy

Most backlog trackers let you build infinite lists — which makes picking a game harder, not easier. Lil Shelf uses a "small shelf" approach:

- **3 games max in progress** — forces you to commit before starting something new
- **Smart Fill** — suggests diverse picks from your backlog so you don't end up with 3 long RPGs
- **Shelves for groups** — track solo games separately from co-op sessions with friends
- **Focus on what you're playing** — the main view shows only your active games, not your entire library

## Features

- Search and add games via the RAWG database (auto-fills metadata and cover art)
- Manual entry for games not in the database
- Smart Fill algorithm that maximizes variety across genre, length, and era
- Timestamped notes per game (journal your playthrough)
- Launch games directly from the app
- Multiple shelves for different player groups (Solo, Co-op with roommate, etc.)
- Completion tracking with start/finish timestamps

## Download

Grab the latest installer from the [Releases](../../releases) page.

## Tech Stack

- **Backend:** Rust (Tauri 2.0)
- **Frontend:** Svelte + TypeScript
- **Database:** SQLite (local, embedded)
- **Game Data:** RAWG API

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://rustup.rs/)
- [RAWG API Key](https://rawg.io/apidocs) (free)

### Setup

```bash
cd game_tracker
npm install
```

Create `src-tauri/.env`:
```
RAWG_API_KEY=your_key_here
```

### Run

```bash
npx tauri dev
```

### Build

```bash
npx tauri build
```

Output at `src-tauri/target/release/bundle/`.

## License

MIT

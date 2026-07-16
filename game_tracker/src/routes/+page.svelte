<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Modal from "../lib/Modal.svelte";
  import {open} from "@tauri-apps/plugin-dialog";

  interface ShelfSummary {
    shelf_id: number;
    name: string;
    backlog_count: number;
    in_progress_count: number;
    completed_count: number;
  }

  interface GameEntry {
    entry_id: number;
    shelf_id: number;
    name: string;
    status: string;
    genre: string | null;
    length_category: string | null;
    release_year: number | null;
    source: string | null;
    launch_path: string | null;
    ownership_status: string;
    started_at: string | null;
    completed_at: string | null;
    addition_date: string;
    tags: string[];
    last_played: string | null;
    background_image: string | null;
  }

  interface ShelfEntries {
    shelf: {shelf_id: number, name: string, created_at: string};
    backlog: GameEntry[];
    in_progress: GameEntry[];
    completed: GameEntry[];
  }

  interface RawgGameData {
    id: number;
    name: string;
    released: string | null;
    rating: number | null;
    metacritic: number | null;
    platforms: { platform: { id: number; name: string } }[] | null;
    genres: { id: number; name: string }[] | null;
    background_image: string | null;
    esrb_rating: { name: string } | null;
    [key: string]: any;  // extra fields
  } 


  let shelves = $state<ShelfSummary[]>([]);
  let activeShelfId = $state<number | null>(null);
  let shelfData = $state<ShelfEntries | null>(null);
  let showBacklogPicker = $state(false);
  let backlogView = $state<"grid" | "detail" | "add">("grid");
  let selectedBacklogEntry = $state<GameEntry | null>(null);
  let searchQuery = $state("");
  let searchResults = $state<RawgGameData[]>([]);
  let searchLoading = $state(false);
  let searchError = $state("");
  let showManualForm = $state(false);
  let manualName = $state("");
  let manualGenre = $state("");
  let manualLength = $state("");
  let manualYear = $state("");
  let manualSource = $state("");
  let manualError = $state("");
  let selectedSearchResult = $state<RawgGameData | null>(null);
  let addSource = $state("");
  let addOwnership = $state("NotInstalled");
  let addLaunchPath = $state("");
  let showDetailModal = $state(false);
  let detailEntry = $state<GameEntry | null>(null);
  let detailNotes = $state<any[]>([]);
  let newNoteText = $state("");
  let detailError = $state("");
  let smartFillMode = $state(false);
  let smartFillSuggestions = $state<any[]>([]);
  let smartFillError = $state("");
  let showCompletedModal = $state(false);
  let showNewShelfInput = $state(false);
  let newShelfName = $state("");
  let shelfError = $state("");



  onMount(async () => {
    shelves = await invoke("get_shelves");
    if (shelves.length > 0) {
      activeShelfId = shelves[0].shelf_id;
      await loadShelf(shelves[0].shelf_id);
    }
  });

  async function loadShelf(shelfId: number) {
    activeShelfId = shelfId;
    shelfData = await invoke("get_entries_by_shelf", {shelfId});
  }

  async function openBacklogPicker() {
    backlogView = "grid";
    selectedBacklogEntry = null;
    showBacklogPicker = true;
  }

  function viewBacklogDetail(entry: GameEntry) {
    selectedBacklogEntry = entry;
    backlogView = "detail";
  }

  function backToGrid() {
    backlogView = "grid";
    selectedBacklogEntry = null;
  }

  async function startGame(entryId: number) {
    try {
      await invoke("move_to_in_progress", {entryId});
      showBacklogPicker = false;
      await loadShelf(activeShelfId!);
    } catch (error:any) {
      alert(error.InProgressFull?.shelf_id ? "All slots are full!" : "Failed to start game");
    }
  }

  function openAddGame() {
    backlogView = "add";
    searchQuery = "";
    searchResults = [];
    searchError = "";
    showManualForm = false;
  }

  async function performSearch() {
    if (!searchQuery.trim()) return;

    searchLoading = true;
    searchError = "";
    try {
      searchResults = await invoke("search_games", {query: searchQuery});
      if(searchResults.length === 0) {
        searchError = "No results found";
      }
    } catch (error: any) {
      searchError = error.ApiUnavailable || error.ValidationError || "Search failed";
    }
    searchLoading = false;
  }

  async function addFromSearch(result: RawgGameData) {
    try {
      await invoke("add_game_from_search", {
        rawgData: result,
        shelfId: activeShelfId,
        source: null,
        launchPath: null,
        ownershipStatus: "NotInstalled",
        tagList: [],
      });
      await loadShelf(activeShelfId!);
      backlogView = "grid";
    } catch (error: any) {
      console.error("Add from search error: ", error);
      searchError = JSON.stringify(error);
    }
  }

  async function addManually() {
    if (!activeShelfId) {
      manualError = "No shelf selected";
      return;
    }

    manualError = "";

    let input: any = {
      name: manualName,
      shelfId: activeShelfId,
      genre: manualGenre.trim() || null,
      lengthCategory: manualLength || null,
      releaseYear: manualYear ? parseInt(manualYear) : null,
      source: manualSource.trim() || null,
      launchPath: null,
      ownershipStatus: "NotInstalled",
    };

    try {
      await invoke("add_game_manually", { input, tagList: [] });
      await loadShelf(activeShelfId!);
      // Reset form and go back to grid
      manualName = "";
      manualGenre = "";
      manualLength = "";
      manualYear = "";
      manualSource = "";
      backlogView = "grid";
    } catch (error: any) {
      manualError = typeof error === "string" ? error : JSON.stringify(error);
    }
  }

  function selectSearchResult(result: RawgGameData) {
  selectedSearchResult = result;
  // Pre-populate source dropdown options from platforms
  addSource = "";
  addOwnership = "NotInstalled";
  addLaunchPath = "";
}

function backToResults() {
  selectedSearchResult = null;
}

async function confirmAddFromSearch() {
  if (!activeShelfId || !selectedSearchResult) return;

  try {
    await invoke("add_game_from_search", {
      rawgData: selectedSearchResult,
      shelfId: activeShelfId,
      source: addSource.trim() || null,
      launchPath: addLaunchPath.trim() || null,
      ownershipStatus: addOwnership,
      tagList: [],
    });
    await loadShelf(activeShelfId!);
    selectedSearchResult = null;
    backlogView = "grid";
  } catch (error: any) {
    searchError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function browseLaunchPath() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Executables", extensions: ["exe", "bat", "cmd", "lnk"] }],
  });

  if (selected) {
    addLaunchPath = selected as string;
  }
}

async function openDetail(entry: GameEntry) {
  detailEntry = entry;
  showDetailModal = true;
  detailError = "";
  newNoteText = "";
  // Load notes for this entry
  try {
    detailNotes = await invoke("get_notes", { entryId: entry.entry_id });
  } catch {
    detailNotes = [];
  }
}

async function completeGame() {
  if (!detailEntry) return;
  try {
    await invoke("move_to_completed", { entryId: detailEntry.entry_id });
    showDetailModal = false;
    await loadShelf(activeShelfId!);
  } catch (error: any) {
    detailError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function returnToBacklog() {
  if (!detailEntry) return;
  try {
    await invoke("move_to_backlog", { entryId: detailEntry.entry_id });
    showDetailModal = false;
    await loadShelf(activeShelfId!);
  } catch (error: any) {
    detailError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function addNote() {
  if (!detailEntry || !newNoteText.trim()) return;
  try {
    await invoke("add_note", { entryId: detailEntry.entry_id, text: newNoteText });
    newNoteText = "";
    detailNotes = await invoke("get_notes", { entryId: detailEntry.entry_id });
  } catch (error: any) {
    detailError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function deleteNote(noteId: number) {
  try {
    await invoke("delete_note", { noteId });
    if (detailEntry) {
      detailNotes = await invoke("get_notes", { entryId: detailEntry.entry_id });
    }
  } catch (error: any) {
    detailError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

function formatLocalDate(utcString: string | null): string {
  if (!utcString) return "Unknown";
  const date = new Date(utcString); // append Z to indicate UTC
  return date.toLocaleString(); // converts to user's local timezone
}

function formatOwnership(status: string): string {
  switch (status) {
    case "Installed": return "Installed";
    case "NotInstalled": return "Not Installed";
    case "Wishlisted": return "Wishlisted";
    default: return status;
  }
}

async function triggerSmartFill() {
  if (!activeShelfId) return;
  smartFillError = "";
  try {
    smartFillSuggestions = await invoke("get_smart_fill_suggestions", { shelfId: activeShelfId });
    smartFillMode = true;
  } catch (error: any) {
    smartFillError = typeof error === "string" ? error : JSON.stringify(error);
    alert(smartFillError);
  }
}

async function acceptSuggestion(entryId: number) {
  try {
    await invoke("accept_smart_fill", { entryIds: [entryId] });
    smartFillSuggestions = smartFillSuggestions.filter(s => s.entry_id !== entryId);
    if (smartFillSuggestions.length === 0) smartFillMode = false;
    await loadShelf(activeShelfId!);
  } catch (error: any) {
    alert(typeof error === "string" ? error : JSON.stringify(error));
  }
}

async function acceptAll() {
  const ids = smartFillSuggestions.map(s => s.entry_id);
  try {
    await invoke("accept_smart_fill", { entryIds: ids });
    smartFillSuggestions = [];
    smartFillMode = false;
    await loadShelf(activeShelfId!);
  } catch (error: any) {
    alert(typeof error === "string" ? error : JSON.stringify(error));
  }
}

async function rerollSuggestion(entryId: number) {
  if (!activeShelfId) return;
  const keptIds = smartFillSuggestions.filter(s => s.entry_id !== entryId).map(s => s.entry_id);
  try {
    const newSuggestion = await invoke("reroll_suggestion", {
      shelfId: activeShelfId,
      entryToReplaceId: entryId,
      keptSuggestionIds: keptIds,
    });
    smartFillSuggestions = smartFillSuggestions.map(s =>
      s.entry_id === entryId ? newSuggestion : s
    );
  } catch (error: any) {
    alert(typeof error === "string" ? error : JSON.stringify(error));
  }
}

async function createShelf() {
  if (!newShelfName.trim()) return;
  shelfError = "";
  try {
    await invoke("create_shelf", { name: newShelfName });
    newShelfName = "";
    showNewShelfInput = false;
    shelves = await invoke("get_shelves");
  } catch (error: any) {
    shelfError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function deleteShelf(shelfId: number) {
  const confirmed = confirm("Delete this shelf and all its games?");
  if (!confirmed) return;
  try {
    await invoke("delete_shelf", { shelfId, confirmed: true });
    shelves = await invoke("get_shelves");
    if (activeShelfId === shelfId) {
      if (shelves.length > 0) {
        await loadShelf(shelves[0].shelf_id);
      } else {
        activeShelfId = null;
        shelfData = null;
      }
    }
  } catch (error: any) {
    alert(typeof error === "string" ? error : JSON.stringify(error));
  }
}

async function handleLaunch(entry: GameEntry) {
  if (entry.launch_path && entry.ownership_status === "Installed") {
    try {
      await invoke("launch_game", { entryId: entry.entry_id });
    } catch (error: any) {
      alert(typeof error === "string" ? error : JSON.stringify(error));
    }
  } else {
    // Prompt to set path
    const selected = await open({
      multiple: false,
      filters: [{ name: "Executables", extensions: ["exe", "bat", "cmd", "lnk"] }],
    });
    if (selected) {
      try {
        await invoke("update_launch_path", { entryId: entry.entry_id, path: selected as string });
        await loadShelf(activeShelfId!);
      } catch (error: any) {
        alert(typeof error === "string" ? error : JSON.stringify(error));
      }
    }
  }
}

function cancelSmartFill() {
  smartFillMode = false;
  smartFillSuggestions = [];
}

async function updateLaunchPath(detailEntry: GameEntry) {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Executables", extensions: ["exe", "bat", "cmd", "lnk"] }],
  });

  if (selected) {
    detailEntry.launch_path = selected;
    await invoke("update_launch_path", {entryId: detailEntry.entry_id, path: selected});
  }
}

async function undoToBacklog(entryId: number) {
  const confirmed = confirm("Are you sure? This will clear your completion timestamps.");

  await invoke("undo_completion_to_backlog", {entryId, confirmed: confirmed});
}

async function undoToInProgress(entryId: number) {
  const confirmed = confirm("Are you sure? This will clear your completion timestamps.");

  await invoke("undo_completion_to_in_progress", {entryId, confirmed: confirmed});
}

</script>

<Modal bind:showModal={showBacklogPicker}>
  {#if backlogView === "grid"}
    <h2>Backlog</h2>
    <div class="game-grid">
      <!-- Add New card -->
      <div class="grid-card empty" onclick={openAddGame}>
        <div class="grid-art placeholder">
          <span class="plus-icon">+</span>
        </div>
        <p class="grid-name">Add Game</p>
      </div>

      <!-- Game cards -->
      {#each shelfData?.backlog ?? [] as entry}
        <div class="grid-card" onclick={() => viewBacklogDetail(entry)}>
          <div class="grid-art">
            <img src={entry.background_image || "/placeholder.png"} alt={entry.name} />
          </div>
          <p class="grid-name">{entry.name}</p>
          <button class="start-btn" onclick={(e) => { e.stopPropagation(); startGame(entry.entry_id); }}>
            Start
          </button>
        </div>
      {/each}
    </div>
  {:else if backlogView === "add"}
    <button class="back-btn" onclick={() => selectedSearchResult ? backToResults() : backToGrid()}>← Back</button>
    <h2>Add Game</h2>

    {#if !showManualForm}
      {#if !selectedSearchResult}
        <!-- Search + Results list -->
        <div class="search-row">
          <input
            type="text"
            placeholder="Search for a game..."
            bind:value={searchQuery}
            onkeydown={(e) => e.key === "Enter" && performSearch()}
          />
          <button onclick={performSearch} disabled={searchLoading}>
            {searchLoading ? "Searching..." : "Search"}
          </button>
        </div>

        {#if searchError}
          <p class="error-msg">{searchError}</p>
        {/if}

        {#if searchResults.length > 0}
          <ul class="search-results">
            {#each searchResults as result}
              <li class="result-row" onclick={() => selectSearchResult(result)}>
                <img
                  class="result-thumb"
                  src={result.background_image || "/placeholder.png"}
                  alt={result.name}
                />
                <div class="result-info">
                  <strong>{result.name}</strong>
                  <span class="meta">
                    {#if result.released}({result.released.slice(0, 4)}){/if}
                    {#if result.genres && result.genres.length > 0}• {result.genres[0].name}{/if}
                  </span>
                </div>
              </li>
            {/each}
          </ul>
        {/if}

        <button class="manual-link" onclick={() => showManualForm = true}>
          Can't find it? Add manually
        </button>
      {:else}
        <!-- Confirmation form for selected result -->
        <div class="confirm-layout">
          <div class="confirm-form">
            <label>
              Source / Platform
              <select bind:value={addSource}>
                <option value="">-- Select --</option>
                {#if selectedSearchResult.platforms}
                  {#each selectedSearchResult.platforms as p}
                    <option value={p.platform.name}>{p.platform.name}</option>
                  {/each}
                {/if}
                <option value="Other">Other</option>
              </select>
            </label>

            <label>
              Ownership Status
              <select bind:value={addOwnership}>
                <option value="NotInstalled">Not Installed</option>
                <option value="Installed">Installed</option>
                <option value="Wishlisted">Wishlisted</option>
              </select>
            </label>

            <label>
              Launch Path (optional)
              <div class="path-row">
                <input type="text" bind:value={addLaunchPath} placeholder="C:\Games\game.exe" />
                <button type="button" onclick={browseLaunchPath}>Browse</button>
              </div>
            </label>

            {#if searchError}
              <p class="error-msg">{searchError}</p>
            {/if}

            <button class="submit-btn" onclick={confirmAddFromSearch}>Add to Backlog</button>
          </div>

          <div class="confirm-sidebar">
            <img
              class="confirm-art"
              src={selectedSearchResult.background_image || "/placeholder.png"}
              alt={selectedSearchResult.name}
            />
            <h3>{selectedSearchResult.name}</h3>
            {#if selectedSearchResult.released}
              <p class="meta">Released: {selectedSearchResult.released.slice(0, 4)}</p>
            {/if}
            {#if selectedSearchResult.genres && selectedSearchResult.genres.length > 0}
              <p class="meta">Genre: {selectedSearchResult.genres.map(g => g.name).join(", ")}</p>
            {/if}
          </div>
        </div>
      {/if}
    {:else}
      <!-- Manual form -->
      <button class="back-btn" onclick={() => showManualForm = false}>← Back to search</button>
      <h3>Add Game Manually</h3>
      <p class="info-note">Only the game name is required. Genre, length, and year help Smart Fill recommend this game.</p>

      <div class="manual-form">
        <label>
          Game Name *
          <input type="text" bind:value={manualName} placeholder="e.g. Hollow Knight" />
        </label>

        <label>
          Genre (for Smart Fill)
          <select bind:value={manualGenre}>
            <option value="">-- Select --</option>
            <option value="Action">Action</option>
            <option value="RPG">RPG</option>
            <option value="Adventure">Adventure</option>
            <option value="Puzzle">Puzzle</option>
            <option value="Strategy">Strategy</option>
            <option value="Platformer">Platformer</option>
            <option value="Horror">Horror</option>
            <option value="FPS">FPS</option>
            <option value="Simulation">Simulation</option>
            <option value="Sports">Sports</option>
            <option value="Racing">Racing</option>
            <option value="Fighting">Fighting</option>
            <option value="Metroidvania">Metroidvania</option>
            <option value="Roguelite">Roguelite</option>
          </select>
        </label>

        <label>
          Estimated Length (for Smart Fill)
          <select bind:value={manualLength}>
            <option value="">-- Select --</option>
            <option value="Short">Short (under 10 hours)</option>
            <option value="Medium">Medium (10-30 hours)</option>
            <option value="Long">Long (over 30 hours)</option>
          </select>
        </label>

        <label>
          Release Year (for Smart Fill)
          <input type="number" bind:value={manualYear} placeholder="e.g. 2017" min="1950" max="2100" />
        </label>

        <label>
          Source
          <input type="text" bind:value={manualSource} placeholder="e.g. Steam, Epic, Emulated" />
        </label>

        {#if manualError}
          <p class="error-msg">{manualError}</p>
        {/if}

        <button class="submit-btn" onclick={addManually}>Add to Backlog</button>
      </div>
    {/if}
  {:else}
    <!-- Backlog Detail View -->
    <button class="back-btn" onclick={backToGrid}>← Back</button>
    {#if selectedBacklogEntry}
      <div class="detail-layout">
        <div class="detail-main">
          <h2>{selectedBacklogEntry.name}</h2>

          <div class="detail-meta">
            <p><strong>Genre:</strong> {selectedBacklogEntry.genre ?? "Not set"}</p>
            <p><strong>Length:</strong> {selectedBacklogEntry.length_category ?? "Not set"}</p>
            <p><strong>Release Year:</strong> {selectedBacklogEntry.release_year ?? "Not set"}</p>
            <p><strong>Source:</strong> {selectedBacklogEntry.source ?? "Not set"}</p>
            <p><strong>Status:</strong> {formatOwnership(selectedBacklogEntry.ownership_status)}</p>
            <div>
              <strong>Launch Path:</strong>
              <button class="update-path-btn" onclick={() => updateLaunchPath(detailEntry!)} title="Set new launch path for game">🔎</button>
            </div>
            <p style="font-size: 0.7rem">{selectedBacklogEntry.launch_path ?? "Not set"}</p>
          </div>

          {#if !selectedBacklogEntry.genre || !selectedBacklogEntry.length_category || !selectedBacklogEntry.release_year}
            <p class="warning-note">⚠ Missing metadata — not eligible for Smart Fill</p>
          {/if}

          <div class="detail-actions">
            <button class="submit-btn" onclick={() => startGame(selectedBacklogEntry!.entry_id)}>▶ Start Playing</button>
          </div>
        </div>

        <div class="detail-sidebar">
          <img class="confirm-art" src={selectedBacklogEntry.background_image || "/placeholder.png"} alt={selectedBacklogEntry.name} />
        </div>
      </div>
    {/if}
  {/if}

</Modal>

<Modal bind:showModal={showDetailModal}>
  {#if detailEntry}
    <div class="detail-layout">
      <div class="detail-main">
        <h2>{detailEntry.name}</h2>

        <div class="detail-meta">
          <p><strong>Last Played:</strong> {formatLocalDate(detailEntry.last_played) ?? "Unknown"}</p>
          <p><strong>Started:</strong> {formatLocalDate(detailEntry.started_at) ?? "Unknown"}</p>
          <div>
            <strong>Launch Path:</strong>
            <button class="update-path-btn" onclick={() => updateLaunchPath(detailEntry!)} title="Set new launch path for game">🔎</button>
          </div>
          <p style="font-size: 0.7rem">{detailEntry.launch_path ?? "Not set"}</p>
        </div>

        {#if !detailEntry.genre || !detailEntry.length_category || !detailEntry.release_year}
          <p class="warning-note">⚠ Missing metadata — not eligible for Smart Fill</p>
        {/if}

        <!-- Notes section -->
        <div class="notes-section">
          <h3>Notes</h3>
          <div class="note-input-row">
            <input
              type="text"
              placeholder="Add a note..."
              bind:value={newNoteText}
              onkeydown={(e) => e.key === "Enter" && addNote()}
            />
            <button class="submit-btn" onclick={addNote}>Add</button>
          </div>
          {#if detailNotes.length > 0}
            <ul class="notes-list">
              {#each detailNotes as note}
                <li>
                  <p class="note-text">{note.text}</p>
                  <span class="note-date">{formatLocalDate(note.created_at)}</span>
                  <button class="note-delete" onclick={() => deleteNote(note.note_id)}>×</button>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="meta">No notes yet.</p>
          {/if}
        </div>

        {#if detailError}
          <p class="error-msg">{detailError}</p>
        {/if}

        <div class="detail-actions">
          <button class="submit-btn" onclick={completeGame}>✓ Complete</button>
          <button class="cancel-btn" onclick={returnToBacklog}>✗ Return to Backlog</button>
        </div>
      </div>

      <div class="detail-sidebar">

        <img
          class="confirm-art"
          src={detailEntry.background_image || "/placeholder.png"}
          alt={detailEntry.name}
        />
        <div class="meta-columns">
          <div class="column">
            <p><strong>Genre:</strong> {detailEntry.genre ?? "Not set"}</p>
            <p><strong>Length:</strong> {detailEntry.length_category ?? "Not set"}</p>
            <p><strong>Release Year:</strong> {detailEntry.release_year ?? "Not set"}</p>
          </div>
          <div class="column">
            <p><strong>Source:</strong> {detailEntry.source ?? "Not set"}</p>
            <p><strong>Status:</strong> {formatOwnership(detailEntry.ownership_status)}</p>
          </div>
        </div>
      </div>
    </div>
  {/if}
</Modal>

<Modal bind:showModal={showCompletedModal}>
  <h2>Completed</h2>
  {#if (shelfData?.completed ?? []).length === 0}
    <p class="meta">No completed games yet.</p>
  {:else}
    <div class="game-grid">
      {#each shelfData?.completed ?? [] as entry}
        <div class="grid-card">
          <div class="grid-art">
            <img src={entry.background_image || "/placeholder.png"} alt={entry.name} />
          </div>
          <p class="grid-name">{entry.name}</p>
          <span class="completed-date">{formatLocalDate(entry.completed_at)}</span>
        </div>
      {/each}
    </div>
  {/if}
</Modal>

<div class="app">
  <!--Shelf Tabs-->
  <div class="shelf-tabs">
    {#each shelves as shelf}
      <button
        class="tab"
        class:active={activeShelfId === shelf.shelf_id}
        onclick={() => loadShelf(shelf.shelf_id)}
        oncontextmenu={(e) => { e.preventDefault(); deleteShelf(shelf.shelf_id); }}
      >
        {shelf.name}
      </button>
    {/each}

    {#if showNewShelfInput}
      <div class="new-shelf-input">
        <input
          type="text"
          placeholder="Shelf name..."
          bind:value={newShelfName}
          onkeydown={(e) => {
            if (e.key === "Enter") createShelf();
            if (e.key === "Escape") { showNewShelfInput = false; newShelfName = ""; }
          }}
        />
      </div>
    {:else}
      <button class="tab add-tab" onclick={() => showNewShelfInput = true}>+</button>
    {/if}
  </div>



    <!-- In-Progress Cards -->
     <div class="cards-area">
      {#each [0, 1, 2] as slot}
        {@const entry = shelfData?.in_progress[slot]}
        <div class="card-slot">
          {#if entry}
            <div class="card filled" onclick={() => openDetail(entry)} ondblclick={() => handleLaunch(entry)}>
              <img class="box-art" src={entry.background_image || "/placeholder.png"} alt={entry.name} />
            </div>
            <p class="game-name">{entry.name}</p>
            <button class="launch-btn" onclick={() => handleLaunch(entry)}>
              {entry.ownership_status === "Installed" && entry.launch_path ? "Launch" :
              entry.ownership_status === "Installed" ? "Set Path" :
              entry.ownership_status === "Wishlisted" ? "Wishlisted" : "Not Installed"}
            </button>
          {:else}
            {@const suggestion = smartFillMode ? smartFillSuggestions[slot - (shelfData?.in_progress.length ?? 0)] : null}
            {#if suggestion}
              <div class="card filled suggestion">
              <img class="box-art" src={suggestion.background_image || "/placeholder.png"} alt={suggestion.name} />
              </div>
              <p class="game-name">{suggestion.name}</p>
              <div class="suggestion-actions">
                <button class="submit-btn" onclick={() => acceptSuggestion(suggestion.entry_id)}>Accept</button>
                <button class="back-btn" onclick={() => rerollSuggestion(suggestion.entry_id)}>Re-roll</button>
              </div>
            {:else}
              <div class="card empty" onclick={() => openBacklogPicker()}>
                <span class="plus-icon">+</span>
              </div>
              <p class="game-name placeholder">Click to add</p>
            {/if}
          {/if}
        </div>
      {/each}
     </div>

     <!-- Bottom Actions -->
      <div class="bottom-bar">
        <button class="bottom-btn" onclick={openBacklogPicker}>Backlog ({shelfData?.backlog.length ?? 0})</button>
        <button
          class="bottom-btn smart-fill"
          onclick={triggerSmartFill}
          disabled={(shelfData?.in_progress.length ?? 0) >= 3}
        >
          🎲 Smart Fill
        </button>
        <button class="bottom-btn" onclick={() => showCompletedModal = true}>Completed ({shelfData?.completed.length ?? 0})</button>
      </div>

      {#if smartFillMode && smartFillSuggestions.length > 0}
        <div class="accept-all-bar">
          <span>
            <button class="submit-btn" onclick={acceptAll}>Accept All</button>
            <button class="submit-btn" onclick={cancelSmartFill}>Cancel</button>
          </span>
        </div>
      {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    background: #1a1a1a;
    color: #eee;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .shelf-tabs {
    display: flex;
    padding: 0.5rem 1rem;
    gap: 0.25rem;
    background: #222;
    border-bottom: 1px solid #333;
  }

  .tab {
    padding: 0.4rem 1rem;
    background: #333;
    border: none;
    color: #aaa;
    cursor: pointer;
    border-radius: 6px 6px 0 0;
    font-size: 0.85rem;
  }

  .tab.active {
    padding: 0.4rem 0.8rem;
    font-size: 1rem;
  }

  .tab:hover {
    color: #fff;
  }

  .add-tab {
    padding: 0.4rem 0.8rem;
    font-size: 1rem;
  }

  .cards-area {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 2rem;
    padding: 2rem;
  }

  .card-slot {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .card {
    width: 320px;
    height: 180px;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .card:hover {
    transform: scale(1.03);
  }

  .card.filled {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .card.empty {
    border: 2px dashed #444;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #252525;
  }

  .card.empty:hover {
    border-color: #666;
    background: #2a2a2a;
  }

  .plus-icon {
    font-size: 2.5rem;
    color: #555;
  }

  .box-art {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .game-name {
    font-size: 0.95rem;
    font-weight: 600;
    text-align: center;
    margin: 0;
  }

  .game-name.placeholder {
    color: #666;
    font-weight: normal;
  }

  .launch-btn {
    padding: 0.4rem 1.2rem;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-size: 0.8rem;
    background: #4d148e;
    color: #fff;
  }

  .launch-btn:hover {
    background: #6b1fba;
  }

  .bottom-bar {
    display: flex;
    justify-content: center;
    gap: 1.5rem;
    padding: 1rem;
    background: #222;
    border-top: 1px solid #333;
  }

  .bottom-btn {
    padding: 0.5rem 1.5rem;
    border-radius: 6px;
    border: 1px solid #444;
    background: #2a2a2a;
    color: #ccc;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .bottom-btn:hover {
    border-color: #666;
    color: #fff;
  }

  .bottom-btn.smart-fill {
    background: #4d148e;
    border-color: #4d148e;
    color: #fff;
  }

  .bottom-btn.smart-fill:hover {
    background: #6b1fba;
  }

  .game-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 1rem;
}

.grid-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
}

.grid-art {
  width: 200px;
  height: 112px;
  border-radius: 6px;
  overflow: hidden;
  background: #333;
}

.grid-art img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.grid-art.placeholder {
  border: 2px dashed #555;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #252525;
}

.grid-card:hover .grid-art {
  box-shadow: 0 0 0 2px #6b1fba;
}

.grid-name {
  font-size: 0.8rem;
  text-align: center;
  margin: 0;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.start-btn {
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  border: none;
  background: #4d148e;
  color: #fff;
  cursor: pointer;
  font-size: 0.75rem;
}

.start-btn:hover {
  background: #6b1fba;
}

.search-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.search-row input {
  flex: 1;
  padding: 0.5rem;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
}

.search-row button {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  border: none;
  background: #4d148e;
  color: #fff;
  cursor: pointer;
}

.search-results {
  list-style: none;
  padding: 0;
  max-height: 300px;
  overflow-y: auto;
}

.search-results li {
  padding: 0.6rem 0.5rem;
  cursor: pointer;
  border-radius: 4px;
  border-bottom: 1px solid #333;
}

.search-results li:hover {
  background: #333;
}

.meta {
  font-size: 0.85rem;
  color: #999;
  margin-left: 0.5rem;
}

.error-msg {
  color: #ff6b6b;
  font-size: 0.85rem;
}

.manual-link {
  background: none;
  border: none;
  color: #aaa;
  text-decoration: underline;
  cursor: pointer;
  margin-top: 1rem;
  font-size: 0.85rem;
}

.info-note {
  color: #888;
  font-size: 0.8rem;
  font-style: italic;
}

.manual-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  max-width: 400px;
}

.manual-form label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  color: #aaa;
}

.manual-form input,
.manual-form select {
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
  font-size: 0.85rem;
}

.manual-form input:focus,
.manual-form select:focus {
  border-color: #6b1fba;
  outline: none;
}

.result-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem;
  cursor: pointer;
  border-radius: 4px;
  border-bottom: 1px solid #333;
}

.result-row:hover {
  background: #333;
}

.result-thumb {
  width: 60px;
  height: 40px;
  border-radius: 4px;
  object-fit: cover;
  background: #333;
}

.result-info {
  display: flex;
  flex-direction: column;
}

.confirm-layout {
  display: flex;
  gap: 2rem;
  margin-top: 1rem;
}

.confirm-form {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  flex: 1;
  max-width: 350px;
}

.confirm-sidebar {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  min-width: 200px;
}

.confirm-sidebar h3 {
  margin: 0;
  text-align: center;
  font-size: 1rem;
}

.confirm-art {
  width: 320px;
  height: 180px;
  border-radius: 6px;
  object-fit: cover;
  background: #333;
}

.confirm-form label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  color: #aaa;
}

.confirm-form input,
.confirm-form select {
  padding: 0.5rem;
  border-radius: 6px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
  font-size: 0.85rem;
}

.confirm-form input:focus,
.confirm-form select:focus {
  border-color: #6b1fba;
  outline: none;
}

.path-row {
  display: flex;
  gap: 0.5rem;
}

.path-row input {
  flex: 1;
}

.path-row button {
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  border: none;
  background: #4d148e;
  color: #fff;
  cursor: pointer;
  font-size: 0.8rem;
}

.path-row button:hover {
  background: #6b1fba;
}

.back-btn {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  font-size: 0.85rem;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  align-self: flex-start;
  margin-bottom: 0.5rem;
}

.back-btn:hover {
  color: #fff;
  background: #333;
}

.submit-btn {
  padding: 0.5rem 1.2rem;
  border-radius: 6px;
  border: none;
  background: #4d148e;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  align-self: flex-start;
  margin-top: 0.25rem;
}

.cancel-btn {
  padding: 0.5rem 1.2rem;
  border-radius: 6px;
  border: none;
  background: #9f1414;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  align-self: flex-start;
  margin-top: 0.25rem;
}

.submit-btn:hover {
  background: #6b1fba;
}

.detail-layout {
  display: flex;
  gap: 2rem;
}

.detail-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.detail-main h2 {
  margin: 0;
}

.detail-sidebar {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.detail-meta p {
  justify-content: flex-start;
  margin: 0.2rem 0;
  font-size: 0.85rem;
  color: #ccc;
}

.warning-note {
  color: #f5a623;
  font-size: 0.8rem;
  background: rgba(245, 166, 35, 0.1);
  padding: 0.4rem 0.6rem;
  border-radius: 4px;
}

.notes-section {
  margin-top: 0.5rem;
}

.notes-section h3 {
  margin: 0 0 0.5rem 0;
  font-size: 0.95rem;
}

.note-input-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.note-input-row input {
  flex: 1;
  padding: 0.4rem;
  border-radius: 6px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
  font-size: 0.85rem;
}

.note-input-row input:focus {
  border-color: #6b1fba;
  outline: none;
}

.notes-list {
  list-style: none;
  padding: 0;
  max-height: 200px;
  overflow-y: auto;
}

.notes-list li {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 0.4rem 0;
  border-bottom: 1px solid #333;
}

.note-text {
  flex: 1;
  margin: 0;
  font-size: 0.85rem;
}

.note-date {
  font-size: 0.7rem;
  color: #666;
  white-space: nowrap;
}

.note-delete {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 1rem;
}

.note-delete:hover {
  color: #ff6b6b;
}

.detail-actions {
  display: flex;
  gap: 0.75rem;
  margin-top: 0.5rem;
}

.suggestion-actions {
  display: flex;
  gap: 0.5rem;
}

.card.suggestion {
  border: 2px solid #6b1fba;
}

.accept-all-bar {
  display: flex;
  justify-content: center;
  padding: 0.5rem;
}

.completed-date {
  font-size: 0.7rem;
  color: #888;
  text-align: center;
}

.new-shelf-input {
  display: flex;
  align-items: center;
}

.new-shelf-input input {
  padding: 0.3rem 0.5rem;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
  font-size: 0.8rem;
  width: 120px;
}

.new-shelf-input input:focus {
  border-color: #6b1fba;
  outline: none;
}

.update-path-btn {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  color: #ccc;
}

.update-path-btn:hover {
  color: #6b1fba;
}

.column {
  padding: 0;
}

.meta-columns {
  display: flex;
  gap: 2rem;
  font-size: 0.85rem;
  color: #ccc;
  justify-content: space-between;
  width: 100%;
}

</style>

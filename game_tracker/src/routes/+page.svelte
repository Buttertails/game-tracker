<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Modal from "../lib/Modal.svelte";
  import {open, ask} from "@tauri-apps/plugin-dialog";
  import {openUrl} from "@tauri-apps/plugin-opener";
  import {getVersion} from "@tauri-apps/api/app";
  
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
  let backlogView = $state<"grid" | "add">("grid");
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
  let appUpdateAvailabe = $state(false);
  let editingMetadata = $state(false);
  let editGenre = $state("");
  let editLength = $state("");
  let editYear = $state("");
  let editSource = $state("");
  let editOwnership = $state("");



  onMount(async () => {
    shelves = await invoke("get_shelves");
    if (shelves.length > 0) {
      activeShelfId = shelves[0].shelf_id;
      await loadShelf(shelves[0].shelf_id);
    }

    const response = await fetch("https://api.github.com/repos/Buttertails/game-tracker/releases/latest");
    const data = await response.json();
    const latestVersion = data.tag_name;
    const latest = latestVersion.replace("v", "");
    const currentVersion = await getVersion();
    const appUpdateAvailabe = latest !== currentVersion; 
  });

  async function loadShelf(shelfId: number) {
    activeShelfId = shelfId;
    shelfData = await invoke("get_entries_by_shelf", {shelfId});
  }

  async function openBacklogPicker() {
    backlogView = "grid";
    detailEntry = null;
    showBacklogPicker = true;
  }

  function backToGrid() {
    backlogView = "grid";
    detailEntry = null;
  }

  async function startGame(entryId: number) {
    try {
      await invoke("move_to_in_progress", {entryId});
      showDetailModal = false;
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
      shelf_id: activeShelfId,
      genre: manualGenre.trim() || null,
      lengthCategory: manualLength || null,
      releaseYear: manualYear ? parseInt(manualYear) : null,
      source: manualSource.trim() || null,
      launchPath: null,
      ownership_status: "NotInstalled",
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
  showBacklogPicker = false;
  detailEntry = entry;
  showDetailModal = true;
  editingMetadata = false;
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

async function addNote(entryId: number) {
  if (!newNoteText.trim()) return;
  try {
    await invoke("add_note", { entryId, text: newNoteText });
    newNoteText = "";
    detailNotes = await invoke("get_notes", { entryId });
  } catch (error: any) {
    detailError = typeof error === "string" ? error : JSON.stringify(error);
  }
}

async function deleteNote(noteId: number, entryId: number) {
  try {
    await invoke("delete_note", { noteId });
    
    detailNotes = await invoke("get_notes", { entryId });
    
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
  const confirmed = await ask('Delete this shelf and all of its games? The data will be permanently lost.', {title: 'Confirm', kind: 'warning'});
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
        await invoke("update_ownership_status", {entryId: entry.entry_id, ownershipStatus: "Installed"});
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

    if (detailEntry.ownership_status !== "Installed") {
      await invoke("update_ownership_status", {entryId: detailEntry.entry_id, ownershipStatus: "Installed"});
      detailEntry.ownership_status = "Installed";
    }
    await loadShelf(activeShelfId!);
  }
}

async function undoToBacklog(entryId: number) {
  const confirmed = await ask("Are you sure? This will clear your completion timestamps.", {title: "Confirm", kind: "warning"});

  await invoke("undo_completion_to_backlog", {entryId, confirmed: confirmed});
}

async function undoToInProgress(entryId: number) {
  const confirmed = await ask("Are you sure? This will clear your completion timestamps.", {title: "Confirm", kind: "warning"});

  await invoke("undo_completion_to_in_progress", {entryId, confirmed: confirmed});
}

async function updateApp() {
  await openUrl("https://github.com/Buttertails/game-tracker/releases");
}

function handleGlobalKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    if (showBacklogPicker) showBacklogPicker=false;
    else if (showCompletedModal) showCompletedModal=false;
    else if (showDetailModal) showDetailModal=false;
    else if (showManualForm) showManualForm=false;
  }
  else if (e.key === "Backspace" && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) {
    if (showBacklogPicker && backlogView === "add") {
      if (showManualForm) showManualForm = false; 
      else if (selectedSearchResult) backToResults();
      else backToGrid();
    }
  }
}

function startEditingMetadata() {
  editGenre = detailEntry?.genre ?? "";
  editLength = detailEntry?.length_category ?? "";
  editYear = detailEntry?.release_year?.toString() ?? "";
  editSource = detailEntry?.source ?? "";
  editOwnership = detailEntry?.ownership_status ?? "";

  editingMetadata = true;
}

async function saveMetadata(entryId: number) {

  const genre = editGenre.trim() || null;
  const length_category = editLength || null;
  const release_year = editYear ? parseInt(editYear) : null;
  const source = editSource.trim() || null;
  const status = editOwnership.trim() || null;

  await invoke("update_entry_metadata", {
    entryId,
    genre,
    lengthCategory: length_category,
    releaseYear: release_year,
    source,
  });

  if (editOwnership !== detailEntry?.ownership_status) {
    await invoke("update_ownership_status", {entryId, ownershipStatus: status});
  }
  

  detailEntry = await invoke("get_game_entry", {entryId});
  await loadShelf(activeShelfId!);
  editingMetadata = false;
}

async function deleteEntry(entryId: number) {
  await invoke("delete_game_entry", {entryId});
  await loadShelf(activeShelfId!);
  detailEntry = null;
  showDetailModal = false;
}

async function clearLaunchPath() {
  await invoke("update_launch_path", {entryId: detailEntry!.entry_id, path: null});
  await invoke("update_ownership_status", {entryId: detailEntry!.entry_id, ownershipStatus: "NotInstalled"});
  
  await loadShelf(activeShelfId!);
  detailEntry!.launch_path = null;
  detailEntry!.ownership_status = "NotInstalled";
}

</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#snippet detailSidebar(entry: GameEntry)}
  <div class="detail-sidebar">
    <img
      class="confirm-art"
      src={entry.background_image || "/placeholder.png"}
      alt={entry.name}
    />
    <div class="meta-columns">
      {#if editingMetadata}
        <div class="column">
          <p>
            <strong>Genre:</strong>
            <select bind:value={editGenre}>
              <option value="">{entry.genre}</option>
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
          </p>
          <p>
            <strong>Length:</strong>
            <select bind:value={editLength}>
              <option value="">{entry.length_category}</option>
              <option value="Short">Short (under 10 hours)</option>
              <option value="Medium">Medium (10-30 hours)</option>
              <option value="Long">Long (over 30 hours)</option>
            </select>
          </p>
          <p>
            <strong>Release Year:</strong>
            <input type="number" bind:value={editYear} placeholder={entry.release_year!.toString()} min="1950" max="2100" />
          </p>
          <div class="edit-actions">
            <button class="edit-submit-btn" title="Save changes" onclick={() => saveMetadata(detailEntry?.entry_id!)}>✓ Save</button>
            <button class="edit-cancel-btn" title="Cancel changes" onclick={() => {if (editingMetadata) {editingMetadata = false;}}}>✗ Cancel</button>
          </div>
        </div>
        <div class="column">
          <p>
            <strong>Source:</strong>
            <input type="text" bind:value={editSource} placeholder={entry.source} />
          </p>
          <p>
            <strong>Status:</strong>
            <select bind:value={editOwnership}>
              <option value="NotInstalled">Not Installed</option>
              <option value="Installed">Installed</option>
              <option value="Wishlisted">Wishlisted</option>
          </select>
          </p>
        </div>
      {:else}
        <div class="column">
          <p><strong>Genre:</strong> {entry.genre ?? "Not set"}</p>
          <p><strong>Length:</strong> {entry.length_category ?? "Not set"}</p>
          <p><strong>Release Year:</strong> {entry.release_year ?? "Not set"}</p>
        </div>
        <div class="column">
          <p><strong>Source:</strong> {entry.source ?? "Not set"}</p>
          <p><strong>Status:</strong> {formatOwnership(entry.ownership_status)}</p> 
          <button class="edit-btn" title="Edit metadata" onclick={startEditingMetadata}>✏️ Edit</button>
        </div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet notesSection(entryId: number)}
  <div class="notes-section">
    <h3>Notes</h3>
    <div class="note-input-row">
      <input
        type="text"
        placeholder="Add a note..."
        bind:value={newNoteText}
        onkeydown={(e) => e.key === "Enter" && addNote(entryId!)}
      />
      <button class="submit-btn" onclick={() => addNote(entryId!)}>Add</button>
    </div>
    {#if detailNotes.length > 0}
      <ul class="notes-list">
        {#each detailNotes as note}
          <li>
            <p class="note-text">{note.text}</p>
            <span class="note-date">{formatLocalDate(note.created_at)}</span>
            <button class="note-delete" onclick={() => deleteNote(note.note_id, entryId!)}>×</button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="meta">No notes yet.</p>
    {/if}
  </div>
{/snippet}

<Modal bind:showModal={showBacklogPicker} >
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
        <div class="grid-card" onclick={() => openDetail(entry)}>
          <div class="grid-art">
            <img src={entry.background_image || "/placeholder.png"} alt={entry.name} />
          </div>
          <p class="grid-name">
              {#if !entry.genre || !entry.length_category || !entry.release_year}
                <span style="color: #f5a623" title="Missing metadata for SmartFill">⚠</span> 
              {/if}
                {entry.name}
            </p>
          <button class="start-btn" onclick={(e) => { e.stopPropagation(); startGame(entry.entry_id); }}>
            Start
          </button>
        </div>
      {/each}
    </div>
  {:else if backlogView === "add"}
    <button class="back-btn" onclick={() => {if (showManualForm) { showManualForm = false; } else if (selectedSearchResult) { backToResults() } else { backToGrid() } }} >← Back</button>
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
  {/if}

</Modal>

<Modal bind:showModal={showDetailModal}>
  {#if detailEntry}
    <div class="detail-layout">
      <div class="detail-main">
        <h2>{detailEntry.name}</h2>

        <div class="detail-meta">
          {#if detailEntry.status === "InProgress"}
            <p><strong>Last Played:</strong> {formatLocalDate(detailEntry.last_played) ?? "Unknown"}</p>
            <p><strong>Started:</strong> {formatLocalDate(detailEntry.started_at) ?? "Unknown"}</p>
          {/if}
          <div>
            <strong>Launch Path:</strong>
            <button class="update-btn" onclick={() => updateLaunchPath(detailEntry!)} title="Set new launch path for game">🔎</button>
          </div>
          <p style="font-size: 0.7rem">{detailEntry.launch_path ?? "Not set"}
            {#if detailEntry.launch_path}
              <button class="clear-path" title="Clear launch path" onclick={clearLaunchPath}>×</button>
            {/if}
          </p>
        </div>

        {#if !detailEntry.genre || !detailEntry.length_category || !detailEntry.release_year}
        <div>
          <p class="warning-note">⚠ Missing metadata — not eligible for Smart Fill
            <button class=update-btn title="Manually set missing data" onclick={startEditingMetadata}>✏️</button>
          </p>
        </div>
        {/if}

        <!-- Notes section -->
        {@render notesSection(detailEntry!.entry_id)}

        {#if detailError}
          <p class="error-msg">{detailError}</p>
        {/if}

        <div class="detail-actions">
          {#if detailEntry.status === "Backlog"}
            <button class="submit-btn" onclick={() => startGame(detailEntry!.entry_id)}>▶ Start Playing</button>
            <button class="cancel-btn" onclick={() => deleteEntry(detailEntry!.entry_id)}>✗ Remove</button>
          {:else if detailEntry.status === "InProgress"}
            <button class="submit-btn" onclick={completeGame}>✓ Complete</button>
            <button class="cancel-btn" onclick={returnToBacklog}>✗ Return to Backlog</button>
          {/if}  
        </div>
      </div>

      {@render detailSidebar(detailEntry!)}
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
            <p class="game-name">
              {#if !entry.genre || !entry.length_category || !entry.release_year}
                <span style="color: #f5a623" title="Missing metadata for SmartFill">⚠</span> 
              {/if}
                {entry.name}
            </p>

            
            <button class="launch-btn" onclick={() => handleLaunch(entry)}>
              {entry.launch_path ? "Launch" : "Not Installed"}
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
        <div class="side-area-left">
          <button class="cancel-btn" onclick={() => deleteShelf(shelfData?.shelf.shelf_id!)}>Delete Shelf</button>
        </div>
        <button class="bottom-btn" onclick={openBacklogPicker}>Backlog ({shelfData?.backlog.length ?? 0})</button>
        <button
          class="bottom-btn smart-fill"
          onclick={triggerSmartFill}
          disabled={(shelfData?.in_progress.length ?? 0) >= 3}
        >
          🎲 Smart Fill
        </button>
        <button class="bottom-btn" onclick={() => showCompletedModal = true}>Completed ({shelfData?.completed.length ?? 0})</button>
        <div class="side-area-right">
          {#if appUpdateAvailabe}
            <button class="submit-btn" onclick={updateApp}>Update</button>
          {/if}
          </div>
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
    width: clamp(280px, 25vw, 500px);
    height: clamp(158px, 14vw, 281px);
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
    padding: 0.4rem 0.6rem;
    border-radius: 4px;
    border: 2px solid transparent;
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
  align-items: center;
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

.update-btn {
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

.side-area-left {
  flex: 1;
  display: flex;
  justify-content: flex-start;
}

.side-area-right {
  flex: 1;
  display: flex;
  justify-content: flex-end;
}

.edit-btn {
  padding: 0.2rem 0.4rem;
  border-radius: 6px;
  border: none;
  background: #5e5d5d;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  align-self: flex-start;
}

.edit-btn:hover {
  background: #4b4b4b;
}

.edit-cancel-btn {
  padding: 0.2rem 0.4rem;
  border-radius: 6px;
  border: none;
  background: #9f1414;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  align-self: flex-start;
}

.edit-submit-btn {
  padding: 0.2rem 0.4rem;
  border-radius: 6px;
  border: none;
  background: #4d148e;
  color: #fff;
  cursor: pointer;
  font-size: 0.85rem;
  align-self: flex-start;
}

.meta-columns input,
.meta-columns select {
  width: 100%;
  box-sizing: border-box;
  font-size: 0.85rem;
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
  border: 1px solid #444;
  background: #1a1a1a;
  color: #eee;
}

.edit-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.2rem;
}

.clear-path {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 1rem;
  vertical-align: middle;
}

.clear-path:hover {
  color: #ff6b6b;
}

</style>

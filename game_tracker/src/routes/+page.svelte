<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {onMount} from "svelte";
  import Modal from 'svelte';

  interface Category {
      category_id: number;
      name: string;
      is_preset: boolean;
      display_order: number;
      entry_count: number;
  }

  interface GameEntry {
      entry_id: number;
      name: string;
      category_id: number;
      category_number: string;
      addition_date: string;
      tags: string[];
      source: string | null;
      last_played: string | null;
  }

  // Reactive state
  let categories = $state<Category[]>([]); 
  let selectedCategoryId = $state<number | null>(null);
  let entries = $state<GameEntry[]>([]);

  // New category
  let showNewCategoryForm = $state(false);
  let newCategoryName = $state("");
  let categoryError = $state("");

  // Add game modal
  let showAddGameModal = $state(false);
  let searchQuery = $state("");
  let searchResults = $state<any[]>([]);
  let manualMode = $state(false);
  let newGameName = $state("");
  let newGameSource = $state("");
  let newGameTags= $state("");
  let addError = $state("");

  function closeModal() {
    showAddGameModal = false;
    searchQuery = "";
    searchResults = [];
    manualMode = false;
    newGameName = "";
    newGameSource = "";
    newGameTags = "";
    addError = "";
  }

  // Load categories on startup
  onMount(async () => {
      categories = await invoke("get_categories");

      if(categories.length > 0) {
          selectedCategoryId = categories[0].category_id;
          await loadEntries(categories[0].category_id);
      }
  });

  // Load entries when category is selected
  async function loadEntries(categoryId: number) {
      selectedCategoryId = categoryId;
      entries = await invoke("get_entries_by_category", {categoryId});
  }

  // Create new category
  async function createCategory() {
    categoryError = "";

    try {
      await invoke("create_category", {name: newCategoryName});
      newCategoryName = "";
      showNewCategoryForm = false;

      // Refresh categories list
      categories = await invoke("get_categories");
    } catch (error: any) {
      categoryError = error.ValidationError || error.DuplicateCategory || "Failed to create category";
    }
  }

  // Delete category
  async function deleteCategory(id: number, entryCount: number) {
    if (entryCount > 0) {
      const confirmed = confirm(`This category has ${entryCount} game(s). Delete anyway?`);
      if(!confirmed) return;
    }

    try {
      await invoke("delete_category", {id, confirm: entryCount > 0});
      categories = await invoke("get_categories");

      if (selectedCategoryId === id) {
        if (categories.length > 0) {
          await loadEntries(categories[0].category_id);
        } else {
          selectedCategoryId = null;
          entries = [];
        }
      }
    } catch (error: any) {
      alert(error.CategoryNotFound || "Failed to delete category");
    } 
  }

  //async function addGameEntry()  
</script>

<div class="app-layout">
    <aside class="sidebar">
        <ul>
            {#each categories as category}
                <li>
                  <div class="category-row"> 
                    <button
                        class:selected={selectedCategoryId === category.category_id}
                        onclick={() => loadEntries(category.category_id)}
                    >
                      {category.name}
                      <span class="count">({category.entry_count})</span>
                    </button>
                    <button
                      class="delete-btn"
                      onclick={() => deleteCategory(category.category_id, category.entry_count)}
                    >
                      x
                    </button>
                  </div>    
                </li>
            {/each}
        </ul>
        {#if showNewCategoryForm}
          <div class="new-category-form">
            <input 
              type="text"
              placeholder="Category name..."
              bind:value={newCategoryName}
              onkeydown={(e) => {
                if (e.key === "Enter") createCategory();
                if (e.key === "Escape") { showNewCategoryForm = false, categoryError = ""; }
              }}
            />
            <div class="form-buttons">
              <button onclick={createCategory}>Add</button>
              <button onclick={() => { showNewCategoryForm = false; categoryError = ""; }}>Cancel</button>
            </div>
            {#if categoryError}
              <p class="error">{categoryError}</p>
            {/if}
          </div>
        {:else}
          <button class="new-category-btn" onclick={() => showNewCategoryForm = true}>
            New Category...
          </button>
        {/if}
    </aside>

    <main class="content">
        <h2 class = "text-style">{categories.find(c => c.category_id === selectedCategoryId)?.name ?? "Select a category"}</h2>
        {#if entries.length === 0}
            <p class="empty">No games in this category yet.</p>
        {:else}
            <ul class="entry-list">
                {#each entries as entry}
                    <li class="entry-item">
                        <strong>{entry.name}</strong>
                        <span class="meta">Added: {entry.addition_date}</span>
                        {#if entry.source}
                            <span class="meta">Source: {entry.source}</span>
                        {/if}
                        {#if entry.tags.length > 0}
                            <div class="tags">
                                {#each entry.tags as tag}
                                    <span class="tag">{tag}</span>
                                {/each}
                            </div>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
          <button class="new-entry-btn" onclick={() => showAddGameModal = true}>Add new game</button>
          {#if showAddGameModal}
            <div class="add-game-overlay" onclick={closeModal}>
              <div class="add-game-modal">
                <h3>Add game to {categories.find(c => c.category_id === selectedCategoryId)?.name}</h3>
              </div>
            </div>
          {/if}
    </main>

    <Modal bind:showAddGameModal>
      {#snippet header()}
        <h2>Add a game to {categories.find(c => c.category_id === selectedCategoryId)?.name}</h2>
      {/snippet}
    </Modal>
</div>

<style>
    :global(body) {
        margin: 0;
    }
    .app-layout {
        display: flex;
        height: 100vh;
    }

    .sidebar {
        width: 250px;
        background: #262626;
        color: #eee;
        padding: 0.25rem;
        overflow-y: auto;
        resize: horizontal;
        overflow: auto;
        min-width: 150px;
        max-width: 400px;
    }

    .sidebar ul {
        list-style: none;
        padding: 0;
        margin-bottom: 0
    }

    .category-row {
      position: relative;
    }

    .category-row button:first-child {
      width: 100%;
      text-align: left;
      padding: 0.5rem;
      background: none;
      border: none;
      color: #ccc;
      cursor: pointer;
      border-radius: 4px;
    }

    .category-row button:first-child:hover{
      background: #2d0d52;
    }

    .category-row button:first-child.selected {
      background: #4d148e;
      color: #fff;
    }

    .category-row:hover .count{
      visibility: hidden;
    }

    .count {
      float: right;
      opacity: 0.6;
    }

    .delete-btn {
      position: absolute;
      top: 50%;
      right: 4px;
      transform: translateY(-50%);
      display: none;
      background: none;
      border: none;
      color: #ccc;
      cursor: pointer;
      padding: 0.2rem 0.4rem;
      font-size: 1rem;
      border-radius: 4px;
    }

    .category-row:hover .delete-btn {
      display: block;
    }

    .delete-btn:hover {
      color: #ff6b6b
    }

    .new-category-btn {
      width: 100%;
      padding: 0.5rem;
      margin-top: 0;
      background: none;
      border: 1px dashed #666;
      color: #ccc;
      cursor: pointer;
      border-radius: 4px;
    }

    .new-category-btn:hover {
      border-color: #aaa;
      color: #fff
    }

    .new-category-form {
      margin-top: 0.5rem;
    }

    new-category-form input {
      width: 100%;
      padding: 0.4rem;
      border-radius: 4px;
      border: 1px solid #444;
      background: #2a2a3e;
      color: #eee;
      box-sizing: border-box;
    }
    
    .form-buttons {
      display: flex;
      gap: 0.25rem;
      margin-top: 0.25rem;
    }

    .form-buttons button {
      flex: 1;
      padding: 0.3rem;
      border-radius: 4px;
      border: none;
      cursor: pointer;
      font-size: 0.8rem;
    }

    .error {
      color: #ff6b6b;
      font-size: 0.8rem;
      margin-top: 0.25rem;
    }

    .content {
        flex: 1;
        background: #313131;
        padding: 1rem;
        overflow-y: auto;
    }

    .content h2 {
      color: #eee
    }

    .content h2.text-style {
      font-family: 'Courier New', Courier, monospace;
    }

    .empty {
        color: #888;
        font-style: italic;
    }

    .entry-list {
        list-style: none;
        padding: 0;
    }

    .entry-item {
        padding: 0.75rem;
        border-bottom: 1px solid #eee;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .meta {
        font-size: 0.85rem;
        color: #666;
    }

    .tags {
        display: flex;
        gap: 0.25rem;
        flex-wrap: wrap;
    }

    .tag {
        background: #e0e7ff;
        color: #3730a3;
        padding: 0.1rem 0.5rem;
        border-radius: 12px;
        font-size: 0.75rem;
    }
</style>

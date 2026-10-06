<script lang="ts">
  let { showModal = $bindable(false), size = "wide", children } = $props();
</script>

{#if showModal}
  <div class="modal-overlay" onclick={() => showModal = false}>
    <div class="modal" class:narrow={size === "narrow"} onclick={(e) => e.stopPropagation()}>
      {@render children()}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .modal {
    background: #2a2a2a;
    border-radius: 8px;
    padding: 1.5rem;
    width: 80%;
    max-width: 900px;
    max-height: 80vh;
    overflow-y: auto;
    color: #eee;
  }

  .modal.narrow {
    width: fit-content;
    max-width: 700px;
  }
</style>
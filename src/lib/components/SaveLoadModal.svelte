<script lang="ts">
  import { onMount } from 'svelte';
  import { listSaves, saveGame, loadGame, deleteSave, gameState } from '../stores/gameStore';
  import type { SaveMetadata } from '../types';

  let { onclose }: { onclose: () => void } = $props();

  let slots = $state<Array<SaveMetadata | null>>(Array(5).fill(null));
  let pendingSaveSlot = $state<number | null>(null);
  let pendingName = $state('');
  let confirmOverwrite = $state<number | null>(null);
  let busy = $state(false);
  let errorMsg = $state('');

  const defaultSaveName = $derived(
    $gameState?.player?.name
      ? `${$gameState.player.name} - ${$gameState.scene || 'Unknown'}`
      : 'New Save'
  );

  onMount(async () => {
    await refresh();
  });

  async function refresh() {
    slots = await listSaves();
  }

  function clickSave(slotIdx: number) {
    if (slots[slotIdx] !== null) {
      confirmOverwrite = slotIdx;
    } else {
      openNameInput(slotIdx);
    }
  }

  function openNameInput(slotIdx: number) {
    confirmOverwrite = null;
    pendingSaveSlot = slotIdx;
    pendingName = defaultSaveName;
  }

  async function confirmSave(slotIdx: number) {
    busy = true;
    errorMsg = '';
    try {
      await saveGame(slotIdx, pendingName.trim() || defaultSaveName);
      pendingSaveSlot = null;
      pendingName = '';
      await refresh();
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function clickLoad(slotIdx: number) {
    busy = true;
    errorMsg = '';
    try {
      await loadGame(slotIdx);
      onclose();
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function clickDelete(slotIdx: number) {
    busy = true;
    errorMsg = '';
    try {
      await deleteSave(slotIdx);
      await refresh();
    } catch (e) {
      errorMsg = String(e);
    } finally {
      busy = false;
    }
  }

  function formatTimestamp(ts: string): string {
    const secs = parseInt(ts, 10);
    if (isNaN(secs)) return ts;
    return new Date(secs * 1000).toLocaleString();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="backdrop"
  onclick={onclose}
  role="button"
  tabindex="-1"
  onkeydown={() => {}}
></div>

<div class="modal">
  <div class="modal-header">
    <span class="modal-title">Save / Load</span>
    <button class="close-btn" onclick={onclose}>&#x2715;</button>
  </div>

  <div class="modal-body">
    {#each slots as meta, i}
      <div class="slot" class:occupied={meta !== null}>
        <div class="slot-info">
          <span class="slot-num">Slot {i + 1}</span>
          {#if meta}
            <span class="slot-name">{meta.name}</span>
            <span class="slot-detail">{meta.player_name} · {meta.scene}</span>
            <span class="slot-date">{formatTimestamp(meta.saved_at)}</span>
          {:else}
            <span class="slot-empty">Empty</span>
          {/if}
        </div>

        <div class="slot-actions">
          {#if confirmOverwrite === i}
            <span class="overwrite-warning">Overwrite?</span>
            <button
              class="btn-danger-sm"
              onclick={() => openNameInput(i)}
              disabled={busy}
            >
              Yes
            </button>
            <button
              class="btn-muted-sm"
              onclick={() => (confirmOverwrite = null)}
              disabled={busy}
            >
              No
            </button>
          {:else if pendingSaveSlot === i}
            <input
              class="name-input"
              bind:value={pendingName}
              placeholder="Save name..."
              disabled={busy}
            />
            <button
              class="btn-accent-sm"
              onclick={() => confirmSave(i)}
              disabled={busy}
            >
              Save
            </button>
            <button
              class="btn-muted-sm"
              onclick={() => (pendingSaveSlot = null)}
              disabled={busy}
            >
              Cancel
            </button>
          {:else}
            {#if $gameState?.phase === 'Playing' || $gameState?.phase === 'Combat'}
              <button
                class="btn-sm"
                onclick={() => clickSave(i)}
                disabled={busy}
              >
                Save
              </button>
            {/if}
            {#if meta}
              <button
                class="btn-sm accent"
                onclick={() => clickLoad(i)}
                disabled={busy}
              >
                Load
              </button>
              <button
                class="btn-sm danger"
                onclick={() => clickDelete(i)}
                disabled={busy}
              >
                Delete
              </button>
            {/if}
          {/if}
        </div>
      </div>
    {/each}

    {#if errorMsg}
      <p class="error-hint">{errorMsg}</p>
    {/if}
  </div>

  <div class="modal-footer">
    <button class="btn-cancel" onclick={onclose}>Close</button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1000;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 500px;
    max-height: 90vh;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-modal);
    z-index: 1001;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .modal-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 18px;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .modal-body {
    overflow-y: auto;
    padding: 16px 20px;
    flex: 1;
    min-height: 0;
  }

  .slot {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
  }

  .slot:last-child {
    border-bottom: none;
  }

  .slot.occupied .slot-name {
    font-weight: 600;
  }

  .slot-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .slot-num {
    font-size: 12px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .slot-name {
    font-size: 14px;
    color: var(--text-primary);
  }

  .slot-detail {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .slot-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .slot-empty {
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
  }

  .slot-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .overwrite-warning {
    font-size: 12px;
    color: var(--danger);
    font-weight: 500;
  }

  .btn-sm,
  .btn-accent-sm,
  .btn-danger-sm,
  .btn-muted-sm {
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    border: none;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-sm {
    background: var(--accent-dim);
    color: var(--accent);
  }

  .btn-sm:hover:not(:disabled) {
    background: var(--accent);
    color: white;
  }

  .btn-accent-sm {
    background: var(--accent);
    color: white;
  }

  .btn-accent-sm:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-danger-sm {
    background: rgba(220, 38, 38, 0.15);
    color: var(--danger);
  }

  .btn-danger-sm:hover:not(:disabled) {
    background: var(--danger);
    color: white;
  }

  .btn-muted-sm {
    background: var(--accent-dim);
    color: var(--text-secondary);
  }

  .btn-muted-sm:hover:not(:disabled) {
    background: var(--border);
    color: var(--text-primary);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .name-input {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-base);
    color: var(--text-primary);
    font-size: 13px;
    font-family: var(--font-ui);
    flex: 1;
    min-width: 120px;
  }

  .name-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .name-input::placeholder {
    color: var(--text-muted);
  }

  .modal-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .btn-cancel {
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    border: none;
    background: var(--accent-dim);
    color: var(--accent);
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-cancel:hover {
    background: var(--accent);
    color: white;
  }

  .error-hint {
    margin-top: 12px;
    padding: 10px;
    background: rgba(220, 38, 38, 0.1);
    border-left: 3px solid var(--danger);
    color: var(--danger);
    font-size: 12px;
    border-radius: 2px;
  }
</style>

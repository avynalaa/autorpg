<script lang="ts">
  import { activeQuests, quests } from '../stores/gameStore';

  let showCompleted = $state(false);
  let expanded = $state<string | null>(null);

  const displayQuests = $derived(
    showCompleted
      ? $quests
      : $quests.filter(q => q.status === 'Active' || q.status === 'OnHold')
  );

  function statusColor(status: string): string {
    switch (status) {
      case 'Active':    return '#a6e3a1';
      case 'Completed': return '#6c7086';
      case 'Failed':    return '#f38ba8';
      case 'OnHold':    return '#f9e2af';
      default:          return '#a6adc8';
    }
  }
</script>

<div class="panel quest-panel">
  <div class="panel-header">
    <div class="panel-title">Quests <span class="count">({$activeQuests.length})</span></div>
    <label class="toggle-label">
      <input type="checkbox" bind:checked={showCompleted} />
      <span>All</span>
    </label>
  </div>

  {#if displayQuests.length === 0}
    <div class="empty">No quests.</div>
  {:else}
    {#each displayQuests as quest}
      <div class="quest-item" class:expanded={expanded === quest.id}>
        <button class="quest-header" onclick={() => expanded = expanded === quest.id ? null : quest.id}>
          <span class="quest-dot" style="color:{statusColor(quest.status)}">&#x25CF;</span>
          <span class="quest-title">{quest.title}</span>
          <span class="quest-status" style="color:{statusColor(quest.status)}">{quest.status}</span>
        </button>

        {#if expanded === quest.id}
          <div class="quest-body">
            {#if quest.description}
              <p class="quest-desc">{quest.description}</p>
            {/if}
            {#if quest.objectives.length > 0}
              <ul class="objectives">
                {#each quest.objectives as obj}
                  <li class:done={obj.completed} class:optional={obj.optional}>
                    <span class="obj-check">{obj.completed ? '\u2713' : '\u25CB'}</span>
                    {obj.description}
                    {#if obj.optional}<span class="opt-badge">opt</span>{/if}
                  </li>
                {/each}
              </ul>
            {/if}
            {#if quest.notes.length > 0}
              <div class="quest-notes">
                {#each quest.notes as note}
                  <div class="note">\u2022 {note}</div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>

<style>
  .panel { background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px; box-shadow: var(--shadow-panel); font-family: var(--font-ui); }
  .panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
  .panel-title { font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 1px; color: var(--text-muted); }
  .count { color: var(--text-secondary); }
  .toggle-label { display: flex; align-items: center; gap: 4px; font-size: var(--font-size-xs); color: var(--text-muted); cursor: pointer; }
  .empty { color: var(--text-muted); font-size: var(--font-size-sm); }
  .quest-item { border-bottom: 1px solid var(--bg-base); }
  .quest-header { display: flex; align-items: center; gap: 6px; width: 100%; background: none; border: none; cursor: pointer; padding: 6px 0; text-align: left; }
  .quest-dot { font-size: var(--font-size-xs); }
  .quest-title { flex: 1; font-size: var(--font-size-md); color: var(--text-primary); }
  .quest-status { font-size: var(--font-size-xs); }
  .quest-body { padding: 4px 0 8px 16px; }
  .quest-desc { font-size: var(--font-size-sm); color: var(--text-secondary); margin: 0 0 6px; }
  .objectives { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  .objectives li { display: flex; align-items: center; gap: 6px; font-size: var(--font-size-sm); color: var(--text-secondary); }
  .objectives li.done { color: var(--text-muted); text-decoration: line-through; }
  .objectives li.optional { opacity: 0.7; }
  .obj-check { color: var(--success); width: 12px; }
  .opt-badge { font-size: 9px; background: var(--bg-active); padding: 0 4px; border-radius: var(--radius-full); color: var(--text-muted); }
  .quest-notes { margin-top: 6px; }
  .note { font-size: var(--font-size-sm); color: var(--text-muted); }
</style>

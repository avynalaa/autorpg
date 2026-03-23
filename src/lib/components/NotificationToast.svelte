<script lang="ts">
  import { fly } from 'svelte/transition';
  import { gameState } from '../stores/gameStore';
  import { notifications, pushNotification, dismiss } from '../stores/notificationStore';
  import type { GameState } from '../types';

  let prevState: GameState | null = null;
  let hasInit = false;

  $effect(() => {
    const gs = $gameState;
    if (!gs) return;

    // Skip on initial load — don't notify for existing state
    if (!hasInit) {
      hasInit = true;
      prevState = gs;
      return;
    }
    if (!prevState) { prevState = gs; return; }

    const prev = prevState;

    // ── Quest diffs ──────────────────────────────────────────────────────────
    const curQuests  = gs.quests?.quests    ?? [];
    const prevQuests = prev.quests?.quests  ?? [];

    for (const q of curQuests) {
      const p = prevQuests.find(x => x.id === q.id);
      if (!p) {
        pushNotification({ type: 'quest_new',  icon: '⚔', label: 'New Quest',       title: q.title });
      } else if (p.status !== q.status) {
        if (q.status === 'Completed')
          pushNotification({ type: 'quest_done', icon: '✓', label: 'Quest Complete', title: q.title });
        else if (q.status === 'Failed')
          pushNotification({ type: 'quest_fail', icon: '✗', label: 'Quest Failed',   title: q.title });
      } else {
        for (const obj of q.objectives) {
          const pObj = p.objectives.find(o => o.id === obj.id);
          if (pObj && !pObj.completed && obj.completed)
            pushNotification({ type: 'objective', icon: '◆', label: 'Objective Done', title: obj.description, body: q.title });
        }
      }
    }

    // ── Inventory diffs ──────────────────────────────────────────────────────
    const curSlots  = gs.inventory?.slots   ?? [];
    const prevSlots = prev.inventory?.slots ?? [];

    for (const slot of curSlots) {
      if (!slot.item) continue;
      const pSlot = prevSlots.find(s => s.index === slot.index);
      if (!pSlot?.item || pSlot.item.id !== slot.item.id)
        pushNotification({ type: 'item', icon: '◈', label: slot.item.category, title: slot.item.name });
    }

    // ── Relation diffs ───────────────────────────────────────────────────────
    const curRels  = Object.values(gs.relations?.relations   ?? {});
    const prevRels = Object.values(prev.relations?.relations ?? {});

    for (const rel of curRels) {
      const pRel = prevRels.find(r => r.npc_id === rel.npc_id);
      if (pRel && rel.score !== pRel.score) {
        const delta = rel.score - pRel.score;
        const sign  = delta > 0 ? '+' : '';
        pushNotification({ type: 'relation', icon: '♦', label: 'Relation', title: rel.npc_name, body: `${sign}${delta}` });
      }
    }

    prevState = gs;
  });

  const typeColor: Record<string, string> = {
    quest_new:  'var(--accent)',
    quest_done: 'var(--success)',
    quest_fail: 'var(--danger)',
    objective:  'var(--special)',
    item:       'var(--warning)',
    relation:   'var(--special)',
  };
</script>

<div class="notif-stack" aria-live="polite">
  {#each $notifications as n (n.id)}
    <div
      class="notif"
      style="--accent-color: {typeColor[n.type] ?? 'var(--accent)'}"
      in:fly={{ x: 320, duration: 280, opacity: 0 }}
      out:fly={{ x: 320, duration: 220, opacity: 0 }}
    >
      <div class="notif-bar"></div>
      <div class="notif-icon">{n.icon}</div>
      <div class="notif-body">
        <span class="notif-label">{n.label}</span>
        <span class="notif-title">{n.title}</span>
        {#if n.body}<span class="notif-sub">{n.body}</span>{/if}
      </div>
      <button class="notif-close" onclick={() => dismiss(n.id)}>✕</button>
    </div>
  {/each}
</div>

<style>
  .notif-stack {
    position: fixed;
    bottom: 24px;
    right: 20px;
    z-index: 9000;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    pointer-events: none;
    width: 280px;
  }

  .notif {
    pointer-events: all;
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-left: none;
    border-radius: var(--radius-sm);
    overflow: hidden;
    box-shadow: 0 4px 20px rgba(0,0,0,0.35);
    min-height: 54px;
    padding-right: 8px;
  }

  .notif-bar {
    width: 4px;
    align-self: stretch;
    flex-shrink: 0;
    background: var(--accent-color);
  }

  .notif-icon {
    font-size: 14px;
    color: var(--accent-color);
    flex-shrink: 0;
    width: 20px;
    text-align: center;
  }

  .notif-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 0;
  }

  .notif-label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    color: var(--accent-color);
    font-family: var(--font-ui);
  }

  .notif-title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    font-family: var(--font-ui);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notif-sub {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-family: var(--font-ui);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notif-close {
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    padding: 4px;
    flex-shrink: 0;
    opacity: 0.5;
  }
  .notif-close:hover { opacity: 1; color: var(--text-primary); }
</style>

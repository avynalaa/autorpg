<script lang="ts">
  import { combat, inCombat } from '../stores/gameStore';
  import { playerAttack } from '../stores/gameStore';

  async function attack(targetId: string) {
    await playerAttack(targetId);
  }

  // Strip === Round X === and similar backend formatting from log entries
  function cleanEntry(s: string): string {
    return s
      .replace(/^={2,}\s*(.+?)\s*={2,}$/, '$1')
      .replace(/^-{2,}\s*(.+?)\s*-{2,}$/, '$1');
  }

  function hpPct(hp: number, max: number): number {
    return max > 0 ? Math.max(0, Math.min(100, hp / max * 100)) : 0;
  }

  function hpColor(pct: number): string {
    if (pct > 60) return 'var(--success)';
    if (pct > 30) return 'var(--warning)';
    return 'var(--danger)';
  }

  // Current combatant ID
  const currentId = $derived(
    $combat?.turn_order?.[$combat?.current_turn_idx ?? 0] ?? null
  );

  // Sorted enemies (alive first)
  const enemies = $derived(
    ($combat?.combatants ?? [])
      .filter(c => !c.is_player)
      .sort((a, b) => Number(b.is_alive) - Number(a.is_alive))
  );

  const player = $derived(
    ($combat?.combatants ?? []).find(c => c.is_player) ?? null
  );
</script>

{#if $inCombat}
<div class="combat-panel">

  <!-- Header -->
  <div class="combat-header">
    <span class="combat-label">Combat</span>
    <span class="round-badge">Round {$combat?.round ?? 1}</span>
  </div>

  <!-- Turn order strip -->
  {#if ($combat?.turn_order?.length ?? 0) > 0}
    <div class="turn-strip">
      {#each ($combat?.turn_order ?? []) as id, i}
        {@const c = ($combat?.combatants ?? []).find(x => x.id === id)}
        {#if c}
          <div
            class="turn-pip"
            class:active={id === currentId}
            class:dead={!c.is_alive}
            class:is-player={c.is_player}
            title="{c.name} (initiative {c.initiative})"
          >
            {c.name.charAt(0).toUpperCase()}
          </div>
        {/if}
      {/each}
      <span class="turn-strip-label">Turn order</span>
    </div>
  {/if}

  <!-- Player row -->
  {#if player}
    <div class="combatant-row is-player" class:active-turn={player.id === currentId}>
      <div class="c-top">
        <span class="c-name">{player.name}</span>
        <span class="c-stats">ATK {player.attack} · DEF {player.defense}</span>
      </div>
      <div class="c-bar-track">
        <div class="c-bar-fill" style="width:{hpPct(player.hp, player.hp_max)}%; background:{hpColor(hpPct(player.hp, player.hp_max))}"></div>
      </div>
      <div class="c-hp">{player.hp}<span class="c-hp-max">/{player.hp_max}</span></div>
    </div>
  {/if}

  <!-- Divider -->
  <div class="vs-rule"><span>vs</span></div>

  <!-- Enemies -->
  {#each enemies as c}
    <div class="combatant-row is-enemy" class:dead={!c.is_alive} class:active-turn={c.id === currentId}>
      <div class="c-top">
        <span class="c-name">{c.name}</span>
        <span class="c-stats">ATK {c.attack} · DEF {c.defense}</span>
      </div>
      <div class="c-bar-track">
        <div class="c-bar-fill" style="width:{hpPct(c.hp, c.hp_max)}%; background:{hpColor(hpPct(c.hp, c.hp_max))}"></div>
      </div>
      <div class="c-hp-row">
        <span class="c-hp">{c.hp}<span class="c-hp-max">/{c.hp_max}</span></span>
        {#if !c.dead && c.is_alive && $combat?.state === 'PlayerTurn'}
          <button class="atk-btn" onclick={() => attack(c.id)}>Attack</button>
        {/if}
      </div>
    </div>
  {/each}

  <!-- State banner -->
  {#if $combat?.state === 'PlayerTurn'}
    <div class="state-banner your-turn">Your turn</div>
  {:else if $combat?.state === 'EnemyTurn'}
    <div class="state-banner enemy-turn">Enemy acting…</div>
  {:else if $combat?.state === 'Initiative'}
    <div class="state-banner neutral">Rolling initiative…</div>
  {:else if $combat?.state === 'Victory'}
    <div class="state-banner victory">Victory</div>
  {:else if $combat?.state === 'Defeat'}
    <div class="state-banner defeat">Defeated</div>
  {/if}

  <!-- Combat log -->
  {#if ($combat?.log?.entries?.length ?? 0) > 0}
    <div class="clog">
      {#each ($combat?.log?.entries ?? []).slice(-5).map(cleanEntry) as line}
        <div class="clog-line">{line}</div>
      {/each}
    </div>
  {/if}

</div>
{/if}

<style>
  .combat-panel {
    background: var(--bg-surface);
    border: 1px solid var(--combat-border);
    border-top: 2px solid var(--combat-title);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-family: var(--font-ui);
  }

  /* Header */
  .combat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px 6px;
    border-bottom: 1px solid var(--combat-border);
  }
  .combat-label {
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--combat-title);
    font-weight: 600;
  }
  .round-badge {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--combat-title);
    opacity: 0.7;
    background: var(--danger-dim);
    padding: 2px 7px;
    border-radius: var(--radius-full);
  }

  /* Turn order strip */
  .turn-strip {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 6px 12px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--combat-border);
    flex-wrap: wrap;
  }
  .turn-pip {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--bg-hover);
    border: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    font-weight: 700;
    color: var(--text-muted);
    transition: all var(--transition);
    cursor: default;
  }
  .turn-pip.active { background: var(--combat-title); color: var(--bg-base); border-color: var(--combat-title); transform: scale(1.15); }
  .turn-pip.is-player { border-color: var(--accent); color: var(--accent); }
  .turn-pip.is-player.active { background: var(--accent); color: var(--bg-base); }
  .turn-pip.dead { opacity: 0.25; }
  .turn-strip-label {
    margin-left: auto;
    font-size: 8px;
    color: var(--text-muted);
    letter-spacing: 0.8px;
    text-transform: uppercase;
  }

  /* Combatant rows */
  .combatant-row {
    padding: 8px 12px;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    transition: background var(--transition);
  }
  .combatant-row.active-turn { background: color-mix(in srgb, var(--accent) 5%, transparent); }
  .combatant-row.is-enemy.active-turn { background: color-mix(in srgb, var(--danger) 6%, transparent); }
  .combatant-row.dead { opacity: 0.35; }

  .c-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 5px;
  }
  .c-name {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
  }
  .is-player .c-name { color: var(--accent); }
  .is-enemy  .c-name { color: var(--danger); }
  .c-stats {
    font-size: 9px;
    color: var(--text-muted);
    letter-spacing: 0.4px;
  }

  .c-bar-track {
    height: 5px;
    background: var(--bg-hover);
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 4px;
  }
  .c-bar-fill {
    height: 100%;
    border-radius: 3px;
    transition: width 0.4s ease;
  }

  .c-hp { font-size: 10px; color: var(--text-secondary); }
  .c-hp-max { color: var(--text-muted); }
  .c-hp-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  /* vs divider */
  .vs-rule {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    margin: 2px 0;
  }
  .vs-rule::before, .vs-rule::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--combat-border);
    opacity: 0.5;
  }
  .vs-rule span {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: var(--text-muted);
  }

  /* Attack button */
  .atk-btn {
    padding: 3px 10px;
    background: var(--danger-dim);
    border: 1px solid var(--danger);
    border-radius: var(--radius-xs);
    color: var(--danger);
    font-size: 10px;
    font-family: var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    cursor: pointer;
    transition: background var(--transition), filter var(--transition);
  }
  .atk-btn:hover { filter: brightness(1.2); }

  /* State banner */
  .state-banner {
    text-align: center;
    padding: 6px 12px;
    font-size: var(--font-size-xs);
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    border-top: 1px solid transparent;
    border-bottom: 1px solid transparent;
  }
  .your-turn  { background: var(--accent-dim);  color: var(--accent);  border-color: color-mix(in srgb, var(--accent) 25%, transparent); }
  .enemy-turn { background: var(--danger-dim);  color: var(--danger);  border-color: color-mix(in srgb, var(--danger) 25%, transparent); }
  .neutral    { background: var(--bg-elevated); color: var(--text-muted); }
  .victory    { background: var(--success-dim); color: var(--success); border-color: color-mix(in srgb, var(--success) 25%, transparent); }
  .defeat     { background: var(--danger-dim);  color: var(--danger);  border-color: color-mix(in srgb, var(--danger) 25%, transparent); }

  /* Combat log */
  .clog {
    padding: 6px 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border-top: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  }
  .clog-line {
    font-size: 10px;
    color: var(--text-muted);
    line-height: 1.5;
  }
  .clog-line:last-child { color: var(--text-secondary); }
</style>

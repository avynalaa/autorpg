<script lang="ts">
  import { combat, isLoading } from '../stores/gameStore';
  import { playerAttack, sendToAI } from '../stores/gameStore';

  async function attack(targetId: string) {
    await playerAttack(targetId);
    const state = $combat?.state;
    if (state === 'Victory' || state === 'Defeat') {
      await sendToAI(state === 'Victory' ? '[Combat ended: Victory]' : '[Combat ended: Defeat]');
    }
  }

  async function flee() {
    await sendToAI('[Action: Flee]');
  }

  function hpPct(hp: number, max: number) {
    return max > 0 ? Math.max(0, Math.min(100, hp / max * 100)) : 0;
  }
  function hpColor(pct: number) {
    if (pct > 60) return 'var(--success)';
    if (pct > 30) return 'var(--warning)';
    return 'var(--danger)';
  }

  const player  = $derived(($combat?.combatants ?? []).find(c => c.is_player) ?? null);
  const enemies = $derived(($combat?.combatants ?? []).filter(c => !c.is_player));
  const isPlayerTurn = $derived($combat?.state === 'PlayerTurn');
  const isEnemyTurn  = $derived($combat?.state === 'EnemyTurn');
  const isOver       = $derived($combat?.state === 'Victory' || $combat?.state === 'Defeat' || $combat?.state === 'Fled');
</script>

<div class="combat-interface">

  <!-- Header bar -->
  <div class="ci-header">
    <span class="ci-label">Combat</span>
    <span class="ci-round">Round {$combat?.round ?? 1}</span>
    <span class="ci-divider"></span>
    {#if isPlayerTurn}
      <span class="ci-state your-turn">Your turn</span>
    {:else if isEnemyTurn}
      <span class="ci-state enemy-turn">Enemy acting…</span>
    {:else if $combat?.state === 'Initiative'}
      <span class="ci-state neutral">Rolling initiative…</span>
    {:else if $combat?.state === 'Victory'}
      <span class="ci-state victory">Victory</span>
    {:else if $combat?.state === 'Defeat'}
      <span class="ci-state defeat">Defeated</span>
    {:else if $combat?.state === 'Fled'}
      <span class="ci-state neutral">Escaped</span>
    {/if}
  </div>

  <!-- Field: player vs enemies -->
  <div class="ci-field">
    <!-- Player card -->
    {#if player}
      <div class="ci-card ci-card-player" class:active={isPlayerTurn}>
        <div class="ci-card-name">{player.name}</div>
        <div class="ci-bar-track">
          <div class="ci-bar-fill" style="width:{hpPct(player.hp, player.hp_max)}%; background:{hpColor(hpPct(player.hp, player.hp_max))}"></div>
        </div>
        <div class="ci-card-footer">
          <span class="ci-hp">{player.hp}<span class="ci-hp-max">/{player.hp_max}</span></span>
          <span class="ci-stats">ATK {player.attack} · DEF {player.defense}</span>
        </div>
      </div>
    {/if}

    <!-- Divider glyph -->
    <div class="ci-vs">⚔</div>

    <!-- Enemy cards -->
    <div class="ci-enemies">
      {#each enemies as c}
        <div class="ci-card ci-card-enemy" class:dead={!c.is_alive} class:active={c.id === ($combat?.turn_order?.[$combat?.current_turn_idx ?? 0] ?? null)}>
          <div class="ci-card-name">{c.name}</div>
          <div class="ci-bar-track">
            <div class="ci-bar-fill" style="width:{hpPct(c.hp, c.hp_max)}%; background:{hpColor(hpPct(c.hp, c.hp_max))}"></div>
          </div>
          <div class="ci-card-footer">
            <span class="ci-hp">{c.hp}<span class="ci-hp-max">/{c.hp_max}</span></span>
            <span class="ci-stats">ATK {c.attack} · DEF {c.defense}</span>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <!-- Actions -->
  {#if isPlayerTurn && !$isLoading}
    <div class="ci-actions">
      {#each enemies.filter(c => c.is_alive) as c}
        <button class="ci-btn ci-btn-attack" onclick={() => attack(c.id)}>
          <span class="ci-btn-glyph">⚔</span> Attack {c.name}
        </button>
      {/each}
      <button class="ci-btn ci-btn-flee" onclick={flee}>
        <span class="ci-btn-glyph">↩</span> Flee
      </button>
    </div>
  {:else if isEnemyTurn || $isLoading}
    <div class="ci-waiting">
      <span class="ci-waiting-dot"></span>
      <span class="ci-waiting-dot"></span>
      <span class="ci-waiting-dot"></span>
    </div>
  {/if}

</div>

<style>
  .combat-interface {
    flex-shrink: 0;
    background: color-mix(in srgb, var(--combat-border) 8%, var(--bg-elevated));
    border: 1px solid var(--combat-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    font-family: var(--font-ui);
  }

  /* Header */
  .ci-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    border-bottom: 1px solid var(--combat-border);
    background: color-mix(in srgb, var(--combat-border) 12%, var(--bg-elevated));
  }
  .ci-label {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: var(--combat-title);
    font-weight: 700;
  }
  .ci-round {
    font-size: 9px;
    color: var(--text-muted);
    letter-spacing: 1px;
    text-transform: uppercase;
  }
  .ci-divider { flex: 1; height: 1px; background: var(--combat-border); opacity: 0.4; }
  .ci-state {
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }
  .ci-state.your-turn  { background: var(--accent-dim);  color: var(--accent);  }
  .ci-state.enemy-turn { background: var(--danger-dim);  color: var(--danger);  }
  .ci-state.victory    { background: var(--success-dim); color: var(--success); }
  .ci-state.defeat     { background: var(--danger-dim);  color: var(--danger);  }
  .ci-state.neutral    { color: var(--text-muted); }

  /* Combatant field */
  .ci-field {
    display: grid;
    grid-template-columns: 1fr 28px 1fr;
    gap: 10px;
    align-items: center;
    padding: 12px 14px;
  }
  .ci-vs {
    text-align: center;
    font-size: 13px;
    color: var(--combat-title);
    opacity: 0.5;
  }
  .ci-enemies { display: flex; flex-direction: column; gap: 8px; }

  /* Cards */
  .ci-card {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 8px 10px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    transition: border-color var(--transition), background var(--transition);
  }
  .ci-card.active {
    border-color: color-mix(in srgb, var(--combat-title) 60%, var(--border));
    background: color-mix(in srgb, var(--combat-title) 6%, var(--bg-elevated));
  }
  .ci-card-player.active { border-color: color-mix(in srgb, var(--accent) 60%, var(--border)); background: color-mix(in srgb, var(--accent) 5%, var(--bg-elevated)); }
  .ci-card.dead { opacity: 0.35; }

  .ci-card-name {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
  }
  .ci-card-player .ci-card-name { color: var(--accent); }
  .ci-card-enemy  .ci-card-name { color: var(--danger); }

  .ci-bar-track {
    height: 4px;
    background: var(--bg-hover);
    border-radius: 2px;
    overflow: hidden;
  }
  .ci-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.4s ease;
  }

  .ci-card-footer {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .ci-hp { font-size: 10px; color: var(--text-secondary); }
  .ci-hp-max { color: var(--text-muted); }
  .ci-stats { font-size: 9px; color: var(--text-muted); letter-spacing: 0.3px; }

  /* Action row */
  .ci-actions {
    display: flex;
    gap: 6px;
    padding: 0 14px 12px;
    flex-wrap: wrap;
  }
  .ci-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 14px;
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    font-weight: 600;
    cursor: pointer;
    transition: filter var(--transition), transform var(--transition);
    letter-spacing: 0.3px;
  }
  .ci-btn:hover  { filter: brightness(1.15); transform: translateY(-1px); }
  .ci-btn:active { transform: translateY(0); }
  .ci-btn-glyph  { font-size: 11px; }

  .ci-btn-attack {
    background: var(--danger-dim);
    color: var(--danger);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--danger) 35%, transparent);
  }
  .ci-btn-flee {
    background: var(--bg-hover);
    color: var(--text-secondary);
    box-shadow: inset 0 0 0 1px var(--border);
    margin-left: auto;
  }

  /* Waiting dots */
  .ci-waiting {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 10px 14px 14px;
  }
  .ci-waiting-dot {
    width: 5px; height: 5px;
    border-radius: 50%;
    background: var(--combat-title);
    opacity: 0.4;
    animation: ci-dot 1.1s ease-in-out infinite;
  }
  .ci-waiting-dot:nth-child(2) { animation-delay: 0.18s; }
  .ci-waiting-dot:nth-child(3) { animation-delay: 0.36s; }
  @keyframes ci-dot {
    0%, 80%, 100% { transform: scale(0.7); opacity: 0.3; }
    40%            { transform: scale(1.2); opacity: 1; }
  }
</style>

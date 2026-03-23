<script lang="ts">
  import { player } from '../stores/gameStore';

  function mod(score: number): string {
    const m = Math.floor((score - 10) / 2);
    return m >= 0 ? `+${m}` : `${m}`;
  }

  function hpColor(hp: number, max: number): string {
    const pct = max > 0 ? hp / max : 0;
    if (pct > 0.5) return '#a6e3a1';
    if (pct > 0.25) return '#f9e2af';
    return '#f38ba8';
  }

  import type { Purse } from '../types';

  function formatPurse(purse: Purse): string {
    const parts: string[] = [];
    if (purse.platinum > 0) parts.push(`${purse.platinum}pp`);
    if (purse.gold     > 0) parts.push(`${purse.gold}gp`);
    if (purse.silver   > 0) parts.push(`${purse.silver}sp`);
    if (purse.copper   > 0) parts.push(`${purse.copper}cp`);
    return parts.length > 0 ? parts.join('  ') : '0cp';
  }
</script>

{#if $player}
  <div class="panel char-panel">
    <div class="panel-title">Character</div>

    <div class="char-name">{$player.name}</div>
    <div class="char-sub">
      {#if typeof $player.race === 'string'}
        {$player.race}
      {:else}
        {JSON.stringify($player.race).replace(/[{"'}]/g,'')}
      {/if}
      ·
      {#if typeof $player.class === 'string'}
        {$player.class}
      {:else}
        {JSON.stringify($player.class).replace(/[{"'}]/g,'')}
      {/if}
      · Lv.{$player.level}
    </div>

    <!-- HP Bar -->
    <div class="stat-row">
      <span class="stat-label">HP</span>
      <div class="bar-wrap">
        <div class="bar-fill"
          style="width:{Math.max(0,$player.hp/$player.hp_max*100)}%;background:{hpColor($player.hp,$player.hp_max)}">
        </div>
      </div>
      <span class="stat-val">{$player.hp}/{$player.hp_max}</span>
    </div>

    <!-- MP Bar -->
    {#if $player.mp_max > 0}
    <div class="stat-row">
      <span class="stat-label">MP</span>
      <div class="bar-wrap">
        <div class="bar-fill" style="width:{Math.max(0,$player.mp/$player.mp_max*100)}%;background:#818cf8"></div>
      </div>
      <span class="stat-val">{$player.mp}/{$player.mp_max}</span>
    </div>
    {/if}

    <!-- XP Bar -->
    <div class="stat-row">
      <span class="stat-label">XP</span>
      <div class="bar-wrap">
        <div class="bar-fill"
          style="width:{Math.max(0,$player.experience/$player.experience_to_next*100)}%;background:#fb923c">
        </div>
      </div>
      <span class="stat-val">{$player.experience}/{$player.experience_to_next}</span>
    </div>

    <div class="gold-row">
      <span class="gold-icon">◈</span>
      <span>{formatPurse($player.purse)}</span>
    </div>

    <!-- Ability scores -->
    <div class="abilities">
      {#each [
        ['STR', $player.ability_scores.strength],
        ['DEX', $player.ability_scores.dexterity],
        ['CON', $player.ability_scores.constitution],
        ['INT', $player.ability_scores.intelligence],
        ['WIS', $player.ability_scores.wisdom],
        ['CHA', $player.ability_scores.charisma],
      ] as [label, score]}
        <div class="ability-box">
          <div class="ability-label">{label}</div>
          <div class="ability-score">{score}</div>
          <div class="ability-mod">{mod(score)}</div>
        </div>
      {/each}
    </div>

    <!-- Status effects -->
    {#if $player.status_effects.length > 0}
      <div class="status-effects">
        {#each $player.status_effects as fx}
          <span class="status-badge" title={fx.description}>{fx.name}</span>
        {/each}
      </div>
    {/if}

    <!-- Custom stats -->
    {#if Object.keys($player.custom_stats).length > 0}
      <div class="custom-stats">
        {#each Object.entries($player.custom_stats) as [k, v]}
          <div class="custom-stat"><span>{k}</span><span>{v}</span></div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .panel { background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius-md); padding: 12px; }
  .panel-title { font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 1px; color: var(--text-muted); margin-bottom: 8px; }
  .char-name { font-size: 18px; font-weight: 700; color: var(--text-primary); }
  .char-sub { font-size: var(--font-size-sm); color: var(--accent); margin-bottom: 10px; }
  .stat-row { display: flex; align-items: center; gap: 6px; margin-bottom: 4px; }
  .stat-label { font-size: var(--font-size-sm); color: var(--text-muted); width: 20px; }
  .bar-wrap { flex: 1; height: 8px; background: var(--bg-hover); border-radius: var(--radius-xs); overflow: hidden; }
  .bar-fill { height: 100%; border-radius: var(--radius-xs); transition: width var(--transition-slow); }
  .stat-val { font-size: var(--font-size-sm); color: var(--text-secondary); width: 55px; text-align: right; }
  .gold-row { display: flex; align-items: center; gap: 6px; font-size: var(--font-size-md); color: var(--warning); margin: 8px 0; }
  .gold-icon { font-size: 14px; }
  .abilities { display: grid; grid-template-columns: repeat(3, 1fr); gap: 4px; margin-top: 10px; }
  .ability-box { background: var(--bg-elevated); border-radius: var(--radius-sm); padding: 6px 4px; text-align: center; }
  .ability-label { font-size: 9px; color: var(--text-muted); text-transform: uppercase; }
  .ability-score { font-size: 16px; font-weight: 700; color: var(--text-primary); }
  .ability-mod { font-size: var(--font-size-sm); color: var(--accent); }
  .status-effects { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 8px; }
  .status-badge { background: var(--bg-active); color: var(--text-primary); font-size: var(--font-size-xs); padding: 2px 6px; border-radius: var(--radius-full); }
  .custom-stats { margin-top: 8px; }
  .custom-stat { display: flex; justify-content: space-between; font-size: var(--font-size-sm); color: var(--text-secondary); padding: 2px 0; border-bottom: 1px solid var(--border); }
</style>

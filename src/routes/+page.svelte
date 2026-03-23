<script lang="ts">
  import { onMount } from 'svelte';
  import { gameState, loadGameState, createCharacter, resetGame, lastError, isLoading } from '$lib/stores/gameStore';
  import CharacterPanel from '$lib/components/CharacterPanel.svelte';
  import InventoryPanel from '$lib/components/InventoryPanel.svelte';
  import StoryLog from '$lib/components/StoryLog.svelte';
  import InputBar from '$lib/components/InputBar.svelte';
  import QuestLog from '$lib/components/QuestLog.svelte';
  import CombatInterface from '$lib/components/CombatInterface.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import SaveLoadModal from '$lib/components/SaveLoadModal.svelte';

  let showSettings = $state(false);
  let showSaveLoad = $state(false);
  let activeRightTab = $state<'quests' | 'inventory'>('quests');
  let leftOpen  = $state(true);
  let rightOpen = $state(true);


  let charForm = $state({ name: '', race: 'Human', charClass: 'Warrior', backstory: '' });
  let charError = $state<string | null>(null);
  const races = ['Human','Elf','Dwarf','Halfling','Orc'];
  const classes = ['Warrior','Mage','Rogue','Cleric','Ranger','Bard'];

  onMount(async () => {
    await loadGameState();
  });

  async function handleCreateChar() {
    if (!charForm.name.trim()) return;
    charError = null;
    try {
      await createCharacter({
        name: charForm.name,
        race: charForm.race,
        class: charForm.charClass,
        backstory: charForm.backstory,
      });
    } catch (e) {
      charError = String(e);
    }
  }

  async function handleNewGame() {
    await resetGame();
  }

  function formatTime(gs: typeof $gameState): string {
    if (!gs?.memory?.current_time) return '';
    const t = gs.memory.current_time;
    const h = t.hour.toString().padStart(2, '0');
    const m = t.minute.toString().padStart(2, '0');
    return `Day ${t.day + 1} \u00B7 ${h}:${m} \u00B7 ${t.season}, Year ${t.year + 1}`;
  }

  function currentHour(gs: typeof $gameState): number {
    return gs?.memory?.current_time?.hour ?? 12;
  }

  // Title-case display for location/scene names from the AI (which may return lowercase)
  function titleCase(s: string): string {
    return s.replace(/\b\w/g, c => c.toUpperCase());
  }

  const _svg = 'width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"';

  function timeIcon(hour: number): string {
    if (hour >= 6 && hour < 20) {
      // Sun
      return `<svg ${_svg} stroke-width="1.4"><circle cx="8" cy="8" r="2.8"/><line x1="8" y1="1.2" x2="8" y2="2.8"/><line x1="8" y1="13.2" x2="8" y2="14.8"/><line x1="1.2" y1="8" x2="2.8" y2="8"/><line x1="13.2" y1="8" x2="14.8" y2="8"/><line x1="3.3" y1="3.3" x2="4.4" y2="4.4"/><line x1="11.6" y1="11.6" x2="12.7" y2="12.7"/><line x1="12.7" y1="3.3" x2="11.6" y2="4.4"/><line x1="4.4" y1="11.6" x2="3.3" y2="12.7"/></svg>`;
    } else {
      // Crescent moon
      return `<svg ${_svg} stroke-width="1.4"><path d="M10 3 Q5 3 5 8 Q5 13 10 13 Q7 11.5 7 8 Q7 4.5 10 3 Z"/></svg>`;
    }
  }

  function weatherIcon(weather: string): string {
    const w = weather.toLowerCase();
    if (w.includes('storm') || w.includes('thunder'))
      return `<svg ${_svg} stroke-width="1.5"><path d="M2.5 9.5 Q1 9.5 1 7.5 Q1 5.5 3 5.5 Q3 3 6 3 Q9 3 9.5 5.5 Q11.5 5.5 11.5 7.5 Q11.5 9.5 10 9.5 Z"/><polyline points="7.5,9.5 5.5,13 8,13 6,16"/></svg>`;
    if (w.includes('snow') || w.includes('blizzard') || w.includes('flurr'))
      return `<svg ${_svg} stroke-width="1.4"><line x1="8" y1="2" x2="8" y2="14"/><line x1="2" y1="8" x2="14" y2="8"/><line x1="3.8" y1="3.8" x2="12.2" y2="12.2"/><line x1="12.2" y1="3.8" x2="3.8" y2="12.2"/><circle cx="8" cy="8" r="1.5" fill="currentColor" stroke="none"/></svg>`;
    if (w.includes('rain') || w.includes('drizzle') || w.includes('shower') || w.includes('light rain'))
      return `<svg ${_svg} stroke-width="1.4"><path d="M2.5 9 Q1 9 1 7 Q1 5 3 5 Q3 2.5 6 2.5 Q9 2.5 9.5 5 Q11.5 5 11.5 7 Q11.5 9 10 9 Z"/><line x1="4" y1="11.5" x2="3.5" y2="13.5"/><line x1="7" y1="11.5" x2="6.5" y2="13.5"/><line x1="10" y1="11.5" x2="9.5" y2="13.5"/></svg>`;
    if (w.includes('fog') || w.includes('mist') || w.includes('haze'))
      return `<svg ${_svg} stroke-width="1.4"><line x1="2" y1="5" x2="14" y2="5"/><line x1="1" y1="8" x2="15" y2="8"/><line x1="3" y1="11" x2="13" y2="11"/></svg>`;
    if (w.includes('wind') || w.includes('breezy') || w.includes('gust') || w.includes('gale'))
      return `<svg ${_svg} stroke-width="1.4"><path d="M1 6.5 Q7 3 12 5 Q14.5 6 14 8.5 Q13.5 10 11 9"/><path d="M1 10.5 Q5 8.5 9.5 10 Q12 11 11.5 13.5"/></svg>`;
    if (w.includes('cloud') || w.includes('overcast') || w.includes('grey') || w.includes('gray'))
      return `<svg ${_svg} stroke-width="1.4"><path d="M3 11 Q1 11 1 8.5 Q1 6 3.5 6 Q3.5 3.5 7 3.5 Q10.5 3.5 11 6 Q13.5 6 13.5 8.5 Q13.5 11 11 11 Z"/></svg>`;
    if (w.includes('clear') || w.includes('sun') || w.includes('fair') || w.includes('bright'))
      return `<svg ${_svg} stroke-width="1.4"><circle cx="8" cy="8" r="2.8"/><line x1="8" y1="1.2" x2="8" y2="2.8"/><line x1="8" y1="13.2" x2="8" y2="14.8"/><line x1="1.2" y1="8" x2="2.8" y2="8"/><line x1="13.2" y1="8" x2="14.8" y2="8"/><line x1="3.3" y1="3.3" x2="4.4" y2="4.4"/><line x1="11.6" y1="11.6" x2="12.7" y2="12.7"/><line x1="12.7" y1="3.3" x2="11.6" y2="4.4"/><line x1="4.4" y1="11.6" x2="3.3" y2="12.7"/></svg>`;
    // Default: partly cloudy
    return `<svg ${_svg} stroke-width="1.4"><circle cx="10.5" cy="5.5" r="2.5"/><line x1="10.5" y1="1.5" x2="10.5" y2="2.5"/><line x1="14.5" y1="5.5" x2="13.5" y2="5.5"/><line x1="13.4" y1="2.6" x2="12.7" y2="3.3"/><path d="M3.5 12 Q1.5 12 1.5 9.5 Q1.5 7.5 3.5 7.5 Q3.5 5.5 6.5 5.5 Q9.5 5.5 10 7.5 Q12 7.5 12 9.5 Q12 12 3.5 12 Z"/></svg>`;
  }

</script>

<div class="app">
  <header class="header">
    <div class="header-brand">
      <span class="logo-mark">
        {@html `<svg width="18" height="18" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round"><path d="M10 1 L12.2 7.8 L19 10 L12.2 12.2 L10 19 L7.8 12.2 L1 10 L7.8 7.8 Z"/><circle cx="10" cy="10" r="1.8" fill="currentColor" stroke="none"/></svg>`}
      </span>
      <span class="logo"><span class="logo-auto">Auto</span><span class="logo-rpg">RPG</span></span>
    </div>
    <div class="header-context">
      {#if ($gameState?.phase === 'Playing' || $gameState?.phase === 'Combat') && $gameState?.scene}
        <span class="header-scene">{titleCase($gameState.scene)}</span>
      {:else if $gameState?.phase === 'Playing' || $gameState?.phase === 'Combat'}
        <span class="header-scene">Unnamed Scene</span>
      {/if}
      {#if ($gameState?.phase === 'Playing' || $gameState?.phase === 'Combat') && (formatTime($gameState) || $gameState?.weather)}
        <div class="header-meta">
          {#if formatTime($gameState)}
            <span class="meta-chip">
              <span class="meta-icon">{@html timeIcon(currentHour($gameState))}</span>
              {formatTime($gameState)}
            </span>
          {/if}
          {#if $gameState?.weather && formatTime($gameState)}
            <span class="meta-sep"></span>
          {/if}
          {#if $gameState?.weather}
            <span class="meta-chip">
              <span class="meta-icon">{@html weatherIcon($gameState.weather)}</span>
              {$gameState.weather}
            </span>
          {/if}
        </div>
      {/if}
    </div>
    <div class="header-actions">
      {#if $gameState?.phase === 'Playing' || $gameState?.phase === 'Combat'}
        <button class="icon-btn" onclick={handleNewGame} title="New Game">&#x21BA;</button>
        <button class="icon-btn" onclick={() => showSaveLoad = true} title="Save / Load">&#x1F4BE;</button>
      {/if}
      <button class="icon-btn" onclick={() => showSettings = true} title="Settings">&#x2699;</button>
    </div>
  </header>

  {#if $gameState?.phase === 'MainMenu' || !$gameState}
    <div class="main-menu-shell">
      <div class="main-menu-backdrop"></div>
      <div class="main-menu-card">
        <div class="menu-kicker">AI-Driven Campaign Interface</div>
        <h1 class="menu-title">AutoRPG</h1>
        <p class="menu-sub">A dramatic command deck for character stories, evolving quests, and world-state-driven play.</p>
        <div class="menu-feature-grid">
          <div class="menu-feature">
            <span class="feature-label">Reactive narration</span>
            <span class="feature-copy">Scene text, combat beats, and systemic updates live in one flowing timeline.</span>
          </div>
          <div class="menu-feature">
            <span class="feature-label">Quest intelligence</span>
            <span class="feature-copy">Plans, objectives, and rewards stay visible instead of disappearing into the log.</span>
          </div>
          <div class="menu-feature">
            <span class="feature-label">World awareness</span>
            <span class="feature-copy">Track weather, time, region type, and social hooks at a glance.</span>
          </div>
        </div>
        <div class="menu-actions">
          <button class="menu-btn primary" onclick={handleNewGame}>Start New Adventure</button>
          <button class="menu-btn secondary" onclick={() => showSettings = true}>Tune the Engine</button>
        </div>
      </div>
    </div>

  {:else if $gameState?.phase === 'Playing' || $gameState?.phase === 'Combat'}
    <div class="game-layout-shell">
      <div class="game-layout">
        <aside class="panel-left" class:closed={!leftOpen}>
          <div class="panel-scroller">
            <CharacterPanel />
          </div>
          <button class="ptab ptab-left" onclick={() => leftOpen = !leftOpen}
            aria-label={leftOpen ? 'Collapse' : 'Expand'}>
            <span class="ptab-arrow">{leftOpen ? '‹' : '›'}</span>
          </button>
        </aside>

        <main class="panel-center">
          {#if $lastError}
            <div class="error-banner">{$lastError}</div>
          {/if}
          <StoryLog />
          <div class="input-container">
            {#if $gameState?.phase === 'Combat'}
              <CombatInterface />
            {:else}
              <InputBar />
            {/if}
          </div>
        </main>

        <aside class="panel-right" class:closed={!rightOpen}>
          <button class="ptab ptab-right" onclick={() => rightOpen = !rightOpen}
            aria-label={rightOpen ? 'Collapse' : 'Expand'}>
            <span class="ptab-arrow">{rightOpen ? '›' : '‹'}</span>
          </button>
          <div class="panel-scroller">
            <div class="tab-bar">
              <button class="tab" class:active={activeRightTab === 'quests'}
                onclick={() => activeRightTab = 'quests'}>Quests</button>
              <button class="tab" class:active={activeRightTab === 'inventory'}
                onclick={() => activeRightTab = 'inventory'}>Loadout</button>
            </div>
            {#if activeRightTab === 'quests'}
              <QuestLog />
            {:else}
              <InventoryPanel />
            {/if}
          </div>
        </aside>
      </div>
    </div>
  {/if}
</div>

{#if showSettings}
  <SettingsModal onclose={() => showSettings = false} />
{/if}

{#if showSaveLoad}
  <SaveLoadModal onclose={() => showSaveLoad = false} />
{/if}

{#if $gameState?.phase === 'CharacterCreation'}
  <div class="backdrop" role="presentation"></div>
  <div class="modal char-create-modal">
    <div class="modal-header">
      <span class="modal-title">Create Character</span>
    </div>
    <div class="modal-body">
      <div class="field-label">Name</div>
      <input class="text-field" bind:value={charForm.name} placeholder="Your hero's name" />

      <div class="field-label" style="margin-top:12px">Race</div>
      <div class="option-grid">
        {#each races as r}
          <button class="option-btn" class:selected={charForm.race === r}
            onclick={() => charForm.race = r}>{r}</button>
        {/each}
      </div>

      <div class="field-label" style="margin-top:12px">Class</div>
      <div class="option-grid">
        {#each classes as c}
          <button class="option-btn" class:selected={charForm.charClass === c}
            onclick={() => charForm.charClass = c}>{c}</button>
        {/each}
      </div>

      <div class="field-label" style="margin-top:12px">Backstory (optional)</div>
      <textarea class="text-area" rows="3" bind:value={charForm.backstory}
        placeholder="A few words about your character's past..."></textarea>
    </div>
    {#if charError}
      <div class="char-error">{charError}</div>
    {/if}
    <div class="modal-footer">
      <button class="btn-save" onclick={handleCreateChar}
        disabled={!charForm.name.trim()}>Begin Adventure</button>
      <button class="btn-cancel" onclick={handleNewGame}>Cancel</button>
    </div>
  </div>
{/if}

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) { margin: 0; background: var(--bg-base); color: var(--text-primary); font-family: var(--font-ui); height: 100vh; overflow: hidden; }
  .app { display: flex; flex-direction: column; height: 100vh; }
  .header {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 16px;
    padding: 0 16px;
    background: var(--bg-elevated);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    min-height: 52px;
  }
  .header-brand { display: flex; align-items: center; gap: 8px; }
  .logo-mark { display: flex; align-items: center; color: var(--accent); opacity: 0.85; }
  .logo { display: flex; align-items: baseline; gap: 1px; }
  .logo-auto { font-size: 11px; font-weight: 500; color: var(--text-muted); letter-spacing: 2.5px; text-transform: uppercase; }
  .logo-rpg  { font-size: 14px; font-weight: 800; color: var(--accent); letter-spacing: 1.5px; }
  .header-context {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    min-width: 0;
    padding: 8px 0;
  }
  .header-scene {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .header-meta {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .meta-chip {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    letter-spacing: 0.3px;
    white-space: nowrap;
  }
  .meta-icon {
    display: flex;
    align-items: center;
    opacity: 0.7;
    color: var(--accent);
  }
  .meta-icon :global(svg) { display: block; }
  .meta-sep {
    display: block;
    width: 1px;
    height: 12px;
    background: var(--border);
    opacity: 0.8;
    flex-shrink: 0;
  }
  .header-actions { display: flex; gap: 4px; align-items: center; justify-content: flex-end; }
  .icon-btn {
    width: 30px; height: 30px; background: none; border: 1px solid var(--border);
    border-radius: var(--radius-sm); color: var(--text-muted); font-size: 15px; cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: color var(--transition), border-color var(--transition), background var(--transition);
  }
  .icon-btn:hover { color: var(--accent); border-color: var(--accent); background: var(--accent-dim); }
  .game-layout-shell { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .main-menu-shell { flex: 1; position: relative; display: flex; align-items: center; justify-content: center; padding: 32px; overflow: hidden; }
  .main-menu-backdrop { position: absolute; inset: 0; background:
      radial-gradient(circle at top, color-mix(in srgb, var(--accent) 10%, transparent), transparent 45%),
      linear-gradient(180deg, color-mix(in srgb, var(--bg-elevated) 92%, transparent), var(--bg-base)); }
  .main-menu-card { position: relative; width: min(720px, 100%); display: flex; flex-direction: column; gap: 18px; padding: 32px; background: color-mix(in srgb, var(--bg-surface) 94%, transparent); border: 1px solid var(--border); border-radius: calc(var(--radius-lg) + 4px); box-shadow: var(--shadow-modal); backdrop-filter: blur(10px); }
  .menu-kicker { font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 1.8px; color: var(--text-muted); }
  .menu-title { font-size: clamp(40px, 7vw, 64px); line-height: 0.95; font-weight: 800; color: var(--text-primary); margin: 0; }
  .menu-sub { max-width: 60ch; color: var(--text-secondary); margin: 0; font-size: var(--font-size-md); line-height: 1.7; }
  .menu-feature-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; }
  .menu-feature { display: flex; flex-direction: column; gap: 6px; padding: 14px; background: color-mix(in srgb, var(--bg-elevated) 88%, transparent); border: 1px solid var(--border); border-radius: var(--radius-md); }
  .feature-label { font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 1px; color: var(--accent); }
  .feature-copy { font-size: var(--font-size-sm); line-height: 1.65; color: var(--text-secondary); }
  .menu-actions { display: flex; gap: 10px; flex-wrap: wrap; }
  .menu-btn { padding: 12px 32px; background: var(--accent-dim); border: 1px solid var(--accent); border-radius: var(--radius-md); color: var(--accent); font-size: 16px; cursor: pointer; font-family: var(--font-ui); }
  .menu-btn:hover { filter: brightness(1.08); }
  .menu-btn.secondary { background: var(--bg-elevated); border-color: var(--border); color: var(--text-secondary); }
  .game-layout { flex: 1; min-height: 0; display: flex; overflow: hidden; }

  /* ── Collapsible left panel ── */
  .panel-left {
    position: relative;
    flex-shrink: 0;
    width: var(--layout-left);
    transition: width 0.22s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
    border-right: 1px solid var(--border);
  }
  .panel-left.closed { width: 28px; }
  .panel-left .panel-scroller {
    position: absolute;
    top: 0; left: 0; bottom: 0;
    right: 28px;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: opacity 0.14s ease;
  }
  .panel-left:not(.closed) .panel-scroller::-webkit-scrollbar { width: 4px; }
  .panel-left:not(.closed) .panel-scroller::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }
  .panel-left.closed .panel-scroller { opacity: 0; pointer-events: none; }

  /* ── Collapsible right panel ── */
  .panel-right {
    position: relative;
    flex-shrink: 0;
    width: var(--layout-right);
    transition: width 0.22s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
    border-left: 1px solid var(--border);
  }
  .panel-right.closed { width: 28px; }
  .panel-right .panel-scroller {
    position: absolute;
    top: 0; right: 0; bottom: 0;
    left: 28px;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: opacity 0.14s ease;
  }
  .panel-right.closed .panel-scroller { opacity: 0; pointer-events: none; }

  /* ── Toggle tabs ── */
  .ptab {
    position: absolute;
    top: 0; bottom: 0;
    width: 28px;
    background: var(--bg-elevated);
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: color var(--transition), background var(--transition);
    z-index: 1;
  }
  .ptab:hover { color: var(--accent); background: var(--bg-hover); }
  .ptab-left  { right: 0; border-left: 1px solid var(--border); }
  .ptab-right { left: 0;  border-right: 1px solid var(--border); }
  .ptab-arrow { font-size: 11px; }

  .panel-center { flex: 1; min-width: 0; display: flex; flex-direction: column; overflow: hidden; padding: 10px; gap: 10px; min-height: 0; }
  .input-container { flex-shrink: 0; flex-grow: 0; }
  .error-banner { background: #3b1f1f; border: 1px solid var(--danger); border-radius: var(--radius-sm); padding: 8px 12px; font-size: var(--font-size-sm); color: var(--danger); flex-shrink: 0; }
  .tab-bar { display: flex; background: var(--bg-elevated); border-radius: var(--radius-sm); overflow: hidden; flex-shrink: 0; border: 1px solid var(--border); }
  .tab { flex: 1; padding: 8px 10px; background: none; border: none; color: var(--text-muted); font-size: var(--font-size-sm); cursor: pointer; font-family: var(--font-ui); }
  .tab.active { background: var(--bg-hover); color: var(--text-primary); }
  .backdrop { position: fixed; inset: 0; background: #00000088; z-index: 100; }
  .modal { position: fixed; top: 50%; left: 50%; transform: translate(-50%,-50%); z-index: 101; background: var(--bg-surface); border: 1px solid var(--border); border-radius: var(--radius-lg); width: 480px; max-width: 95vw; max-height: 90vh; display: flex; flex-direction: column; box-shadow: var(--shadow-modal); }
  .modal-header { display: flex; justify-content: space-between; padding: 14px 16px; border-bottom: 1px solid var(--border); }
  .modal-title { font-size: 15px; font-weight: 600; color: var(--text-primary); }
  .modal-body { padding: 16px; overflow-y: auto; }
  .modal-footer { padding: 12px 16px; border-top: 1px solid var(--border); display: flex; gap: 8px; justify-content: flex-end; }
  .field-label { display: block; font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-muted); margin-bottom: 6px; }
  .text-field { width: 100%; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); font-size: var(--font-size-md); padding: 8px 10px; outline: none; font-family: var(--font-ui); }
  .text-field:focus { border-color: var(--accent); }
  .text-area { width: 100%; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); font-size: var(--font-size-md); padding: 8px 10px; outline: none; resize: vertical; font-family: inherit; }
  .text-area:focus { border-color: var(--accent); }
  .option-grid { display: flex; flex-wrap: wrap; gap: 6px; }
  .option-btn { padding: 5px 12px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-secondary); font-size: var(--font-size-sm); cursor: pointer; }
  .option-btn.selected { background: var(--accent-dim); border-color: var(--accent); color: var(--accent); }
  .btn-save { padding: 8px 20px; background: var(--accent-dim); border: 1px solid var(--accent); border-radius: var(--radius-sm); color: var(--accent); font-size: var(--font-size-md); cursor: pointer; font-family: var(--font-ui); }
  .btn-save:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-cancel { padding: 8px 14px; background: var(--border); border: none; border-radius: var(--radius-sm); color: var(--text-secondary); font-size: var(--font-size-md); cursor: pointer; font-family: var(--font-ui); }
  .char-error { margin: 0 16px; padding: 8px 12px; background: #3b1f1f; border: 1px solid var(--danger); border-radius: var(--radius-sm); font-size: var(--font-size-sm); color: var(--danger); }

  @media (max-width: 800px) {
    .header { min-height: auto; padding: 8px 12px; flex-direction: column; align-items: flex-start; gap: 4px; }
    .header-context { align-items: flex-start; }
    .main-menu-shell { padding: 16px; }
    .main-menu-card { padding: 20px; }
    .menu-feature-grid { grid-template-columns: 1fr; }
  }
</style>

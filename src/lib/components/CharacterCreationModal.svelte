<script lang="ts">
  import { createCharacter, resetGame } from '$lib/stores/gameStore';
  import type { AbilityScores } from '$lib/types';

  type CreationMode = 'choice' | 'simple' | 'detailed' | 'confirm';
  type DetailedPage = 'identity' | 'persona' | 'origin' | 'stats' | 'review';
  type StatMode = 'realistic' | 'godmode';

  let mode = $state<CreationMode>('choice');
  let previousMode = $state<CreationMode | null>(null); // Track which mode we came from
  let page = $state<DetailedPage>('identity');
  let statMode = $state<StatMode>('realistic');
  let error = $state<string | null>(null);

  // Form data
  let firstName = $state('');
  let lastName = $state('');
  let age = $state<number | null>(null);
  let gender = $state('');
  let sex = $state('');
  let appearance = $state('');
  let personality = $state('');
  let race = $state('Human');
  let charClass = $state('Warrior');
  let backstory = $state('');

  // Ability scores
  let str = $state(10);
  let dex = $state(10);
  let con = $state(10);
  let int = $state(10);
  let wis = $state(10);
  let cha = $state(10);

  const races = ['Human', 'Elf', 'Dwarf', 'Halfling', 'Orc'];
  const classes = ['Warrior', 'Mage', 'Rogue', 'Cleric', 'Ranger', 'Bard'];

  // Class presets matching Rust backend
  const classPresets: Record<string, AbilityScores> = {
    Warrior: { strength: 16, dexterity: 12, constitution: 14, intelligence: 8,  wisdom: 10, charisma: 10 },
    Mage:    { strength: 8,  dexterity: 12, constitution: 10, intelligence: 16, wisdom: 14, charisma: 10 },
    Rogue:   { strength: 10, dexterity: 16, constitution: 12, intelligence: 12, wisdom: 10, charisma: 12 },
    Cleric:  { strength: 12, dexterity: 10, constitution: 12, intelligence: 10, wisdom: 16, charisma: 12 },
    Ranger:  { strength: 12, dexterity: 16, constitution: 12, intelligence: 10, wisdom: 14, charisma: 8  },
    Bard:    { strength: 8,  dexterity: 14, constitution: 10, intelligence: 12, wisdom: 10, charisma: 16 },
  };

  // Point-buy system for realistic mode
  const POINT_BUY_TOTAL = 27;
  const pointCost = (score: number): number => {
    if (score <= 13) return score - 8;
    if (score === 14) return 7;
    if (score === 15) return 9;
    return 12; // 16+
  };

  const totalPointsSpent = $derived(
    pointCost(str) + pointCost(dex) + pointCost(con) +
    pointCost(int) + pointCost(wis) + pointCost(cha)
  );
  const pointsRemaining = $derived(POINT_BUY_TOTAL - totalPointsSpent);

  // Apply class preset when class changes
  $effect(() => {
    if (mode === 'detailed' && page === 'stats') {
      const preset = classPresets[charClass];
      if (preset) {
        str = preset.strength;
        dex = preset.dexterity;
        con = preset.constitution;
        int = preset.intelligence;
        wis = preset.wisdom;
        cha = preset.charisma;
      }
    }
  });

  function startSimple() {
    mode = 'simple';
  }

  function startDetailed() {
    mode = 'detailed';
    page = 'identity';
  }

  function nextPage() {
    const pages: DetailedPage[] = ['identity', 'persona', 'origin', 'stats', 'review'];
    const idx = pages.indexOf(page);
    if (idx < pages.length - 1) {
      page = pages[idx + 1];
    }
  }

  function prevPage() {
    const pages: DetailedPage[] = ['identity', 'persona', 'origin', 'stats', 'review'];
    const idx = pages.indexOf(page);
    if (idx > 0) {
      page = pages[idx - 1];
    }
  }

  function canProceedIdentity(): boolean {
    return firstName.trim().length > 0;
  }

  function canProceedStats(): boolean {
    if (statMode === 'realistic') {
      return pointsRemaining === 0;
    }
    return true; // God mode has no restrictions
  }

  function showConfirmation() {
    // Apply class preset for simple mode before showing confirmation
    if (mode === 'simple') {
      const preset = classPresets[charClass];
      if (preset) {
        str = preset.strength;
        dex = preset.dexterity;
        con = preset.constitution;
        int = preset.intelligence;
        wis = preset.wisdom;
        cha = preset.charisma;
      }
    }
    previousMode = mode;
    mode = 'confirm';
  }

  function goBackToEdit() {
    if (previousMode === 'simple') {
      mode = 'simple';
    } else if (previousMode === 'detailed') {
      mode = 'detailed';
      page = 'review'; // Go back to review page
    }
  }

  function goBackToChoice() {
    mode = 'choice';
    previousMode = null;
  }

  async function handleFinalCreate() {
    if (!firstName.trim()) return;
    error = null;
    try {
      await createCharacter({
        name: firstName,
        last_name: lastName || null,
        age: age || null,
        gender: gender || '',
        sex: sex || '',
        appearance: appearance || '',
        personality: personality || '',
        race,
        class: charClass,
        backstory: backstory || '',
        ability_scores: {
          strength: str,
          dexterity: dex,
          constitution: con,
          intelligence: int,
          wisdom: wis,
          charisma: cha,
        },
      });
      // Character creation successful - game state will transition to Playing
    } catch (e) {
      error = String(e);
    }
  }

  async function handleCancel() {
    await resetGame();
  }

  function abilityModifier(score: number): string {
    const mod = Math.floor((score - 10) / 2);
    return mod >= 0 ? `+${mod}` : `${mod}`;
  }

  function fullName(): string {
    if (lastName.trim()) {
      return `${firstName} ${lastName}`;
    }
    return firstName;
  }
</script>

<div class="backdrop" role="presentation"></div>
<div class="modal" role="dialog" aria-modal="true">
  {#if mode === 'choice'}
    <div class="modal-header">
      <span class="modal-title">Create Character</span>
    </div>
    <div class="modal-body choice-body">
      <p class="choice-desc">Choose your character creation experience:</p>
      <button class="mode-card" onclick={startSimple}>
        <div class="mode-title">Simple Creation</div>
        <div class="mode-desc">Quick setup — name, class, race, and backstory. Perfect for jumping right in.</div>
      </button>
      <button class="mode-card" onclick={startDetailed}>
        <div class="mode-title">Detailed Creation</div>
        <div class="mode-desc">Full persona card with identity, appearance, personality, and custom ability scores.</div>
      </button>
    </div>
    <div class="modal-footer">
      <button class="btn-cancel" onclick={handleCancel}>Cancel</button>
    </div>

  {:else if mode === 'simple'}
    <div class="modal-header">
      <span class="modal-title">Simple Character Creation</span>
    </div>
    <div class="modal-body">
      <div class="field-label">First Name</div>
      <input class="text-field" bind:value={firstName} placeholder="Your hero's name" />

      <div class="field-label" style="margin-top:12px">Last Name (optional)</div>
      <input class="text-field" bind:value={lastName} placeholder="Family name" />

      <div class="field-label" style="margin-top:12px">Gender (optional)</div>
      <input class="text-field" bind:value={gender} placeholder="e.g., Male, Female, Non-binary" />

      <div class="field-label" style="margin-top:12px">Race</div>
      <div class="option-grid">
        {#each races as r}
          <button class="option-btn" class:selected={race === r} onclick={() => race = r}>{r}</button>
        {/each}
      </div>

      <div class="field-label" style="margin-top:12px">Class</div>
      <div class="option-grid">
        {#each classes as c}
          <button class="option-btn" class:selected={charClass === c} onclick={() => charClass = c}>{c}</button>
        {/each}
      </div>

      <div class="field-label" style="margin-top:12px">Backstory (optional)</div>
      <textarea class="text-area" rows="3" bind:value={backstory}
        placeholder="A few words about your character's past..."></textarea>
    </div>
    {#if error}
      <div class="char-error">{error}</div>
    {/if}
    <div class="modal-footer">
      <button class="btn-save" onclick={showConfirmation} disabled={!firstName.trim()}>Review Character</button>
      <button class="btn-cancel" onclick={() => mode = 'choice'}>Back</button>
      <button class="btn-cancel" onclick={handleCancel}>Cancel</button>
    </div>

  {:else if mode === 'detailed'}
    <div class="modal-header">
      <span class="modal-title">Detailed Character Creation</span>
      <span class="page-indicator">
        {page === 'identity' ? '1' : page === 'persona' ? '2' : page === 'origin' ? '3' : page === 'stats' ? '4' : '5'} / 5
      </span>
    </div>

    <div class="modal-body">
      {#if page === 'identity'}
        <div class="page-title">Identity</div>
        <div class="field-label">First Name</div>
        <input class="text-field" bind:value={firstName} placeholder="Required" />

        <div class="field-label" style="margin-top:12px">Last Name (optional)</div>
        <input class="text-field" bind:value={lastName} placeholder="Family name" />

        <div class="field-label" style="margin-top:12px">Age (optional)</div>
        <input class="text-field" type="number" bind:value={age} placeholder="Years" min="1" max="999" />

        <div class="field-label" style="margin-top:12px">Gender (optional)</div>
        <input class="text-field" bind:value={gender} placeholder="e.g., Male, Female, Non-binary" />

        <div class="field-label" style="margin-top:12px">Sex (optional)</div>
        <input class="text-field" bind:value={sex} placeholder="e.g., Male, Female" />

      {:else if page === 'persona'}
        <div class="page-title">Persona</div>
        <div class="field-label">Appearance</div>
        <textarea class="text-area" rows="4" bind:value={appearance}
          placeholder="How you look — height, build, hair, eyes, distinguishing features..."></textarea>

        <div class="field-label" style="margin-top:12px">Personality</div>
        <textarea class="text-area" rows="4" bind:value={personality}
          placeholder="How you act and think — traits, mannerisms, values, quirks..."></textarea>

      {:else if page === 'origin'}
        <div class="page-title">Origin</div>
        <div class="field-label">Race</div>
        <div class="option-grid">
          {#each races as r}
            <button class="option-btn" class:selected={race === r} onclick={() => race = r}>{r}</button>
          {/each}
        </div>

        <div class="field-label" style="margin-top:12px">Class</div>
        <div class="option-grid">
          {#each classes as c}
            <button class="option-btn" class:selected={charClass === c} onclick={() => charClass = c}>{c}</button>
          {/each}
        </div>

        <div class="field-label" style="margin-top:12px">Backstory</div>
        <textarea class="text-area" rows="4" bind:value={backstory}
          placeholder="Your character's history, motivations, and past..."></textarea>

      {:else if page === 'stats'}
        <div class="page-title">Ability Scores</div>
        <div class="stat-mode-toggle">
          <button class="stat-mode-btn" class:active={statMode === 'realistic'} onclick={() => statMode = 'realistic'}>
            Realistic (Point-Buy)
          </button>
          <button class="stat-mode-btn" class:active={statMode === 'godmode'} onclick={() => statMode = 'godmode'}>
            God Mode (Free Entry)
          </button>
        </div>

        {#if statMode === 'realistic'}
          <div class="points-display">
            Points Remaining: <span class:points-ok={pointsRemaining === 0} class:points-over={pointsRemaining < 0}>{pointsRemaining}</span> / {POINT_BUY_TOTAL}
          </div>
        {/if}

        <div class="stat-grid">
          <div class="stat-row">
            <span class="stat-label">STR</span>
            <input class="stat-input" type="number" bind:value={str} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(str)}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">DEX</span>
            <input class="stat-input" type="number" bind:value={dex} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(dex)}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">CON</span>
            <input class="stat-input" type="number" bind:value={con} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(con)}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">INT</span>
            <input class="stat-input" type="number" bind:value={int} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(int)}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">WIS</span>
            <input class="stat-input" type="number" bind:value={wis} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(wis)}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">CHA</span>
            <input class="stat-input" type="number" bind:value={cha} min={statMode === 'realistic' ? 8 : 1} max={statMode === 'realistic' ? 16 : 99} />
            <span class="stat-mod">{abilityModifier(cha)}</span>
          </div>
        </div>

      {:else if page === 'review'}
        <div class="page-title">Review Your Character</div>
        <div class="review-section">
          <div class="review-label">Name</div>
          <div class="review-value">{fullName()}</div>
        </div>
        {#if age}
          <div class="review-section">
            <div class="review-label">Age</div>
            <div class="review-value">{age}</div>
          </div>
        {/if}
        {#if gender}
          <div class="review-section">
            <div class="review-label">Gender</div>
            <div class="review-value">{gender}</div>
          </div>
        {/if}
        {#if sex}
          <div class="review-section">
            <div class="review-label">Sex</div>
            <div class="review-value">{sex}</div>
          </div>
        {/if}
        <div class="review-section">
          <div class="review-label">Race & Class</div>
          <div class="review-value">{race} {charClass}</div>
        </div>
        {#if appearance}
          <div class="review-section">
            <div class="review-label">Appearance</div>
            <div class="review-value">{appearance}</div>
          </div>
        {/if}
        {#if personality}
          <div class="review-section">
            <div class="review-label">Personality</div>
            <div class="review-value">{personality}</div>
          </div>
        {/if}
        {#if backstory}
          <div class="review-section">
            <div class="review-label">Backstory</div>
            <div class="review-value">{backstory}</div>
          </div>
        {/if}
        <div class="review-section">
          <div class="review-label">Ability Scores</div>
          <div class="review-stats">
            <span>STR {str} ({abilityModifier(str)})</span>
            <span>DEX {dex} ({abilityModifier(dex)})</span>
            <span>CON {con} ({abilityModifier(con)})</span>
            <span>INT {int} ({abilityModifier(int)})</span>
            <span>WIS {wis} ({abilityModifier(wis)})</span>
            <span>CHA {cha} ({abilityModifier(cha)})</span>
          </div>
        </div>
      {/if}
    </div>

    {#if error}
      <div class="char-error">{error}</div>
    {/if}

    <div class="modal-footer">
      {#if page === 'review'}
        <button class="btn-save" onclick={showConfirmation}>Review Character</button>
      {:else if page === 'identity'}
        <button class="btn-save" onclick={nextPage} disabled={!canProceedIdentity()}>Next</button>
      {:else if page === 'stats'}
        <button class="btn-save" onclick={nextPage} disabled={!canProceedStats()}>Next</button>
      {:else}
        <button class="btn-save" onclick={nextPage}>Next</button>
      {/if}
      
      {#if page !== 'identity'}
        <button class="btn-cancel" onclick={prevPage}>Back</button>
      {:else}
        <button class="btn-cancel" onclick={() => mode = 'choice'}>Back</button>
      {/if}
    </div>

  {:else if mode === 'confirm'}
    <div class="modal-header">
      <span class="modal-title">Confirm Your Character</span>
    </div>
    <div class="modal-body">
      <div class="confirm-title">{fullName()}</div>
      <div class="confirm-subtitle">{race} {charClass}</div>
      
      <div class="ability-sheet">
        <div class="ability-row">
          <div class="ability-box">
            <div class="ability-name">STR</div>
            <div class="ability-score">{str}</div>
            <div class="ability-mod">{abilityModifier(str)}</div>
          </div>
          <div class="ability-box">
            <div class="ability-name">DEX</div>
            <div class="ability-score">{dex}</div>
            <div class="ability-mod">{abilityModifier(dex)}</div>
          </div>
          <div class="ability-box">
            <div class="ability-name">CON</div>
            <div class="ability-score">{con}</div>
            <div class="ability-mod">{abilityModifier(con)}</div>
          </div>
        </div>
        <div class="ability-row">
          <div class="ability-box">
            <div class="ability-name">INT</div>
            <div class="ability-score">{int}</div>
            <div class="ability-mod">{abilityModifier(int)}</div>
          </div>
          <div class="ability-box">
            <div class="ability-name">WIS</div>
            <div class="ability-score">{wis}</div>
            <div class="ability-mod">{abilityModifier(wis)}</div>
          </div>
          <div class="ability-box">
            <div class="ability-name">CHA</div>
            <div class="ability-score">{cha}</div>
            <div class="ability-mod">{abilityModifier(cha)}</div>
          </div>
        </div>
      </div>

      {#if age || gender || sex}
        <div class="confirm-section">
          <div class="confirm-section-title">Identity</div>
          {#if age}<div class="confirm-detail">Age: {age}</div>{/if}
          {#if gender}<div class="confirm-detail">Gender: {gender}</div>{/if}
          {#if sex}<div class="confirm-detail">Sex: {sex}</div>{/if}
        </div>
      {/if}

      {#if appearance}
        <div class="confirm-section">
          <div class="confirm-section-title">Appearance</div>
          <div class="confirm-detail">{appearance}</div>
        </div>
      {/if}

      {#if personality}
        <div class="confirm-section">
          <div class="confirm-section-title">Personality</div>
          <div class="confirm-detail">{personality}</div>
        </div>
      {/if}

      {#if backstory}
        <div class="confirm-section">
          <div class="confirm-section-title">Backstory</div>
          <div class="confirm-detail">{backstory}</div>
        </div>
      {/if}
    </div>

    {#if error}
      <div class="char-error">{error}</div>
    {/if}

    <div class="modal-footer">
      <button class="btn-save" onclick={handleFinalCreate}>Begin Adventure</button>
      <button class="btn-cancel" onclick={goBackToEdit}>Edit Character</button>
      <button class="btn-cancel" onclick={goBackToChoice}>Change Mode</button>
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: #00000088;
    z-index: 100;
  }

  .modal {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 101;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    width: 520px;
    max-width: 95vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-modal);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .page-indicator {
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    font-weight: 500;
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .choice-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .choice-desc {
    margin: 0 0 8px 0;
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .mode-card {
    padding: 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: border-color var(--transition), background var(--transition);
  }

  .mode-card:hover {
    border-color: var(--accent);
    background: var(--bg-hover);
  }

  .mode-title {
    font-size: var(--font-size-md);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .mode-desc {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    line-height: 1.5;
  }

  .page-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 16px;
  }

  .modal-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--border);
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .field-label {
    display: block;
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .text-field {
    width: 100%;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-md);
    padding: 8px 10px;
    outline: none;
    font-family: var(--font-ui);
  }

  .text-field:focus {
    border-color: var(--accent);
  }

  .text-area {
    width: 100%;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-md);
    padding: 8px 10px;
    outline: none;
    resize: vertical;
    font-family: inherit;
  }

  .text-area:focus {
    border-color: var(--accent);
  }

  .option-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .option-btn {
    padding: 5px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition);
  }

  .option-btn:hover {
    border-color: var(--accent);
  }

  .option-btn.selected {
    background: var(--accent-dim);
    border-color: var(--accent);
    color: var(--accent);
  }

  .stat-mode-toggle {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .stat-mode-btn {
    flex: 1;
    padding: 8px 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
    cursor: pointer;
    transition: all var(--transition);
  }

  .stat-mode-btn:hover {
    border-color: var(--accent);
  }

  .stat-mode-btn.active {
    background: var(--accent-dim);
    border-color: var(--accent);
    color: var(--accent);
  }

  .points-display {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: 12px;
    text-align: center;
  }

  .points-ok {
    color: var(--success);
    font-weight: 600;
  }

  .points-over {
    color: var(--danger);
    font-weight: 600;
  }

  .stat-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .stat-row {
    display: grid;
    grid-template-columns: 50px 1fr 60px;
    align-items: center;
    gap: 12px;
  }

  .stat-label {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
  }

  .stat-input {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-md);
    padding: 6px 10px;
    outline: none;
    font-family: var(--font-ui);
    text-align: center;
  }

  .stat-input:focus {
    border-color: var(--accent);
  }

  .stat-mod {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    text-align: center;
  }

  .review-section {
    margin-bottom: 16px;
  }

  .review-label {
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  .review-value {
    font-size: var(--font-size-md);
    color: var(--text-primary);
    line-height: 1.5;
  }

  .review-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
    font-size: var(--font-size-sm);
    color: var(--text-primary);
  }

  .btn-save {
    padding: 8px 20px;
    background: var(--accent-dim);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    color: var(--accent);
    font-size: var(--font-size-md);
    cursor: pointer;
    font-family: var(--font-ui);
    transition: filter var(--transition);
  }

  .btn-save:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-save:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-cancel {
    padding: 8px 14px;
    background: var(--border);
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: var(--font-size-md);
    cursor: pointer;
    font-family: var(--font-ui);
    transition: background var(--transition);
  }

  .btn-cancel:hover {
    background: var(--bg-hover);
  }

  .char-error {
    margin: 0 16px;
    padding: 8px 12px;
    background: #3b1f1f;
    border: 1px solid var(--danger);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    color: var(--danger);
  }

  /* Confirmation Screen Styles */
  .confirm-title {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-primary);
    text-align: center;
    margin-bottom: 4px;
  }

  .confirm-subtitle {
    font-size: var(--font-size-md);
    color: var(--text-muted);
    text-align: center;
    margin-bottom: 20px;
  }

  .ability-sheet {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 20px;
    padding: 16px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .ability-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .ability-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }

  .ability-name {
    font-size: var(--font-size-xs);
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .ability-score {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
    margin-bottom: 2px;
  }

  .ability-mod {
    font-size: var(--font-size-sm);
    color: var(--accent);
    font-weight: 600;
  }

  .confirm-section {
    margin-bottom: 16px;
  }

  .confirm-section-title {
    font-size: var(--font-size-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-muted);
    margin-bottom: 6px;
  }

  .confirm-detail {
    font-size: var(--font-size-md);
    color: var(--text-primary);
    line-height: 1.5;
  }
</style>

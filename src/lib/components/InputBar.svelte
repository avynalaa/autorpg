<script lang="ts">
  import { sendToAI, isLoading, choices } from '../stores/gameStore';

  let input = $state('');

  async function submit() {
    const text = input.trim();
    if (!text || $isLoading) return;
    input = '';
    await sendToAI(text);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  // Belt-and-suspenders: strip emoji even if the model ignores the system prompt rule
  const emojiRe = /[\u{1F000}-\u{1FFFF}\u{2600}-\u{27FF}\u{FE00}-\u{FE0F}]/gu;
  function stripEmoji(s: string): string { return s.replace(emojiRe, '').replace(/\s{2,}/g, ' ').trim(); }

  async function pickChoice(id: string, text: string) {
    if ($isLoading) return;
    await sendToAI(`[Choice: ${stripEmoji(text)}]`);
  }

  function choiceType(style: unknown): string {
    if (style === 'Danger')  return 'danger';
    if (style === 'Social')  return 'social';
    if (style === 'Special') return 'special';
    return 'normal';
  }

  function choiceGlyph(style: unknown): string {
    if (style === 'Danger')  return '\u2694';  // ⚔
    if (style === 'Social')  return '\u25C6';  // ◆
    if (style === 'Special') return '\u2726';  // ✦
    return '\u203A';                           // ›
  }

  const sendSVG = `<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><line x1="2" y1="8" x2="13" y2="8"/><polyline points="9,4 13,8 9,12"/></svg>`;
</script>

<div class="input-area">
  {#if $choices.length > 0}
    <div class="choices">
      {#each $choices as choice}
        <button
          class="choice-btn choice-{choiceType(choice.style)}"
          onclick={() => pickChoice(choice.id, choice.text)}
          disabled={$isLoading}
          title={choice.text}
        >
          <span class="choice-pip"></span>
          <span class="choice-glyph">{choiceGlyph(choice.style)}</span>
          <span class="choice-text">{stripEmoji(choice.text)}</span>
        </button>
      {/each}
    </div>
  {/if}

  <div class="input-bar" class:is-loading={$isLoading}>
    <span class="input-accent"></span>
    <textarea
      class="text-input"
      bind:value={input}
      placeholder="What do you do?"
      rows="2"
      onkeydown={onKey}
      disabled={$isLoading}
    ></textarea>
    <button class="send-btn" onclick={submit} disabled={$isLoading || !input.trim()}>
      {#if $isLoading}
        <span class="spinner">&#x25CC;</span>
      {:else}
        {@html sendSVG}
      {/if}
    </button>
  </div>
</div>

<style>
  .input-area { display: flex; flex-direction: column; gap: 8px; }

  /* ── Choice grid ─────────────────────────────────────────────────────────── */
  .choices {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 5px;
  }

  .choice-btn {
    display: flex;
    align-items: stretch;
    gap: 0;
    padding: 0;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    overflow: hidden;
    font-family: var(--font-ui);
    font-size: var(--font-size-md);
    background: var(--bg-elevated);
    transition: filter var(--transition), transform var(--transition);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .choice-btn:disabled { opacity: 0.35; cursor: not-allowed; }
  .choice-btn:not(:disabled):hover { filter: brightness(1.14); transform: translateY(-1px); }
  .choice-btn:not(:disabled):active { transform: translateY(0); filter: brightness(1.05); }

  /* Type-coloured left pip */
  .choice-pip {
    width: 3px;
    flex-shrink: 0;
    align-self: stretch;
  }
  .choice-normal  .choice-pip  { background: var(--border); }
  .choice-danger  .choice-pip  { background: var(--danger); }
  .choice-social  .choice-pip  { background: var(--success); }
  .choice-special .choice-pip  { background: var(--special); }

  /* Type icon */
  .choice-glyph {
    flex-shrink: 0;
    width: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    padding: 10px 0;
  }
  .choice-normal  .choice-glyph { color: var(--text-muted); }
  .choice-danger  .choice-glyph { color: var(--danger); }
  .choice-social  .choice-glyph { color: var(--success); }
  .choice-special .choice-glyph { color: var(--special); }

  /* Text */
  .choice-text {
    flex: 1;
    padding: 9px 12px 9px 2px;
    line-height: 1.4;
    color: var(--text-primary);
  }

  /* Background tint per type */
  .choice-normal  { background: var(--bg-elevated); box-shadow: inset 0 0 0 1px var(--border); }
  .choice-danger  { background: color-mix(in srgb, var(--danger-dim) 85%, var(--bg-elevated));  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--danger)  30%, transparent); }
  .choice-social  { background: color-mix(in srgb, var(--success-dim) 85%, var(--bg-elevated)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--success) 30%, transparent); }
  .choice-special { background: color-mix(in srgb, var(--special-dim) 85%, var(--bg-elevated)); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--special) 30%, transparent); }

  /* ── Input bar ───────────────────────────────────────────────────────────── */
  .input-bar {
    display: flex;
    align-items: stretch;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: border-color var(--transition), box-shadow var(--transition);
  }
  .input-bar:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--border-accent);
  }
  .input-bar.is-loading {
    border-color: color-mix(in srgb, var(--accent) 35%, var(--border));
    animation: bar-pulse 2s ease-in-out infinite;
  }
  @keyframes bar-pulse {
    0%, 100% { box-shadow: 0 0 0 0 transparent; }
    50%       { box-shadow: 0 0 0 2px var(--border-accent); }
  }

  /* Left accent strip */
  .input-accent {
    width: 4px;
    flex-shrink: 0;
    background: color-mix(in srgb, var(--border) 80%, transparent);
    transition: background var(--transition);
  }
  .input-bar:focus-within .input-accent { background: var(--accent); }
  .input-bar.is-loading   .input-accent {
    background: var(--accent);
    animation: accent-blink 1.2s steps(2) infinite;
  }
  @keyframes accent-blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.3; }
  }

  .text-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: var(--font-size-md);
    padding: 10px 12px;
    resize: none;
    font-family: var(--font-ui);
    outline: none;
    line-height: 1.55;
  }
  .text-input:disabled { opacity: 0.5; }
  .text-input::placeholder { color: var(--text-muted); opacity: 0.6; font-style: italic; }

  /* Send button */
  .send-btn {
    flex-shrink: 0;
    width: 46px;
    background: none;
    border: none;
    border-left: 1px solid var(--border);
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color var(--transition), background var(--transition), border-color var(--transition);
  }
  .send-btn:not(:disabled):hover {
    background: var(--accent-dim);
    color: var(--accent);
    border-left-color: color-mix(in srgb, var(--accent) 50%, var(--border));
  }
  .input-bar:focus-within .send-btn:not(:disabled) {
    color: color-mix(in srgb, var(--accent) 70%, var(--text-muted));
  }
  .input-bar:focus-within .send-btn:not(:disabled):hover {
    color: var(--accent);
    background: var(--accent-dim);
  }
  .send-btn:disabled { opacity: 0.3; cursor: not-allowed; }
  .send-btn :global(svg) { display: block; }

  .spinner {
    display: inline-block;
    font-size: 16px;
    animation: spin 1s linear infinite;
    color: var(--accent);
  }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>

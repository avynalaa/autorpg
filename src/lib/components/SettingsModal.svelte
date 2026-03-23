<script lang="ts">
  import { onMount } from 'svelte';
  import {
    setApiKey, setModel, setEndpoint, setApiFormat,
    getSettings, fetchModels, getSystemPrompt, setSystemPrompt, resetSystemPrompt,
    setHistoryMode, setHistoryLimit, setMaxTokens
  } from '../stores/gameStore';
  import { theme, setTheme, THEMES } from '../stores/themeStore';
  import type { Theme } from '../stores/themeStore';

  let { onclose }: { onclose: () => void } = $props();

  let endpoint   = $state('https://api.anthropic.com');
  let apiFormat  = $state('auto');
  let apiKey     = $state('');
  let modelInput = $state('claude-haiku-4-5-20251001');
  let modelList  = $state<string[]>([]);
  let fetchingModels = $state(false);
  let fetchError = $state('');
  let sysPrompt  = $state('');
  let historyMode = $state('turns');
  let historyLimit = $state(20);
  let maxTokens = $state(1024);
  let saved      = $state(false);
  let saveError  = $state('');

  const formats = [
    { id: 'auto',      label: 'Auto-detect from URL' },
    { id: 'anthropic', label: 'Anthropic (v1/messages)' },
    { id: 'openai',    label: 'OpenAI-compatible (v1/chat/completions)' },
  ];

  onMount(async () => {
    const [settings, prompt] = await Promise.all([getSettings(), getSystemPrompt()]);
    endpoint   = settings.endpoint;
    apiFormat  = settings.api_format;
    modelInput = settings.model;
    sysPrompt  = prompt;
    historyMode = settings.history_mode;
    historyLimit = settings.history_limit;
    maxTokens = settings.max_tokens;
  });

  async function loadModels() {
    fetchingModels = true;
    fetchError = '';
    try {
      await setEndpoint(endpoint);
      await setApiFormat(apiFormat);
      if (apiKey.trim()) await setApiKey(apiKey.trim());
      modelList = await fetchModels();
    } catch (e) {
      fetchError = String(e);
      modelList = [];
    } finally {
      fetchingModels = false;
    }
  }

  function pickModel(id: string) { modelInput = id; }

  async function save() {
    saveError = '';
    try {
      await setEndpoint(endpoint.trim());
      await setApiFormat(apiFormat);
      if (apiKey.trim()) await setApiKey(apiKey.trim());
      await setModel(modelInput.trim());
      await setSystemPrompt(sysPrompt);
      await setHistoryMode(historyMode);
      await setHistoryLimit(historyLimit);
      await setMaxTokens(maxTokens);
      saved = true;
      setTimeout(() => { saved = false; }, 2000);
    } catch (e) {
      saveError = String(e);
    }
  }

  async function resetPrompt() {
    await resetSystemPrompt();
    sysPrompt = await getSystemPrompt();
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose} role="button" tabindex="-1" onkeydown={() => {}}></div>

<div class="modal">
  <div class="modal-header">
    <span class="modal-title">Settings</span>
    <button class="close-btn" onclick={onclose}>&#x2715;</button>
  </div>

  <div class="modal-body">

    <!-- Theme -->
    <div class="section-label">Theme</div>
    <div class="theme-grid">
      {#each THEMES as t}
        <button
          class="theme-card"
          class:active={$theme === t.id}
          onclick={() => setTheme(t.id as Theme)}
        >
          <div class="theme-swatch" data-id={t.id}>
            <span class="swatch-bar accent"></span>
            <span class="swatch-bar surface"></span>
            <span class="swatch-bar base"></span>
          </div>
          <div class="theme-info">
            <span class="theme-name">{t.label}</span>
            <span class="theme-desc">{t.description}</span>
          </div>
          {#if $theme === t.id}
            <span class="theme-check">✓</span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Endpoint URL -->
    <div class="section-label" style="margin-top:18px">API Endpoint</div>
    <input class="text-field" bind:value={endpoint} placeholder="https://api.anthropic.com" />
    <p class="hint">Works with Anthropic, OpenAI, Ollama, LM Studio, OpenRouter, or any OpenAI-compatible endpoint.</p>

    <!-- API Format -->
    <div class="section-label" style="margin-top:14px">API Format</div>
    <select class="select-field" bind:value={apiFormat}>
      {#each formats as f}
        <option value={f.id}>{f.label}</option>
      {/each}
    </select>

    <!-- API Key -->
    <div class="section-label" style="margin-top:14px">API Key</div>
    <input class="text-field" type="password" bind:value={apiKey} placeholder="Leave blank to keep existing key" />
    <p class="hint">Stored in memory only — never written to disk.</p>

    <!-- Model -->
    <div class="model-row">
      <div class="section-label" style="margin-top:14px">Model</div>
      <button class="fetch-btn" onclick={loadModels} disabled={fetchingModels}>
        {fetchingModels ? 'Fetching...' : 'Fetch models'}
      </button>
    </div>
    <input class="text-field" bind:value={modelInput} placeholder="e.g. claude-haiku-4-5-20251001 or gpt-4o" />

    {#if fetchError}
      <p class="error-hint">{fetchError}</p>
    {/if}

    {#if modelList.length > 0}
      <div class="model-list">
        {#each modelList as m}
          <button class="model-item" class:selected={modelInput === m} onclick={() => pickModel(m)}>{m}</button>
        {/each}
      </div>
    {/if}

    <!-- System Prompt -->
    <div class="model-row">
      <div class="section-label" style="margin-top:16px">System Prompt</div>
      <button class="reset-btn" onclick={resetPrompt}>Reset to default</button>
    </div>
    <textarea class="text-area" rows="8" bind:value={sysPrompt}></textarea>
    <p class="hint">The instruction given to the AI before every session.</p>

    <!-- Conversation History -->
    <div class="section-label" style="margin-top:18px">Conversation History</div>
    <div class="history-mode-row">
      <label class="radio-label">
        <input type="radio" bind:group={historyMode} value="turns" />
        Turns
      </label>
      <label class="radio-label">
        <input type="radio" bind:group={historyMode} value="tokens" />
        Tokens (estimated)
      </label>
    </div>
    {#if historyMode === 'turns'}
      <input class="text-field" type="number" bind:value={historyLimit} min="1" max="200" />
      <p class="hint">Keep the last <strong>{historyLimit}</strong> back-and-forth exchanges (each exchange = 1 user message + 1 AI reply). The AI never sees older history but it stays in memory for saves.</p>
    {:else}
      <input class="text-field" type="number" bind:value={historyLimit} min="512" max="32000" step="512" />
      <p class="hint">Keep approximately <strong>{historyLimit}</strong> tokens of history (rough estimate: characters / 4). Older messages are dropped when the total exceeds this.</p>
    {/if}

    <!-- Max Response Tokens -->
    <div class="section-label" style="margin-top:18px">Max Response Tokens</div>
    <input class="text-field" type="number" bind:value={maxTokens} min="256" max="8192" step="256" />
    <p class="hint">Maximum tokens the AI can generate per response. Too low and replies get cut off mid-sentence. Default: 1024.</p>

  </div>

  <div class="modal-footer">
    {#if saveError}
      <span class="save-error">{saveError}</span>
    {/if}
    <button class="btn-save" onclick={save}>
      {#if saved}&#x2713; Saved{:else}Save{/if}
    </button>
    <button class="btn-cancel" onclick={onclose}>Cancel</button>
  </div>
</div>

<style>
  .backdrop { position: fixed; inset: 0; background: #00000088; z-index: 100; }
  .modal {
    position: fixed; top: 50%; left: 50%; transform: translate(-50%,-50%);
    z-index: 101; background: var(--bg-surface); border: 1px solid var(--border);
    border-radius: var(--radius-lg); width: 540px; max-width: 95vw; max-height: 90vh;
    display: flex; flex-direction: column; box-shadow: var(--shadow-modal);
    font-family: var(--font-ui);
  }
  .modal-header {
    display: flex; justify-content: space-between; align-items: center;
    padding: 14px 16px; border-bottom: 1px solid var(--border);
  }
  .modal-title { font-size: 15px; font-weight: 600; color: var(--text-primary); }
  .close-btn { background: none; border: none; color: var(--text-muted); font-size: 16px; cursor: pointer; }
  .modal-body { padding: 16px; overflow-y: auto; flex: 1; }
  .modal-footer {
    padding: 12px 16px; border-top: 1px solid var(--border);
    display: flex; gap: 8px; justify-content: flex-end; align-items: center;
  }

  .section-label {
    font-size: var(--font-size-xs); text-transform: uppercase; letter-spacing: 0.5px;
    color: var(--text-muted); margin-bottom: 8px;
  }

  /* ── Theme picker ── */
  .theme-grid { display: flex; flex-direction: column; gap: 6px; }
  .theme-card {
    display: flex; align-items: center; gap: 12px; padding: 10px 12px;
    background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--radius-sm); cursor: pointer; text-align: left;
    font-family: var(--font-ui); transition: border-color var(--transition), background var(--transition);
  }
  .theme-card:hover { border-color: var(--accent); }
  .theme-card.active { border-color: var(--accent); background: var(--accent-dim); }

  .theme-swatch { display: flex; flex-direction: column; gap: 3px; width: 28px; flex-shrink: 0; }
  .swatch-bar { display: block; height: 5px; border-radius: 2px; }

  /* hardcoded swatch colors — intentional, these preview the theme itself */
  [data-id="slate"] .accent  { background: #89b4fa; }
  [data-id="slate"] .surface { background: #1e1e2e; }
  [data-id="slate"] .base    { background: #11111b; }
  [data-id="codex"] .accent  { background: #d4943a; }
  [data-id="codex"] .surface { background: #1c1710; }
  [data-id="codex"] .base    { background: #0d0b08; }
  [data-id="wire"]  .accent  { background: #39ff14; }
  [data-id="wire"]  .surface { background: #040804; }
  [data-id="wire"]  .base    { background: #000000; outline: 1px solid #1e6b1e; }

  .theme-info { flex: 1; display: flex; flex-direction: column; gap: 2px; }
  .theme-name { font-size: var(--font-size-md); color: var(--text-primary); font-weight: 600; }
  .theme-desc { font-size: var(--font-size-xs); color: var(--text-muted); }
  .theme-check { color: var(--accent); font-size: 14px; font-weight: 700; }

  /* ── Fields ── */
  .text-field, .select-field {
    width: 100%; background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--radius-sm); color: var(--text-primary);
    font-size: var(--font-size-md); padding: 8px 10px; outline: none;
    box-sizing: border-box; font-family: var(--font-ui);
    transition: border-color var(--transition);
  }
  .text-field:focus, .select-field:focus { border-color: var(--accent); }
  .text-area {
    width: 100%; background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--radius-sm); color: var(--text-primary);
    font-size: var(--font-size-sm); padding: 8px 10px; outline: none;
    resize: vertical; font-family: var(--font-mono); box-sizing: border-box;
    transition: border-color var(--transition);
  }
  .text-area:focus { border-color: var(--accent); }
  .hint { font-size: var(--font-size-xs); color: var(--text-muted); margin: 4px 0 0; }
  .error-hint { font-size: var(--font-size-xs); color: var(--danger); margin: 4px 0 0; }

  .model-row { display: flex; justify-content: space-between; align-items: flex-end; }
  .fetch-btn {
    background: none; border: 1px solid var(--border); border-radius: var(--radius-xs);
    color: var(--accent); font-size: var(--font-size-xs); cursor: pointer;
    padding: 3px 8px; margin-top: 14px; font-family: var(--font-ui);
    transition: border-color var(--transition);
  }
  .fetch-btn:hover { border-color: var(--accent); }
  .fetch-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .model-list {
    margin-top: 6px; background: var(--bg-elevated); border: 1px solid var(--border);
    border-radius: var(--radius-sm); max-height: 140px; overflow-y: auto;
  }
  .model-item {
    display: block; width: 100%; text-align: left; background: none; border: none;
    border-bottom: 1px solid var(--bg-base); color: var(--text-secondary);
    font-size: var(--font-size-sm); padding: 6px 10px; cursor: pointer;
    font-family: var(--font-ui); transition: background var(--transition);
  }
  .model-item:last-child { border-bottom: none; }
  .model-item:hover { background: var(--bg-hover); color: var(--text-primary); }
  .model-item.selected { background: var(--accent-dim); color: var(--accent); }

  .reset-btn {
    background: none; border: none; color: var(--accent);
    font-size: var(--font-size-xs); cursor: pointer; padding: 0;
    margin-top: 16px; font-family: var(--font-ui);
  }

  .btn-save {
    padding: 8px 20px; background: var(--accent-dim); border: 1px solid var(--accent);
    border-radius: var(--radius-sm); color: var(--accent); font-size: var(--font-size-md);
    cursor: pointer; font-family: var(--font-ui); transition: filter var(--transition);
  }
  .btn-save:hover { filter: brightness(1.15); }
  .btn-cancel {
    padding: 8px 14px; background: var(--bg-hover); border: none;
    border-radius: var(--radius-sm); color: var(--text-secondary);
    font-size: var(--font-size-md); cursor: pointer; font-family: var(--font-ui);
  }
  .save-error { font-size: var(--font-size-xs); color: var(--danger); flex: 1; }

  .history-mode-row { display: flex; gap: 20px; margin-bottom: 8px; }
  .radio-label { display: flex; align-items: center; gap: 6px; font-size: var(--font-size-sm); color: var(--text-secondary); cursor: pointer; }
  .radio-label input[type="radio"] { accent-color: var(--accent); cursor: pointer; }
</style>

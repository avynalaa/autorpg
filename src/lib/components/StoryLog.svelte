<script lang="ts">
  import { storyLog, isLoading, regenerateLast } from '../stores/gameStore';
  import { marked } from 'marked';

  marked.setOptions({ breaks: true, gfm: true });

  let logEl: HTMLDivElement;

  $effect(() => {
    $storyLog;
    $isLoading;
    if (logEl) {
      requestAnimationFrame(() => {
        if (logEl) logEl.scrollTop = logEl.scrollHeight;
      });
    }
  });

  function renderMarkdown(text: string): string {
    const clean = text.replace(/^#{1,6}\s+(.+)$/gm, '$1');
    return marked.parse(clean, { async: false }) as string;
  }

  function playerDisplay(text: string): string {
    const m = text.match(/^\[Choice:\s*(.*)\]$/s);
    return m ? m[1] : text;
  }

  const lastNarratorIdx = $derived(
    (() => {
      for (let i = $storyLog.length - 1; i >= 0; i--) {
        if ($storyLog[i].source === 'Narrator') return i;
      }
      return -1;
    })()
  );

  async function handleRegenerate() {
    if ($isLoading) return;
    await regenerateLast();
  }

  const regenSVG = `<svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M13.5 8a5.5 5.5 0 1 1-1.1-3.3"/><polyline points="14,2 13.5,5.5 10,5"/></svg>`;
</script>

<div class="story-wrap">
  <div class="log" bind:this={logEl}>
    {#each $storyLog as entry, i (entry.id)}
      {#if entry.source === 'Narrator'}
        <div class="entry-narrator markdown-content" class:is-last={i === lastNarratorIdx}>
          {@html renderMarkdown(entry.text)}
          {#if i === lastNarratorIdx && !$isLoading}
            <button class="regen-btn" onclick={handleRegenerate} title="Regenerate">
              {@html regenSVG}
            </button>
          {/if}
        </div>
      {:else if entry.source === 'Player'}
        <div class="entry-player">
          <span class="player-glyph">◆</span>
          <span class="player-text">{playerDisplay(entry.text)}</span>
        </div>
      {:else if entry.source === 'System'}
        <div class="entry-system">
          <span class="system-rule"></span>
          <span class="system-text">{entry.text}</span>
          <span class="system-rule"></span>
        </div>
      {:else}
        <div class="entry-internal">{entry.text}</div>
      {/if}
    {/each}
    {#if $isLoading}
      <div class="entry-narrator loading">
        <span class="dot">.</span><span class="dot">.</span><span class="dot">.</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .story-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .log {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 4px 2px 4px 0;
  }
  .log::-webkit-scrollbar { width: 4px; }
  .log::-webkit-scrollbar-thumb { background: var(--border); border-radius: 2px; }

  /* ── Narrator ── */
  .entry-narrator {
    position: relative;
    padding: 14px 18px;
    font-family: var(--font-narrative);
    font-size: var(--font-size-narrative);
    line-height: var(--line-height-narrative);
    color: var(--text-primary);
    background: color-mix(in srgb, var(--bg-surface) 70%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  }
  .entry-narrator:first-child { border-top: none; }

  /* ── Regenerate button ── */
  .regen-btn {
    position: absolute;
    bottom: 8px;
    right: 10px;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid transparent;
    border-radius: var(--radius-xs);
    color: var(--text-muted);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--transition), color var(--transition), border-color var(--transition), background var(--transition);
  }
  .entry-narrator.is-last:hover .regen-btn,
  .regen-btn:focus {
    opacity: 1;
  }
  .regen-btn:hover {
    color: var(--accent);
    border-color: var(--border);
    background: var(--bg-elevated);
    opacity: 1;
  }
  .regen-btn :global(svg) { display: block; }

  /* ── Player action ── */
  .entry-player {
    display: flex;
    align-items: baseline;
    gap: 8px;
    padding: 8px 18px;
    background: color-mix(in srgb, var(--success-dim) 50%, transparent);
    border-top: 1px solid color-mix(in srgb, var(--border) 40%, transparent);
  }
  .player-glyph {
    font-size: 8px;
    color: var(--success);
    flex-shrink: 0;
    opacity: 0.7;
    position: relative;
    top: -1px;
  }
  .player-text {
    font-family: var(--font-narrative);
    font-size: var(--font-size-md);
    font-style: italic;
    color: var(--success);
    line-height: 1.55;
  }

  /* ── System event ── */
  .entry-system {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 18px;
  }
  .system-rule {
    flex: 1;
    height: 1px;
    background: var(--border);
    opacity: 0.5;
  }
  .system-text {
    font-family: var(--font-ui);
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  /* ── Internal (debug/misc) ── */
  .entry-internal {
    padding: 4px 18px;
    font-size: var(--font-size-xs);
    color: var(--text-muted);
    opacity: 0.6;
  }

  /* ── Loading ── */
  .loading { opacity: 0.45; }
  .dot { animation: blink 1.1s infinite; display: inline-block; }
  .dot:nth-child(2) { animation-delay: 0.22s; }
  .dot:nth-child(3) { animation-delay: 0.44s; }
  @keyframes blink { 0%,80%,100%{opacity:0} 40%{opacity:1} }

  /* ── Markdown inside narrator ── */
  .markdown-content :global(p)           { margin: 0 0 0.7em; }
  .markdown-content :global(p:last-child){ margin-bottom: 0; }
  .markdown-content :global(h1) { font-size: 1.25em; font-weight: 700; color: var(--accent); margin: 0 0 0.5em; }
  .markdown-content :global(h2) { font-size: 1.1em;  font-weight: 600; color: var(--accent); margin: 0 0 0.4em; }
  .markdown-content :global(h3) { font-size: 1em;    font-weight: 600; color: var(--accent); margin: 0 0 0.3em; }
  .markdown-content :global(strong)     { color: var(--text-primary); font-weight: 700; }
  .markdown-content :global(em)         { color: var(--special); font-style: italic; }
  .markdown-content :global(hr)         { border: none; border-top: 1px solid var(--border); margin: 0.8em 0; opacity: 0.5; }
  .markdown-content :global(ul),
  .markdown-content :global(ol)         { margin: 0.3em 0 0.7em; padding-left: 1.4em; }
  .markdown-content :global(li)         { margin-bottom: 0.2em; }
  .markdown-content :global(blockquote) {
    border-left: 2px solid var(--special);
    margin: 0.6em 0;
    padding: 4px 12px;
    color: color-mix(in srgb, var(--text-secondary) 90%, var(--special));
    font-style: italic;
  }
  .markdown-content :global(code) {
    font-family: var(--font-mono);
    font-size: 0.88em;
    background: var(--bg-hover);
    padding: 1px 5px;
    border-radius: var(--radius-xs);
    color: var(--special);
  }
</style>

<script>
  import '../app.css';
  import { theme } from '$lib/stores/themeStore';
  import NotificationToast from '$lib/components/NotificationToast.svelte';

  let { children } = $props();

  $effect(() => {
    document.documentElement.setAttribute('data-theme', $theme);
  });
</script>

{@render children()}
<NotificationToast />

{#if $theme === 'wire'}
  <div class="scanlines" aria-hidden="true"></div>
{/if}

<style>
  .scanlines {
    position: fixed;
    inset: 0;
    background: repeating-linear-gradient(
      0deg,
      transparent,
      transparent 2px,
      rgba(0, 0, 0, 0.14) 2px,
      rgba(0, 0, 0, 0.14) 4px
    );
    pointer-events: none;
    z-index: 9998;
  }
</style>

<script lang="ts">
  import { onMount } from 'svelte';
  import { SlidersHorizontal, Palette, Music2, Bot, Cpu, Gamepad2, LayoutGrid, Info } from '@lucide/svelte';
  import GeneralSection from './lib/settings/GeneralSection.svelte';
  import LookSection from './lib/settings/LookSection.svelte';
  import MusicSection from './lib/settings/MusicSection.svelte';
  import AgentsSection from './lib/settings/AgentsSection.svelte';
  import ToolsSection from './lib/settings/ToolsSection.svelte';
  import GamesSection from './lib/settings/GamesSection.svelte';
  import WidgetsSection from './lib/settings/WidgetsSection.svelte';
  import AboutSection from './lib/settings/AboutSection.svelte';
  import { settings } from './lib/stores/settings.svelte';
  import { platform } from './lib/stores/platform.svelte';
  import { on } from './lib/bridge';
  import { media } from './lib/stores/media.svelte';
  import { system } from './lib/stores/system.svelte';
  import { findTheme, themeVars, applyVars, mix } from './lib/themes';

  const sections = [
    { id: 'general', label: 'General', icon: SlidersHorizontal },
    { id: 'look', label: 'Look', icon: Palette },
    { id: 'music', label: 'Music', icon: Music2 },
    { id: 'agents', label: 'Agents', icon: Bot },
    { id: 'tools', label: 'Harnesses', icon: Cpu },
    { id: 'games', label: 'Games', icon: Gamepad2 },
    { id: 'widgets', label: 'Shelf and widgets', icon: LayoutGrid },
    { id: 'about', label: 'About', icon: Info },
  ] as const;
  type Section = (typeof sections)[number]['id'];

  let current = $state<Section>((new URLSearchParams(location.search).get('section') as Section) || 'general');

  $effect(() => {
    const theme = findTheme(settings.v.theme, settings.v.customThemes);
    const vars = themeVars(theme, { fontUi: settings.v.fontUi, fontDisplay: settings.v.fontDisplay });
    // The settings window is a regular window: a calmer surface than the island itself.
    vars['--window'] = theme.mode === 'dark' ? mix(theme.island, theme.ink, 0.06) : mix(theme.island, '#ffffff', 0.4);
    vars['--sidebar'] = theme.mode === 'dark' ? mix(theme.island, theme.ink, 0.025) : mix(theme.island, theme.ink, 0.035);
    applyVars(document.documentElement, vars);
  });

  onMount(() => {
    void (async () => {
      await Promise.all([settings.load(), platform.load()]);
      // The update note on the island opens Settings on About; an open window switches here.
      await on<string>('settings://section', (s) => {
        if (sections.some((x) => x.id === s)) current = s as Section;
      });
      await media.start();
      await system.start(() => {}, () => {});
    })();
  });
</script>

<div class="app">
  <aside>
    <div class="brand display">
      <svg viewBox="0 0 208 208" width="20" height="20" aria-hidden="true"><path fill="currentColor" fill-rule="evenodd" d="M56 0 H152 C196 0 208 12 208 56 V152 C208 196 196 208 152 208 H56 C12 208 0 196 0 152 V56 C0 12 12 0 56 0 Z M52 0 A16 16 0 0 1 68 16 V44 A36 36 0 0 0 140 44 V16 A16 16 0 0 1 156 0 Z M52 208 A16 16 0 0 0 68 192 V164 A36 36 0 0 1 140 164 V192 A16 16 0 0 0 156 208 Z" /></svg>
      Holm
    </div>
    <nav>
      {#each sections.filter((x) => x.id !== 'games' || platform.info.games) as sec (sec.id)}
        {@const Icon = sec.icon}
        <button class:on={current === sec.id} aria-current={current === sec.id ? 'page' : undefined} onclick={() => (current = sec.id)}>
          <Icon size={16} />
          {sec.label}
        </button>
      {/each}
    </nav>
  </aside>
  <main>
    {#if settings.ready}
      <div class="content">
        {#if current === 'general'}<GeneralSection />
        {:else if current === 'look'}<LookSection />
        {:else if current === 'music'}<MusicSection />
        {:else if current === 'agents'}<AgentsSection />
        {:else if current === 'tools'}<ToolsSection />
        {:else if current === 'games'}<GamesSection />
        {:else if current === 'widgets'}<WidgetsSection />
        {:else}<AboutSection />{/if}
      </div>
    {/if}
  </main>
</div>

<style>
  :global(body) {
    background: var(--window) !important;
    user-select: none;
  }
  .app {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: 100vh;
    background: var(--window);
    color: var(--ink);
  }
  aside {
    padding: 22px 12px;
    background: var(--sidebar);
    border-right: 1px solid var(--line);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 0 10px 18px;
    font-size: var(--t-lg);
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .brand svg {
    color: var(--accent);
    flex: none;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  nav button {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 36px;
    padding: 0 10px;
    border-radius: 9px;
    color: var(--ink-2);
    font-weight: 550;
    text-align: left;
  }
  nav button:hover {
    background: var(--fill-1);
    color: var(--ink);
  }
  nav button.on {
    background: var(--fill-2);
    color: var(--ink);
  }
  main {
    overflow-y: auto;
  }
  .content {
    max-width: 720px;
    padding: 28px 40px 60px;
  }
  .content :global(h2) {
    margin: 0 0 6px;
    font-size: var(--t-2xl);
    font-weight: 700;
    letter-spacing: -0.03em;
  }
  .content :global(h3) {
    margin: 30px 0 4px;
    font-size: var(--t-lg);
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .content :global(.lead) {
    margin: 0 0 12px;
    font-size: var(--t-md);
    line-height: 1.55;
    color: var(--ink-2);
    max-width: 64ch;
  }
  .content :global(button.primary),
  .content :global(button.ghost) {
    height: 32px;
    padding: 0 14px;
    border-radius: 999px;
    font-size: var(--t-sm);
    font-weight: 600;
    white-space: nowrap;
  }
  .content :global(button.primary) {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .content :global(button.ghost) {
    background: var(--fill-2);
  }
  .content :global(button:disabled) {
    opacity: 0.5;
  }
  .content :global(select) {
    height: 32px;
    min-width: 200px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .content :global(select option) {
    background: var(--window);
    color: var(--ink);
  }
  .content :global(input[type='range']) {
    width: 160px;
    accent-color: var(--accent);
  }
  .content :global(input) {
    user-select: text;
  }
</style>

<script lang="ts">
  import { House, Music2, Bot, Activity, Inbox, Timer, Pin, PinOff, Settings2 } from '@lucide/svelte';
  import Home from '../pages/Home.svelte';
  import Music from '../pages/Music.svelte';
  import Agents from '../pages/Agents.svelte';
  import System from '../pages/System.svelte';
  import Shelf from '../pages/Shelf.svelte';
  import Focus from '../pages/Focus.svelte';
  import { settings, type PageId } from '../stores/settings.svelte';
  import { agents } from '../stores/agents.svelte';
  import { media } from '../stores/media.svelte';
  import { send } from '../bridge';

  let { page = $bindable(), pinned = $bindable() }: { page: PageId; pinned: boolean } = $props();

  const META: Record<PageId, { label: string; icon: typeof House }> = {
    home: { label: 'Home', icon: House },
    music: { label: 'Music', icon: Music2 },
    agents: { label: 'Agents', icon: Bot },
    system: { label: 'System', icon: Activity },
    shelf: { label: 'Shelf', icon: Inbox },
    focus: { label: 'Focus', icon: Timer },
  };
  const pages = $derived(settings.v.pages.filter((p) => p in META));
  const go = (p: PageId) => (page = p);
</script>

<div class="expanded">
  <header>
    <nav aria-label="Pages">
      {#each pages as p (p)}
        {@const Icon = META[p].icon}
        <button class:on={page === p} aria-label={META[p].label} title={META[p].label} aria-current={page === p ? 'page' : undefined} onclick={() => (page = p)}>
          <Icon size={16} strokeWidth={2} />
          {#if page === p}<span class="label">{META[p].label}</span>{/if}
          {#if p === 'agents' && (agents.approvals.length || agents.waiting.length)}<i class="dot warn"></i>
          {:else if p === 'agents' && agents.working.length}<i class="dot"></i>
          {:else if p === 'music' && media.now?.playing && page !== 'music'}<i class="dot"></i>{/if}
        </button>
      {/each}
    </nav>
    <div class="tools">
      <button aria-label={pinned ? 'Unpin' : 'Keep open'} title={pinned ? 'Unpin' : 'Keep open'} class:on={pinned} onclick={() => (pinned = !pinned)}>
        {#if pinned}<PinOff size={15} />{:else}<Pin size={15} />{/if}
      </button>
      <button aria-label="Settings" title="Settings" onclick={() => send('open_settings')}><Settings2 size={15} /></button>
    </div>
  </header>

  <main>
    {#if page === 'home'}<Home {go} />
    {:else if page === 'music'}<Music />
    {:else if page === 'agents'}<Agents />
    {:else if page === 'system'}<System />
    {:else if page === 'shelf'}<Shelf />
    {:else if page === 'focus'}<Focus />{/if}
  </main>
</div>

<style>
  .expanded {
    width: 600px;
    padding: 12px 16px 18px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }
  nav,
  .tools {
    display: flex;
    gap: 2px;
  }
  nav button,
  .tools button {
    position: relative;
    height: 30px;
    min-width: 30px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: 999px;
    color: var(--ink-3);
    transition: background 0.18s, color 0.18s;
  }
  nav button:hover,
  .tools button:hover {
    color: var(--ink);
    background: var(--fill-1);
  }
  nav button.on {
    background: var(--fill-2);
    color: var(--ink);
    padding: 0 12px 0 10px;
  }
  .tools button.on {
    color: var(--accent);
  }
  .label {
    font-size: var(--t-sm);
    font-weight: 600;
  }
  .dot {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
  }
  .dot.warn {
    background: var(--warn);
  }
  nav button.on .dot {
    display: none;
  }
</style>

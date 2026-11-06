<script lang="ts">
  import { Plus, X } from '@lucide/svelte';
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import { settings } from '../stores/settings.svelte';

  const s = $derived(settings.v);
  let exe = $state('');
  let name = $state('');

  function cleanExe(text: string): string {
    // Accept a full path or a bare name, with or without .exe.
    const file = text.trim().split(/[\\/]/).pop() ?? '';
    if (!file) return '';
    return file.toLowerCase().endsWith('.exe') ? file : `${file}.exe`;
  }

  const ready = $derived(cleanExe(exe) !== '' && !s.customGames.some((g) => g.exe.toLowerCase() === cleanExe(exe).toLowerCase()));

  function add() {
    if (!ready) return;
    void settings.set({ customGames: [...s.customGames, { exe: cleanExe(exe), name: name.trim() }] });
    exe = '';
    name = '';
  }

  function remove(i: number) {
    void settings.set({ customGames: s.customGames.filter((_, j) => j !== i) });
  }
</script>

<section>
  <h2 class="display">Games</h2>
  <p class="lead">
    Games from Steam, Epic, Riot, Rockstar, EA, Ubisoft, Xbox, GOG and Battle.net are recognised on their own, and so are well-known ones like GTA V, Valorant and Counter-Strike 2.
  </p>

  <Field label="Play an intro" hint="When a game comes to the front, the island shows its name and the keys that bring it back or hide it.">
    <Toggle label="Play an intro" checked={s.gameIntro} onchange={(v) => settings.set({ gameIntro: v })} />
  </Field>

  <Field label="Step aside while playing" hint="The island hides while a game is in front, even in a window, and comes back when you switch away. The open shortcut still brings it up.">
    <Toggle label="Step aside while playing" checked={s.smartHide} onchange={(v) => settings.set({ smartHide: v })} />
  </Field>

  <Field label="Play time when you quit" hint="A short note with how long you played. Sessions under a minute are skipped.">
    <Toggle label="Play time when you quit" checked={s.gameRecap} onchange={(v) => settings.set({ gameRecap: v })} />
  </Field>

  <h3>Your games</h3>
  <p class="lead">Add a game that is not recognised. Its .exe name is in Task Manager: right-click the game, then Properties.</p>

  {#if s.customGames.length}
    <ul class="games">
      {#each s.customGames as g, i (g.exe)}
        <li>
          <span class="name">{g.name || g.exe.replace(/\.exe$/i, '')}</span>
          <span class="exe">{g.exe}</span>
          <button aria-label="Remove {g.name || g.exe}" onclick={() => remove(i)}><X size={14} /></button>
        </li>
      {/each}
    </ul>
  {/if}

  <form class="add" onsubmit={(e) => (e.preventDefault(), add())}>
    <input bind:value={exe} placeholder="game.exe" aria-label="Game .exe name" spellcheck="false" />
    <input bind:value={name} placeholder="Name to show (optional)" aria-label="Name to show" />
    <button type="submit" disabled={!ready}><Plus size={14} />Add</button>
  </form>
</section>

<style>
  .games {
    list-style: none;
    margin: 0 0 10px;
    padding: 0;
  }
  .games li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    border-bottom: 1px solid var(--line);
  }
  .name {
    font-weight: 600;
  }
  .exe {
    flex: 1;
    font-size: var(--t-sm);
    color: var(--ink-3);
  }
  .games button {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--ink-2);
  }
  .games button:hover {
    background: var(--fill-1);
  }
  .add {
    display: flex;
    gap: 8px;
  }
  .add input {
    flex: 1;
    min-width: 0;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--line);
    color: var(--ink);
    font-size: var(--t-sm);
  }
  .add input:focus {
    outline: none;
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }
  .add button {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 34px;
    padding: 0 14px;
    border-radius: 8px;
    background: var(--accent);
    color: var(--accent-ink);
    font-weight: 600;
    font-size: var(--t-sm);
  }
  .add button:disabled {
    opacity: 0.4;
  }
</style>

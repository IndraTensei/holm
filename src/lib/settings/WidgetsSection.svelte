<script lang="ts">
  import Field from './Field.svelte';
  import Toggle from './Toggle.svelte';
  import Choice from './Choice.svelte';
  import { settings } from '../stores/settings.svelte';
  import { searchCity } from '../stores/widgets.svelte';
  import { send } from '../bridge';

  const s = $derived(settings.v);
  let query = $state('');
  let results = $state<{ name: string; country: string; admin: string; lat: number; lon: number }[]>([]);
  let searching = $state(false);

  async function search() {
    if (!query.trim()) return;
    searching = true;
    results = await searchCity(query.trim()).catch(() => []);
    searching = false;
  }
</script>

<section>
  <h2 class="display">Shelf and widgets</h2>

  <h3>Clipboard</h3>
  <Field label="Clipboard history" hint="Recent text you copy, in the Shelf page. Kept in memory only; copies marked private by password managers are skipped.">
    <Toggle label="Clipboard history" checked={s.clipboardHistory} onchange={(v) => settings.set({ clipboardHistory: v })} />
  </Field>
  <Field label="Keep">
    <Choice label="Keep" value={s.clipboardMax} options={[{ value: 15, label: '15' }, { value: 30, label: '30' }, { value: 60, label: '60' }]} onchange={(v) => settings.set({ clipboardMax: v })} />
    <button class="ghost" onclick={() => send('clipboard_clear')}>Clear now</button>
  </Field>

  <h3>Weather</h3>
  <Field label="Weather on Home" hint="From Open-Meteo. Only the place you choose is sent.">
    <Toggle label="Weather on Home" checked={s.weather} onchange={(v) => settings.set({ weather: v })} />
  </Field>
  {#if s.weather}
    <Field label="Place" hint={s.weatherCity ? `Now: ${s.weatherCity}` : 'Search for your city.'}>
      <form class="search" onsubmit={(e) => { e.preventDefault(); void search(); }}>
        <input placeholder="City" bind:value={query} />
        <button class="ghost" disabled={searching}>{searching ? 'Searching…' : 'Search'}</button>
      </form>
    </Field>
    {#if results.length}
      <ul class="results">
        {#each results as r}
          <li>
            <button
              onclick={() => {
                void settings.set({ weatherCity: r.name, weatherLat: r.lat, weatherLon: r.lon });
                results = [];
                query = '';
              }}
            >{r.name}<span>{[r.admin, r.country].filter(Boolean).join(', ')}</span></button>
          </li>
        {/each}
      </ul>
    {/if}
    <Field label="Units">
      <Choice label="Units" value={s.units} options={[{ value: 'metric', label: '°C' }, { value: 'imperial', label: '°F' }]} onchange={(v) => settings.set({ units: v })} />
    </Field>
  {/if}

  <h3>Focus timer</h3>
  <Field label="Focus length">
    <input type="range" min="5" max="90" step="5" value={s.focusMinutes} oninput={(e) => settings.set({ focusMinutes: Number((e.target as HTMLInputElement).value) })} />
    <span class="num val">{s.focusMinutes} min</span>
  </Field>
  <Field label="Break length">
    <input type="range" min="1" max="30" step="1" value={s.breakMinutes} oninput={(e) => settings.set({ breakMinutes: Number((e.target as HTMLInputElement).value) })} />
    <span class="num val">{s.breakMinutes} min</span>
  </Field>
</section>

<style>
  .search {
    display: flex;
    gap: 8px;
  }
  .search input {
    width: 180px;
    height: 32px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--line);
  }
  .results {
    list-style: none;
    margin: 0 0 8px;
    padding: 4px;
    border-radius: 12px;
    background: var(--fill-1);
  }
  .results button {
    width: 100%;
    display: flex;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: 8px;
    text-align: left;
    font-weight: 550;
  }
  .results button:hover {
    background: var(--fill-2);
  }
  .results span {
    font-weight: 400;
    color: var(--ink-3);
  }
  .val {
    width: 54px;
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
</style>

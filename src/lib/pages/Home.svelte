<script lang="ts">
  import { Wifi, WifiOff, Bluetooth, BluetoothOff, Play, Pause, SkipForward, Sun, CloudSun, Cloud, CloudFog, CloudRain, CloudSnow, CloudLightning, Moon, BatteryCharging, Battery } from '@lucide/svelte';
  import Art from '../ui/Art.svelte';
  import Btn from '../ui/Btn.svelte';
  import Ring from '../ui/Ring.svelte';
  import AgentGlyph from '../ui/AgentGlyph.svelte';
  import { media } from '../stores/media.svelte';
  import { agents } from '../stores/agents.svelte';
  import { system } from '../stores/system.svelte';
  import { weather, describeWeather } from '../stores/widgets.svelte';
  import { settings } from '../stores/settings.svelte';
  import { clock } from '../stores/clock.svelte';
  import { send } from '../bridge';
  import { until } from '../format';
  import type { PageId } from '../stores/settings.svelte';

  let { go }: { go: (page: PageId) => void } = $props();
  let radioError = $state('');

  $effect(() => {
    void system.refreshRadios();
  });

  const d = $derived(new Date(clock.now));
  const time = $derived(d.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit', hour12: !settings.v.clock24h }));
  const [hm, meridiem] = $derived(time.split(' '));
  const date = $derived(d.toLocaleDateString([], { weekday: 'long', day: 'numeric', month: 'long' }));
  const w = $derived(weather.now);
  const wd = $derived(w ? describeWeather(w.code) : null);
  const now = $derived(media.now);
  const agent = $derived(agents.focus ?? agents.sessions[0] ?? null);
  // At most three rings, so the row never wraps: Claude's 5-hour and weekly, then Codex's first.
  const quotaItems = $derived(
    system.quotas
      .filter((p) => p.ok)
      .flatMap((p) => p.items.slice(0, p.provider === 'claude' ? 2 : 1).map((q) => ({ provider: p.provider, q })))
      .slice(0, 3),
  );

  async function toggle(kind: 'wifi' | 'bluetooth', on: boolean) {
    radioError = (await system.setRadio(kind, on)) ?? '';
    if (radioError) send('open_url', { url: kind === 'wifi' ? 'ms-settings:network-wifi' : 'ms-settings:bluetooth' });
  }
</script>

<div class="home">
  <section class="left">
    <div class="clock display num">
      {hm}{#if meridiem}<span class="mer">{meridiem}</span>{/if}
    </div>
    <div class="date">{date}</div>

    {#if w && wd}
      <div class="weather">
        <span class="w-icon">
          {#if wd.icon === 'sun'}{#if w.isDay}<Sun size={18} />{:else}<Moon size={18} />{/if}
          {:else if wd.icon === 'cloud-sun'}<CloudSun size={18} />
          {:else if wd.icon === 'cloud'}<Cloud size={18} />
          {:else if wd.icon === 'fog'}<CloudFog size={18} />
          {:else if wd.icon === 'rain'}<CloudRain size={18} />
          {:else if wd.icon === 'snow'}<CloudSnow size={18} />
          {:else}<CloudLightning size={18} />{/if}
        </span>
        <span class="w-temp num">{Math.round(w.temp)}°</span>
        <span class="w-text">{wd.label}{w.city ? ` in ${w.city}` : ''}<br /><span class="num">{Math.round(w.high)}° / {Math.round(w.low)}°</span></span>
      </div>
    {/if}

    <div class="chips">
      {#if system.radios?.wifi !== null && system.radios?.wifi !== undefined}
        <button class="chip" class:on={system.radios.wifi} onclick={() => toggle('wifi', !system.radios?.wifi)} aria-pressed={system.radios.wifi}>
          {#if system.radios.wifi}<Wifi size={15} />{:else}<WifiOff size={15} />{/if}
          <span>{system.radios.wifi ? system.radios.ssid || 'Wi-Fi' : 'Wi-Fi off'}</span>
        </button>
      {/if}
      {#if system.radios?.bluetooth !== null && system.radios?.bluetooth !== undefined}
        <button class="chip" class:on={system.radios.bluetooth} onclick={() => toggle('bluetooth', !system.radios?.bluetooth)} aria-pressed={system.radios.bluetooth}>
          {#if system.radios.bluetooth}<Bluetooth size={15} />{:else}<BluetoothOff size={15} />{/if}
          <span>{system.radios.bluetooth ? 'Bluetooth' : 'Bluetooth off'}</span>
        </button>
      {/if}
      {#if system.battery}
        <span class="chip static" class:low={!system.battery.plugged && system.battery.percent <= 20}>
          {#if system.battery.charging}<BatteryCharging size={15} />{:else}<Battery size={15} />{/if}
          <span class="num">{system.battery.percent}%</span>
        </span>
      {/if}
    </div>
    {#if radioError}<div class="err">{radioError}. Opened Windows settings instead.</div>{/if}
  </section>

  <section class="right">
    {#if now}
      <div class="tile" role="button" tabindex="0" onclick={() => go('music')} onkeydown={(e) => e.key === 'Enter' && go('music')}>
        <Art src={now.art} size={46} radius={11} />
        <div class="t-text">
          <div class="t-title">{now.title}</div>
          <div class="t-sub">{now.artist || now.app}</div>
        </div>
        <Btn label={now.playing ? 'Pause' : 'Play'} onclick={media.toggle} size={32}>
          {#if now.playing}<Pause size={16} fill="currentColor" />{:else}<Play size={16} fill="currentColor" />{/if}
        </Btn>
        <Btn label="Next" onclick={media.next} disabled={!now.canNext} size={32}><SkipForward size={16} fill="currentColor" /></Btn>
      </div>
    {/if}

    {#if agent}
      <div class="tile" role="button" tabindex="0" onclick={() => go('agents')} onkeydown={(e) => e.key === 'Enter' && go('agents')}>
        <span class="a-glyph"><AgentGlyph status={agent.status} size={18} /></span>
        <div class="t-text">
          <div class="t-title">{agent.project}</div>
          <div class="t-sub">
            {#if agent.status === 'working'}{agent.activity?.target ? `${agent.activity.verb} ${agent.activity.target}` : 'Thinking'}
            {:else if agent.status === 'waiting'}Waiting for you
            {:else if agent.status === 'done'}Finished
            {:else}Idle{/if}
          </div>
        </div>
        {#if agents.sessions.length > 1}<span class="count num">{agents.sessions.length} sessions</span>{/if}
      </div>
    {/if}

    {#if system.quotas.some((q) => q.ok)}
      <div class="quota" style="--n:{quotaItems.length}">
        {#each quotaItems as { provider, q } (provider + q.label)}
          {@const name = provider === 'claude' ? 'Claude' : 'Codex'}
          <div class="q" title="{name} {q.label.toLowerCase()}: {Math.round(q.percent)}% used{q.resetsAt ? `, resets in ${until(q.resetsAt, clock.now)}` : ''}">
            <Ring value={q.percent} size={30} stroke={3}>
              <span class="q-num num">{Math.round(q.percent)}</span>
            </Ring>
            <div class="q-text">
              <span>{provider === 'claude' ? q.label : name}</span>
              <span class="num">{q.resetsAt ? until(q.resetsAt, clock.now) : `${Math.round(q.percent)}% used`}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    {#if !now && !agent && !system.quotas.some((q) => q.ok)}
      <p class="quiet">Music, agents and plan usage show up here when there is something to show.</p>
    {/if}
  </section>
</div>

<style>
  .home {
    display: grid;
    /* minmax(0, …): a long title or command ellipsizes instead of widening the column. */
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr);
    gap: 22px;
    padding: 4px 4px 2px;
  }
  .left {
    display: flex;
    flex-direction: column;
  }
  .clock {
    font-size: var(--t-3xl);
    font-weight: 700;
    letter-spacing: -0.04em;
    line-height: 1;
  }
  .mer {
    margin-left: 6px;
    font-size: var(--t-lg);
    font-weight: 600;
    letter-spacing: 0;
    color: var(--ink-3);
  }
  .date {
    margin-top: 6px;
    font-size: var(--t-md);
    color: var(--ink-2);
  }
  .weather {
    margin-top: 16px;
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .w-icon {
    color: var(--accent);
    display: grid;
  }
  .w-temp {
    font-family: var(--font-display);
    font-size: var(--t-xl);
    font-weight: 650;
    letter-spacing: -0.02em;
  }
  .w-text {
    font-size: var(--t-xs);
    line-height: 1.35;
    color: var(--ink-2);
  }
  .chips {
    margin-top: 18px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 11px;
    border-radius: 999px;
    background: var(--fill-1);
    color: var(--ink-2);
    font-size: var(--t-xs);
    font-weight: 600;
    max-width: 170px;
    transition: background 0.15s, color 0.15s;
  }
  .chip span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chip.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .chip.static {
    cursor: default;
  }
  .chip.low {
    color: var(--err);
  }
  .err {
    margin-top: 6px;
    font-size: var(--t-xs);
    color: var(--warn);
  }
  .right {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tile {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 8px 8px 8px 8px;
    border-radius: 16px;
    background: var(--fill-1);
    cursor: pointer;
    transition: background 0.15s;
  }
  .tile:hover {
    background: var(--fill-2);
  }
  .a-glyph {
    width: 46px;
    height: 46px;
    border-radius: 11px;
    display: grid;
    place-items: center;
    background: var(--fill-1);
  }
  .t-text {
    flex: 1;
    min-width: 0;
  }
  .t-title {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .t-sub {
    margin-top: 1px;
    font-size: var(--t-xs);
    color: var(--ink-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .count {
    font-size: var(--t-xs);
    color: var(--ink-3);
    padding-right: 6px;
  }
  .quota {
    display: grid;
    grid-template-columns: repeat(var(--n), minmax(0, 1fr));
    gap: 10px;
    padding: 8px 4px 0;
  }
  .q {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .q-num {
    font-size: 10.5px;
    font-weight: 700;
  }
  .q-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .q-text span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .q-text span:first-child {
    color: var(--ink);
    font-weight: 600;
    font-size: var(--t-sm);
  }
  .quiet {
    margin: 8px 0 0;
    font-size: var(--t-sm);
    line-height: 1.5;
    color: var(--ink-3);
  }
</style>

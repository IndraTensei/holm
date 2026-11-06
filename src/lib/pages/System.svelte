<script lang="ts">
  import Spark from '../ui/Spark.svelte';
  import { system } from '../stores/system.svelte';
  import { rate } from '../format';

  $effect(() => system.watch());
  const s = $derived(system.stats);
  const netMax = $derived(Math.max(64 * 1024, ...system.down, ...system.up));
  const b = $derived(system.battery);
  const left = $derived.by(() => {
    if (!b?.secondsLeft || b.plugged) return '';
    const m = Math.round(b.secondsLeft / 60);
    return m >= 60 ? `${Math.floor(m / 60)} h ${m % 60} min left` : `${m} min left`;
  });
</script>

<div class="sys">
  <div class="row">
    <div class="label">Processor</div>
    <div class="value display num">{s ? Math.round(s.cpu) : '–'}<small>%</small></div>
    <Spark data={system.cpu} max={100} width={250} height={34} />
  </div>
  <div class="row">
    <div class="label">Memory</div>
    <div class="value display num">{s ? Math.round(s.memory) : '–'}<small>%</small></div>
    <div class="spark-col">
      <Spark data={system.memory} max={100} width={250} height={34} color="var(--ok)" />
      {#if s}<span class="note num">{s.memoryUsedGb.toFixed(1)} of {s.memoryTotalGb.toFixed(0)} GB</span>{/if}
    </div>
  </div>
  <div class="row">
    <div class="label">Network</div>
    <div class="value net num">
      <span><i class="down"></i>{s ? rate(s.netDown) : '–'}</span>
      <span><i class="up"></i>{s ? rate(s.netUp) : '–'}</span>
    </div>
    <div class="dual">
      <Spark data={system.down} max={netMax} width={250} height={34} />
      <div class="overlay"><Spark data={system.up} max={netMax} width={250} height={34} color="var(--warn)" /></div>
    </div>
  </div>
  {#if b}
    <div class="row">
      <div class="label">Battery</div>
      <div class="value display num">{b.percent}<small>%</small></div>
      <div class="battery">
        <div class="cell" class:low={!b.plugged && b.percent <= 20}><span style="width:{b.percent}%"></span></div>
        <span class="note">{b.charging ? 'Charging' : b.plugged ? 'Plugged in' : left || 'On battery'}{b.saver ? ', battery saver on' : ''}</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .sys {
    display: flex;
    flex-direction: column;
    padding: 2px 4px;
  }
  .row {
    display: grid;
    grid-template-columns: 96px 130px 1fr;
    align-items: center;
    gap: 12px;
    padding: 9px 0;
  }
  .row + .row {
    border-top: 1px solid var(--line);
  }
  .label {
    font-size: var(--t-sm);
    color: var(--ink-2);
  }
  .value {
    font-size: var(--t-2xl);
    font-weight: 650;
    letter-spacing: -0.03em;
    line-height: 1;
  }
  .value small {
    font-size: var(--t-md);
    color: var(--ink-3);
    margin-left: 1px;
  }
  .net {
    display: flex;
    flex-direction: column;
    gap: 3px;
    font-size: var(--t-sm);
    font-weight: 600;
  }
  .net i {
    display: inline-block;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    margin-right: 7px;
  }
  .down {
    background: var(--accent);
  }
  .up {
    background: var(--warn);
  }
  .spark-col,
  .battery {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .note {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .dual {
    position: relative;
  }
  .overlay {
    position: absolute;
    inset: 0;
  }
  .cell {
    height: 12px;
    border-radius: 4px;
    background: var(--fill-2);
    overflow: hidden;
    max-width: 250px;
  }
  .cell span {
    display: block;
    height: 100%;
    background: var(--ok);
    border-radius: 4px;
  }
  .cell.low span {
    background: var(--err);
  }
</style>

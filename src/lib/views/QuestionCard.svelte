<script lang="ts">
  // Claude Code asking a multiple-choice question (AskUserQuestion), answered from the island.
  // One question at a time; a single-choice pick moves on by itself, a multi-choice one waits
  // for Next. An answer of your own can be typed below the options (on its own, or added to
  // multi-choice picks). "In terminal" hands it back to Claude Code's own prompt.
  import { MessageCircleQuestionMark, Check } from '@lucide/svelte';
  import Ring from '../ui/Ring.svelte';
  import type { Approval } from '../types';
  import { agents } from '../stores/agents.svelte';
  import { clock } from '../stores/clock.svelte';
  import { inTauri, focusWindow, send } from '../bridge';

  let { approval, more = 0, compact = false }: { approval: Approval; more?: number; compact?: boolean } = $props();

  $effect(() => clock.second());

  const left = $derived(Math.max(0, approval.expiresAt - clock.now));
  const total = $derived(Math.max(1, approval.expiresAt - approval.createdAt));

  let step = $state(0);
  let answers = $state<Record<string, string>>({});
  let picked = $state<string[]>([]);
  /** An answer of the user's own for the question on screen. */
  let typed = $state('');

  const count = $derived(approval.questions.length);
  const q = $derived(approval.questions[Math.min(step, count - 1)]);
  const last = $derived(step >= count - 1);

  const own = $derived(typed.trim());
  const ready = $derived(picked.length > 0 || own.length > 0);

  /** Typing took keyboard focus from the app the user was in; give it back once answered. */
  function releaseFocus() {
    if (inTauri && document.hasFocus()) send('island_release_focus');
  }

  function commit(labels: string[]) {
    const next = { ...answers, [q.question]: labels.join(', ') };
    if (last) {
      releaseFocus();
      return agents.answer(approval.id, next);
    }
    answers = next;
    picked = [];
    typed = '';
    step += 1;
  }

  function choose(label: string) {
    if (!q.multiSelect) return commit([label]);
    picked = picked.includes(label) ? picked.filter((l) => l !== label) : [...picked, label];
  }

  /** Next or Send: the picks, with the typed answer (if any) after them. */
  function sendThis() {
    if (ready) commit(own ? [...picked, own] : picked);
  }

  function back() {
    step -= 1;
    const prev = approval.questions[step];
    const parts = answers[prev.question]?.split(', ') ?? [];
    const labels = prev.options.map((o) => o.label);
    picked = prev.multiSelect ? parts.filter((p) => labels.includes(p)) : [];
    typed = parts.filter((p) => !labels.includes(p)).join(', ');
  }

  function terminal() {
    releaseFocus();
    agents.answer(approval.id, null);
  }
</script>

<div class="card" class:compact>
  <div class="head">
    <span class="icon"><MessageCircleQuestionMark size={16} /></span>
    <div class="who">
      <span class="project display">{approval.project}</span>
      <span class="ask">{count > 1 ? `asks you ${count} questions` : 'asks you'}</span>
    </div>
    <Ring value={(left / total) * 100} size={18} stroke={2.2} color="var(--warn)" />
  </div>

  {#key step}
    <div class="body">
      <div class="meta">
        {#if q.header}<span class="chip">{q.header}</span>{/if}
        {#if count > 1}<span class="step num">{step + 1} of {count}</span>{/if}
        {#if q.multiSelect}<span class="hint">Pick any</span>{/if}
      </div>
      <p class="question">{q.question}</p>
      <div class="options" role={q.multiSelect ? 'group' : 'radiogroup'} aria-label={q.header || q.question}>
        {#each q.options as o, i (o.label)}
          {@const on = picked.includes(o.label)}
          <button
            class="option"
            class:on
            role={q.multiSelect ? 'checkbox' : 'radio'}
            aria-checked={on}
            onclick={(e) => { e.stopPropagation(); choose(o.label); }}
          >
            <span class="mark" class:box={q.multiSelect}>
              {#if on}<Check size={12} strokeWidth={3} />{:else}<span class="num">{i + 1}</span>{/if}
            </span>
            <span class="text">
              <span class="label">{o.label}</span>
              {#if o.description}<span class="desc">{o.description}</span>{/if}
            </span>
          </button>
        {/each}
      </div>
      <input
        class="own"
        type="text"
        bind:value={typed}
        placeholder={q.multiSelect ? 'Add an answer of your own' : 'Or type your own answer'}
        aria-label="Your own answer"
        onfocus={() => void focusWindow()}
        onclick={(e) => e.stopPropagation()}
        onkeydown={(e) => {
          e.stopPropagation();
          if (e.key === 'Enter') sendThis();
        }}
      />
    </div>
  {/key}

  <div class="actions">
    <button class="quiet" title="Answer in Claude Code's own prompt instead" onclick={(e) => { e.stopPropagation(); terminal(); }}>
      In terminal
    </button>
    {#if more > 0}<span class="more">{more} more waiting</span>{/if}
    <span class="grow"></span>
    {#if step > 0}
      <button class="ghost" onclick={(e) => { e.stopPropagation(); back(); }}>Back</button>
    {/if}
    {#if q.multiSelect || own}
      <button class="send" disabled={!ready} onclick={(e) => { e.stopPropagation(); sendThis(); }}>
        {last ? 'Send' : 'Next'}
      </button>
    {/if}
  </div>
</div>

<style>
  .card {
    width: 440px;
    padding: 14px 14px 12px;
  }
  .card.compact {
    width: 100%;
    padding: 12px;
    border-radius: 18px;
    background: var(--fill-1);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .icon {
    flex: none;
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--accent);
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
  }
  .project {
    font-size: var(--t-lg);
    font-weight: 650;
    letter-spacing: -0.015em;
  }
  .ask {
    font-size: var(--t-sm);
    color: var(--ink-2);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .body {
    margin: 12px 0 12px;
    animation: rise 0.22s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .chip {
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--fill-2);
    color: var(--ink-2);
    font-weight: 600;
  }
  .question {
    margin: 0 0 10px;
    font-size: var(--t-md);
    font-weight: 600;
    line-height: 1.35;
    color: var(--ink);
    user-select: text;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 216px;
    overflow: auto;
  }
  .option {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 9px 11px;
    border-radius: 12px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px transparent;
    text-align: left;
    color: var(--ink);
    transition: background 0.15s, box-shadow 0.15s, transform 0.12s;
  }
  .compact .option {
    background: var(--fill-2);
  }
  .option:hover {
    background: var(--fill-2);
  }
  .option:active {
    transform: scale(0.985);
  }
  .option.on {
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 55%, transparent);
  }
  .mark {
    flex: none;
    width: 20px;
    height: 20px;
    margin-top: 1px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--fill-2);
    color: var(--ink-3);
    font-size: 11px;
    font-weight: 650;
  }
  .mark.box {
    border-radius: 6px;
  }
  .on .mark {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .text {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .label {
    font-size: var(--t-sm);
    font-weight: 600;
  }
  .desc {
    font-size: var(--t-xs);
    line-height: 1.4;
    color: var(--ink-2);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .own {
    width: 100%;
    height: 36px;
    margin-top: 8px;
    padding: 0 12px;
    border: none;
    outline: none;
    border-radius: 12px;
    background: var(--fill-1);
    box-shadow: inset 0 0 0 1px var(--fill-2);
    color: var(--ink);
    font: inherit;
    font-size: var(--t-sm);
    transition: box-shadow 0.15s;
  }
  .compact .own {
    background: var(--fill-2);
  }
  .own::placeholder {
    color: var(--ink-3);
  }
  .own:focus {
    box-shadow: inset 0 0 0 1.5px var(--accent);
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .grow {
    flex: 1;
  }
  .more {
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .actions button {
    height: 32px;
    padding: 0 16px;
    border-radius: 999px;
    font-size: var(--t-sm);
    font-weight: 600;
    transition: transform 0.12s, filter 0.15s, background 0.15s, opacity 0.15s;
  }
  .actions button:active:not(:disabled) {
    transform: scale(0.95);
  }
  .quiet {
    padding: 0 12px !important;
    color: var(--ink-2);
  }
  .quiet:hover {
    background: var(--fill-1);
    color: var(--ink);
  }
  .ghost {
    background: var(--fill-2);
    color: var(--ink);
  }
  .ghost:hover {
    background: var(--fill-3);
  }
  .send {
    background: var(--accent);
    color: var(--accent-ink);
  }
  .send:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .send:disabled {
    opacity: 0.4;
  }
</style>

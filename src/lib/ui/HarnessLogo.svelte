<script lang="ts">
  // A harness's own logo (LobeHub Icons, MIT: src/assets/harness), or its initials when there is
  // none. The marks belong to their owners and only name the tool.
  import claude from '../../assets/harness/claude.svg?raw';
  import codex from '../../assets/harness/codex.svg?raw';
  import opencode from '../../assets/harness/opencode.svg?raw';
  import antigravity from '../../assets/harness/antigravity.svg?raw';
  import gemini from '../../assets/harness/gemini.svg?raw';
  import copilot from '../../assets/harness/copilot.svg?raw';
  import cline from '../../assets/harness/cline.svg?raw';
  import cursor from '../../assets/harness/cursor.svg?raw';
  import amp from '../../assets/harness/amp.svg?raw';
  import qwen from '../../assets/harness/qwen.svg?raw';
  import kiro from '../../assets/harness/kiro.svg?raw';
  import goose from '../../assets/harness/goose.svg?raw';

  const LOGOS: Record<string, string> = { claude, codex, opencode, antigravity, gemini, copilot, cline, cursor, amp, qwen, kiro, goose };

  let { id, name, size = 22 }: { id: string; name: string; size?: number } = $props();
  const svg = $derived(LOGOS[id]);
  const initials = $derived(name.replace(/^(GitHub|Factory) /, '').slice(0, 2));
</script>

{#if svg}
  <span class="logo" style="font-size:{size}px" aria-hidden="true">{@html svg}</span>
{:else}
  <span class="initials display" style="font-size:{Math.round(size * 0.55)}px" aria-hidden="true">{initials}</span>
{/if}

<style>
  .logo {
    display: inline-grid;
    place-items: center;
    line-height: 0;
    color: var(--ink);
  }
  .logo :global(svg) {
    display: block;
  }
  .initials {
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--ink-2);
  }
</style>

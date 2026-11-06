<script lang="ts">
  import { send } from '../bridge';
</script>

<section>
  <h2 class="display">About</h2>
  <p class="lead">Holm 0.1.0, a lean Dynamic Island for Windows 10 and 11. Built by Ashish John Thomas at Tensei Tech.</p>

  <h3>What it talks to</h3>
  <ul class="facts">
    <li>Windows media controls, system readings, the clipboard and radios stay on this PC.</li>
    <li>Claude Code reaches the island through local hooks on 127.0.0.1. Nothing is uploaded.</li>
    <li>Plan usage asks Anthropic and OpenAI with the sign-ins already on this PC, and only when turned on.</li>
    <li>Lyrics come from LRCLIB and weather from Open-Meteo, only when turned on.</li>
    <li>Pear is reached on its local API Server with the token it gives the island.</li>
  </ul>

  <h3>Credits</h3>
  <ul class="facts">
    <li>Plan usage reading and some interaction ideas adapted from BrimDeck (Apache-2.0).</li>
    <li>Pear Desktop API by the pear-devs community. Icons by Lucide (ISC).</li>
    <li>Typefaces under the SIL Open Font License: Geist, Inter, Bricolage Grotesque, Manrope, Plus Jakarta Sans, Space Grotesk, Outfit, Sora, DM Sans, Fraunces, Instrument Serif, JetBrains Mono, Geist Mono.</li>
  </ul>

  <div class="links">
    <button class="ghost" onclick={() => send('open_url', { url: 'https://github.com/pear-devs/pear-desktop' })}>Get Pear Desktop</button>
    <button class="ghost" onclick={() => send('open_url', { url: 'https://code.claude.com/docs/en/hooks' })}>Claude Code hooks</button>
  </div>
</section>

<style>
  .facts {
    margin: 0 0 8px;
    padding-left: 18px;
    font-size: var(--t-sm);
    line-height: 1.7;
    color: var(--ink-2);
    max-width: 70ch;
  }
  .links {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
</style>

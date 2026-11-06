<script lang="ts">
  import { FileText, FileImage, FileArchive, FileCode, FileAudio, FileVideo, File, FolderOpen, X, Copy, Check, Trash2 } from '@lucide/svelte';
  import { shelf, note } from '../stores/widgets.svelte';
  import { system } from '../stores/system.svelte';
  import { clock } from '../stores/clock.svelte';
  import { ago, extension, fileName } from '../format';

  let tab = $state<'files' | 'clips' | 'note'>('files');
  let copiedId = $state<number | null>(null);

  const kinds: Record<string, string> = {
    png: 'image', jpg: 'image', jpeg: 'image', gif: 'image', webp: 'image', svg: 'image', psd: 'image', heic: 'image',
    zip: 'archive', rar: 'archive', '7z': 'archive', tar: 'archive', gz: 'archive',
    ts: 'code', js: 'code', rs: 'code', py: 'code', json: 'code', html: 'code', css: 'code', cs: 'code', go: 'code',
    mp3: 'audio', wav: 'audio', flac: 'audio', m4a: 'audio', ogg: 'audio',
    mp4: 'video', mkv: 'video', mov: 'video', webm: 'video',
    pdf: 'doc', doc: 'doc', docx: 'doc', txt: 'doc', md: 'doc', xlsx: 'doc', pptx: 'doc',
  };
  const kind = (p: string) => kinds[extension(p)] ?? 'file';

  function copy(id: number, text: string) {
    system.copy(text);
    copiedId = id;
    setTimeout(() => (copiedId = null), 1200);
  }
</script>

<div class="shelf">
  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'files'} class:on={tab === 'files'} onclick={() => (tab = 'files')}>Files <span class="num">{shelf.items.length || ''}</span></button>
    <button role="tab" aria-selected={tab === 'clips'} class:on={tab === 'clips'} onclick={() => (tab = 'clips')}>Clipboard</button>
    <button role="tab" aria-selected={tab === 'note'} class:on={tab === 'note'} onclick={() => (tab = 'note')}>Note</button>
  </div>

  {#if tab === 'files'}
    {#if shelf.items.length}
      <div class="files">
        {#each shelf.items as item (item.path)}
          <div class="file" role="button" tabindex="0" title={item.path} ondblclick={() => shelf.open(item.path)} onkeydown={(e) => e.key === 'Enter' && shelf.open(item.path)}>
            <span class="f-icon {kind(item.path)}">
              {#if kind(item.path) === 'image'}<FileImage size={22} />
              {:else if kind(item.path) === 'archive'}<FileArchive size={22} />
              {:else if kind(item.path) === 'code'}<FileCode size={22} />
              {:else if kind(item.path) === 'audio'}<FileAudio size={22} />
              {:else if kind(item.path) === 'video'}<FileVideo size={22} />
              {:else if kind(item.path) === 'doc'}<FileText size={22} />
              {:else}<File size={22} />{/if}
            </span>
            <span class="f-name">{fileName(item.path)}</span>
            <span class="f-actions">
              <button aria-label="Show in folder" title="Show in folder" onclick={(e) => { e.stopPropagation(); shelf.reveal(item.path); }}><FolderOpen size={13} /></button>
              <button aria-label="Remove from shelf" title="Remove from shelf" onclick={(e) => { e.stopPropagation(); shelf.remove(item.path); }}><X size={13} /></button>
            </span>
          </div>
        {/each}
      </div>
      <div class="foot"><span>Double-click to open.</span><button onclick={() => shelf.clear()}><Trash2 size={12} /> Clear shelf</button></div>
    {:else}
      <p class="empty">Drag files onto the island to keep them here. They stay where they are on disk; the shelf only remembers them.</p>
    {/if}
  {:else if tab === 'clips'}
    {#if system.clips.length}
      <ul class="clips">
        {#each system.clips.slice(0, 12) as clip (clip.id)}
          <li>
            <button onclick={() => copy(clip.id, clip.text)} title="Copy again">
              <span class="c-text">{clip.text}</span>
              <span class="c-meta num">{#if copiedId === clip.id}<Check size={12} /> Copied{:else}<Copy size={12} /> {ago(clip.at, clock.now)}{/if}</span>
            </button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="empty">Text you copy shows up here, newest first. Copies from password managers are skipped, and nothing is saved to disk.</p>
    {/if}
  {:else}
    <textarea
      class="note"
      placeholder="A quick note. It saves as you type."
      value={note.text}
      oninput={(e) => note.update((e.target as HTMLTextAreaElement).value)}
      spellcheck="false"
    ></textarea>
  {/if}
</div>

<style>
  .shelf {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 2px 2px 0;
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    font-size: var(--t-sm);
    font-weight: 600;
    color: var(--ink-3);
    transition: background 0.15s, color 0.15s;
  }
  .tabs button:hover {
    color: var(--ink);
  }
  .tabs button.on {
    background: var(--fill-2);
    color: var(--ink);
  }
  .tabs .num {
    color: var(--ink-3);
    font-weight: 500;
  }
  .files {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    max-height: 214px;
    overflow-y: auto;
  }
  .file {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    padding: 14px 8px 10px;
    border-radius: 14px;
    background: var(--fill-1);
    cursor: default;
    transition: background 0.15s;
  }
  .file:hover {
    background: var(--fill-2);
  }
  .f-icon {
    color: var(--ink-2);
    display: grid;
  }
  .f-icon.image { color: var(--accent); }
  .f-icon.code { color: var(--ok); }
  .f-icon.archive { color: var(--warn); }
  .f-name {
    width: 100%;
    text-align: center;
    font-size: var(--t-xs);
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    word-break: break-word;
  }
  .f-actions {
    position: absolute;
    top: 5px;
    right: 5px;
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .file:hover .f-actions,
  .file:focus-within .f-actions {
    opacity: 1;
  }
  .f-actions button {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--island);
    color: var(--ink-2);
  }
  .f-actions button:hover {
    color: var(--ink);
  }
  .foot {
    display: flex;
    justify-content: space-between;
    font-size: var(--t-xs);
    color: var(--ink-3);
  }
  .foot button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--ink-3);
  }
  .foot button:hover {
    color: var(--err);
  }
  .clips {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 236px;
    overflow-y: auto;
  }
  .clips button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 10px;
    text-align: left;
  }
  .clips button:hover {
    background: var(--fill-1);
  }
  .c-text {
    flex: 1;
    min-width: 0;
    font-size: var(--t-sm);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .c-meta {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: var(--t-xs);
    color: var(--ink-3);
    white-space: nowrap;
  }
  .note {
    width: 100%;
    height: 180px;
    resize: none;
    padding: 12px 14px;
    border: 0;
    border-radius: 14px;
    background: var(--fill-1);
    font-size: var(--t-md);
    line-height: 1.55;
    color: var(--ink);
    user-select: text;
  }
  .note::placeholder {
    color: var(--ink-3);
  }
  .empty {
    margin: 4px 0 6px;
    max-width: 54ch;
    font-size: var(--t-sm);
    line-height: 1.5;
    color: var(--ink-3);
  }
</style>

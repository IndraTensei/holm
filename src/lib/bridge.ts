// One door to the Rust side. Outside Tauri (a plain browser, used for design work and
// screenshots) every call and event is served by ./mock.ts instead.

type Unlisten = () => void;

export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

let mockModule: Promise<typeof import('./mock')> | null = null;
const mock = () => (mockModule ??= import('./mock'));

export async function call<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!inTauri) return (await mock()).call<T>(cmd, args);
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

/** Fire-and-forget variant for UI actions: errors are logged, never thrown into the UI. */
export function send(cmd: string, args?: Record<string, unknown>): void {
  call(cmd, args).catch((err) => console.warn(`[island] ${cmd} failed`, err));
}

export async function on<T>(event: string, handler: (payload: T) => void): Promise<Unlisten> {
  if (!inTauri) return (await mock()).on<T>(event, handler);
  const { listen } = await import('@tauri-apps/api/event');
  return listen<T>(event, (e) => handler(e.payload));
}

/** GET a JSON document from an allowed public API (through Rust in the app, so CORS never matters). */
export async function getJson<T = unknown>(url: string): Promise<{ status: number; body: T }> {
  if (!inTauri) {
    const res = await fetch(url);
    return { status: res.status, body: (await res.json().catch(() => null)) as T };
  }
  return call<{ status: number; body: T }>('fetch_json', { url });
}

export async function windowLabel(): Promise<string> {
  if (!inTauri) return location.hash.startsWith('#settings') ? 'settings' : 'island';
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  return getCurrentWindow().label;
}

export async function focusWindow(): Promise<void> {
  if (!inTauri) return;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  await getCurrentWindow().setFocus();
}

export type DropEvent =
  | { type: 'enter'; paths: string[] }
  | { type: 'over' }
  | { type: 'drop'; paths: string[] }
  | { type: 'leave' };

export async function onFileDrop(handler: (e: DropEvent) => void): Promise<Unlisten> {
  if (!inTauri) return () => {};
  const { getCurrentWebview } = await import('@tauri-apps/api/webview');
  return getCurrentWebview().onDragDropEvent((event) => {
    const p = event.payload;
    if (p.type === 'enter') handler({ type: 'enter', paths: p.paths });
    else if (p.type === 'drop') handler({ type: 'drop', paths: p.paths });
    else if (p.type === 'over') handler({ type: 'over' });
    else handler({ type: 'leave' });
  });
}

/** Replaces every global shortcut with these. Returns the accelerators that could not be taken. */
export async function registerHotkeys(list: { accelerator: string; handler: () => void }[]): Promise<string[]> {
  if (!inTauri) return [];
  const shortcut = await import('@tauri-apps/plugin-global-shortcut');
  await shortcut.unregisterAll().catch(() => {});
  const failed: string[] = [];
  for (const { accelerator, handler } of list) {
    if (!accelerator) continue;
    try {
      await shortcut.register(accelerator, (e) => {
        if (e.state === 'Pressed') handler();
      });
    } catch (err) {
      console.warn('[island] hotkey', accelerator, err);
      failed.push(accelerator);
    }
  }
  return failed;
}

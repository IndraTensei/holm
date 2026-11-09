// Which system Holm runs on and what works there. Windows has everything; the macOS and Linux
// versions are being built (#10, #11), and what they don't have yet is hidden, not broken.
import { call } from '../bridge';
import type { PlatformInfo } from '../types';

const WINDOWS: PlatformInfo = {
  os: 'windows',
  media: true,
  appVolume: true,
  volume: true,
  radios: true,
  clipboard: true,
  games: true,
  quiet: true,
  frontApp: true,
  battery: true,
  systemStats: true,
};

class Platform {
  info = $state<PlatformInfo>(WINDOWS);

  async load(): Promise<void> {
    const info = await call<PlatformInfo | null>('platform_info').catch(() => null);
    if (info) this.info = info;
  }

  get os() {
    return this.info.os;
  }

  /** "Windows", "macOS" or "Linux", for copy. */
  get name(): string {
    return { windows: 'Windows', macos: 'macOS', linux: 'Linux' }[this.info.os];
  }

  /** What the start-at-login switch is called on this system. */
  get autostartLabel(): string {
    return { windows: 'Start with Windows', macos: 'Open at login', linux: 'Start when you sign in' }[this.info.os];
  }

  /** The tracking issue for a port, for "not available yet" notes. */
  get issue(): string {
    return this.info.os === 'macos' ? 'https://github.com/IndraTensei/holm/issues/10' : 'https://github.com/IndraTensei/holm/issues/11';
  }
}

export const platform = new Platform();

// One shared, low-rate beat for looping decorations: the equalizer, the agent spinner, the
// waiting pulse. The island is a transparent always-on-top WebView2 window, and every animation
// frame recomposites its whole surface: a 60 fps CSS loop measured 13–29% of a core on a real
// desktop. Stepping every loop together at 8 fps paints a fraction of that, and nothing at all
// runs while no loop is on screen or while the island is hidden.

const STEP = 125;

class Beat {
  /** Increments every STEP ms while at least one loop is on screen. */
  n = $state(0);
  private users = 0;
  private paused = false;
  private timer: ReturnType<typeof setTimeout> | null = null;

  /** Call from an $effect while a loop is visible; returns the release function. */
  use(): () => void {
    this.users++;
    this.sync();
    return () => {
      this.users = Math.max(0, this.users - 1);
      this.sync();
    };
  }

  /** Hidden island (full-screen app, or hidden from the tray): stop beating. */
  pause(paused: boolean): void {
    this.paused = paused;
    this.sync();
  }

  private sync(): void {
    const run = this.users > 0 && !this.paused;
    if (run && !this.timer) this.schedule();
    if (!run && this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  // Aligned to wall time like the clock's fast tick (250 ms), so both land in the same frame.
  private schedule(): void {
    const wait = STEP - (Date.now() % STEP) + 5;
    this.timer = setTimeout(() => {
      this.n++;
      this.schedule();
    }, wait);
  }
}

export const beat = new Beat();

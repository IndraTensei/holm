// A shared clock. It ticks fast (4 Hz) only while something live needs it (music progress,
// lyrics, a running timer, an agent's elapsed time) and once every 15 s otherwise.

class Clock {
  now = $state(Date.now());
  private fastUsers = 0;
  private timer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    if (typeof window !== 'undefined') this.restart();
  }

  private restart(): void {
    if (this.timer) clearTimeout(this.timer);
    this.now = Date.now();
    this.schedule();
  }

  /** Slow ticks land on :00, :15, :30 and :45, so the minute on the clock flips on time. */
  private schedule(): void {
    const step = this.fastUsers > 0 ? 250 : 15000;
    const wait = step - (Date.now() % step) + 5;
    this.timer = setTimeout(() => {
      this.now = Date.now();
      this.schedule();
    }, wait);
  }

  /** Call from an $effect; returns the release function. */
  fast(): () => void {
    this.fastUsers++;
    if (this.fastUsers === 1) this.restart();
    return () => {
      this.fastUsers = Math.max(0, this.fastUsers - 1);
      if (this.fastUsers === 0) this.restart();
    };
  }
}

export const clock = new Clock();

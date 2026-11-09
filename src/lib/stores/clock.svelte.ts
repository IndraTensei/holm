// A shared clock. It ticks at 4 Hz only while the music page (lyrics, progress bar) is open, once a second
// while something shows seconds (an agent's elapsed time, a running timer), and once every 15 s
// otherwise. Every tick repaints the island, so the slowest rate that looks right wins.

class Clock {
  now = $state(Date.now());
  private fastUsers = 0;
  private secondUsers = 0;
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
    const step = this.fastUsers > 0 ? 250 : this.secondUsers > 0 ? 1000 : 15000;
    const wait = step - (Date.now() % step) + 5;
    this.timer = setTimeout(() => {
      this.now = Date.now();
      this.schedule();
    }, wait);
  }

  /** Once a second, for things that show seconds. Call from an $effect; returns the release function. */
  second(): () => void {
    this.secondUsers++;
    if (this.secondUsers === 1) this.restart();
    return () => {
      this.secondUsers = Math.max(0, this.secondUsers - 1);
      if (this.secondUsers === 0) this.restart();
    };
  }

  /** 4 Hz, for the music page's lyrics and progress bar. Call from an $effect; returns the release function. */
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

// Tiny synthesized chimes: no audio files to ship, and they follow the system volume.

let ctx: AudioContext | null = null;

function tone(freq: number, start: number, length: number, gain: number): void {
  if (!ctx) return;
  const osc = ctx.createOscillator();
  const amp = ctx.createGain();
  osc.type = 'sine';
  osc.frequency.value = freq;
  const t = ctx.currentTime + start;
  amp.gain.setValueAtTime(0, t);
  amp.gain.linearRampToValueAtTime(gain, t + 0.012);
  amp.gain.exponentialRampToValueAtTime(0.0001, t + length);
  osc.connect(amp).connect(ctx.destination);
  osc.start(t);
  osc.stop(t + length + 0.05);
}

export function chime(kind: 'done' | 'ask' | 'timer'): void {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === 'suspended') void ctx.resume();
    if (kind === 'done') {
      tone(880, 0, 0.5, 0.08);
      tone(1318.5, 0.11, 0.7, 0.07);
    } else if (kind === 'ask') {
      tone(740, 0, 0.18, 0.07);
      tone(740, 0.2, 0.25, 0.07);
    } else {
      tone(659.3, 0, 0.4, 0.08);
      tone(987.8, 0.16, 0.4, 0.07);
      tone(1318.5, 0.32, 0.8, 0.06);
    }
  } catch {
    /* audio unavailable */
  }
}

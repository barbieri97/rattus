// Chamber sounds, synthesized with Web Audio (no audio files needed).

let ctx: AudioContext | null = null;
let master: GainNode | null = null;
let volume = 0.6;
let tone: { osc: OscillatorNode; gain: GainNode } | null = null;
let bellTimer: ReturnType<typeof setInterval> | null = null;

function audio(): { ctx: AudioContext; out: GainNode } | null {
  if (typeof window === "undefined" || typeof AudioContext === "undefined") return null;
  if (!ctx) {
    ctx = new AudioContext();
    master = ctx.createGain();
    master.gain.value = volume;
    master.connect(ctx.destination);
  }
  if (ctx.state === "suspended") void ctx.resume();
  return { ctx, out: master! };
}

/** Browsers only allow audio after a user gesture; call this from one. */
export function unlockAudio(): void {
  audio();
}

/** Sets the overall loudness (the "Sound Proofing" preference), 0 to 1. */
export function setVolume(v: number): void {
  volume = Math.min(1, Math.max(0, v));
  if (master && ctx) master.gain.setTargetAtTime(volume, ctx.currentTime, 0.02);
}

/** The mechanical click of the pellet dispenser. */
export function playClick(): void {
  const a = audio();
  if (!a || volume <= 0) return;
  const { ctx, out } = a;
  const now = ctx.currentTime;
  for (const [delay, level] of [
    [0, 0.9],
    [0.035, 0.5],
  ] as const) {
    const length = Math.floor(ctx.sampleRate * 0.025);
    const buffer = ctx.createBuffer(1, length, ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < length; i++) {
      data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (length / 6));
    }
    const src = ctx.createBufferSource();
    src.buffer = buffer;
    const filter = ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.value = 2400;
    filter.Q.value = 1.2;
    const gain = ctx.createGain();
    gain.gain.value = level;
    src.connect(filter).connect(gain).connect(out);
    src.start(now + delay);
  }
}

/** Starts (or changes) the tone CS. Louder in dB means louder here too. */
export function startTone(db: number): void {
  const a = audio();
  if (!a) return;
  const { ctx, out } = a;
  const level = Math.min(0.5, Math.max(0.03, (db - 55) / 100));
  if (!tone) {
    const osc = ctx.createOscillator();
    osc.type = "sine";
    osc.frequency.value = 1000;
    const gain = ctx.createGain();
    gain.gain.value = 0;
    osc.connect(gain).connect(out);
    osc.start();
    tone = { osc, gain };
  }
  tone.gain.gain.setTargetAtTime(level, ctx.currentTime, 0.01);
}

export function stopTone(): void {
  if (!tone || !ctx) return;
  const t = tone;
  tone = null;
  t.gain.gain.setTargetAtTime(0, ctx.currentTime, 0.01);
  t.osc.stop(ctx.currentTime + 0.1);
}

function strikeBell(): void {
  const a = audio();
  if (!a) return;
  const { ctx, out } = a;
  const now = ctx.currentTime;
  for (const [freq, level] of [
    [880, 0.25],
    [2200, 0.1],
    [3450, 0.05],
  ] as const) {
    const osc = ctx.createOscillator();
    osc.frequency.value = freq;
    const gain = ctx.createGain();
    gain.gain.setValueAtTime(level, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.6);
    osc.connect(gain).connect(out);
    osc.start(now);
    osc.stop(now + 0.65);
  }
}

export function startBell(): void {
  if (bellTimer) return;
  strikeBell();
  bellTimer = setInterval(strikeBell, 450);
}

export function stopBell(): void {
  if (bellTimer) clearInterval(bellTimer);
  bellTimer = null;
}

/** A short electric buzz for the shock. */
export function playShock(): void {
  const a = audio();
  if (!a || volume <= 0) return;
  const { ctx, out } = a;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  osc.type = "sawtooth";
  osc.frequency.value = 90;
  const gain = ctx.createGain();
  gain.gain.setValueAtTime(0.12, now);
  gain.gain.setTargetAtTime(0, now + 0.8, 0.05);
  osc.connect(gain).connect(out);
  osc.start(now);
  osc.stop(now + 1.1);
}

export function stopAll(): void {
  stopTone();
  stopBell();
}

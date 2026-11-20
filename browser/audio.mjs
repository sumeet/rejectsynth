import { SAMPLE_RATE } from '../vscode-extension/engine.mjs';

// Render ahead by a fraction of a second; use the audio clock for decorations.
// The original extension's Node speaker/stream is the only replaced subsystem.
export class BrowserAudio {
  constructor({ contextFactory = () => new AudioContext(), clock = globalThis } = {}) {
    this.contextFactory = contextFactory;
    this.clock = clock;
    this.enabled = false;
    this.volume = 0.5;
  }

  async unlock() {
    if (!this.context) {
      this.context = this.contextFactory();
      this.gain = this.context.createGain();
      this.gain.gain.value = this.volume;
      this.gain.connect(this.context.destination);
    }
    await this.context.resume();
    if (this.context.state !== 'running') throw new Error('Press Play to enable audio.');
    this.enabled = true;
  }

  setVolume(value) {
    this.volume = value;
    if (this.gain) this.gain.gain.setTargetAtTime(value, this.context.currentTime, 0.01);
  }

  stop() {
    const run = this.run;
    if (!run) return;
    this.run = undefined;
    this.clock.clearInterval(run.timer);
    run.frames.dispose();
    for (const source of run.sources) { source.stop(); source.disconnect(); }
    run.sources.clear();
  }

  play(frames, callbacks) {
    this.stop();
    if (!this.enabled || this.context.state !== 'running') {
      frames.dispose();
      throw new Error('Press Play to enable audio.');
    }
    const run = { frames, sources: new Set(), events: [], nextTime: this.context.currentTime + 0.04,
      done: false, callbacks, timer: undefined };
    this.run = run;
    const tick = () => {
      if (this.run !== run) return;
      try {
        const now = this.context.currentTime;
        let budget = 256;
        while (!run.done && run.nextTime < now + 0.15 && budget-- > 0) {
          const frame = frames.next();
          if (!frame) { run.done = true; frames.dispose(); break; }
          if (!frame.samples.length) continue;
          if (!frame.samples.every(Number.isFinite)) throw new Error('This edit produced invalid audio.');
          const buffer = this.context.createBuffer(1, frame.samples.length, SAMPLE_RATE);
          buffer.copyToChannel(frame.samples, 0);
          const source = this.context.createBufferSource();
          source.buffer = buffer;
          source.connect(this.gain);
          run.sources.add(source);
          source.onended = () => { run.sources.delete(source); source.disconnect(); };
          // Recover from a background-tab scheduling gap without overlapping notes.
          run.nextTime = Math.max(run.nextTime, now);
          source.start(run.nextTime);
          run.events.push({ time: run.nextTime, syntaxes: frame.syntaxes });
          run.nextTime += frame.samples.length / SAMPLE_RATE;
        }
        while (run.events.length && run.events[0].time <= now) {
          callbacks.onFrame(run.events.shift().syntaxes);
        }
        if (run.done && now >= run.nextTime) { this.stop(); callbacks.onEnd(); }
      } catch (error) {
        this.stop();
        callbacks.onError(error);
      }
    };
    run.timer = this.clock.setInterval(tick, 20);
    tick();
  }

  dispose() { this.stop(); void this.context?.close(); }
}

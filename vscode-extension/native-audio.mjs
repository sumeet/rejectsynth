import Speaker from 'speaker';
import { Readable } from 'node:stream';
import { SAMPLE_RATE } from './engine.mjs';

export function createNativeAudio() {
  let current;
  function stop() {
    if (!current) return;
    const playing = current;
    current = undefined;
    playing.timers.forEach(clearTimeout);
    playing.frames.dispose();
    playing.stream.destroy();
    playing.speaker.close();
  }
  return {
    enabled: true,
    unlock: async () => {},
    stop,
    dispose: stop,
    play(frames, callbacks) {
      stop();
      const speaker = new Speaker({ channels: 1, bitDepth: 32, sampleRate: SAMPLE_RATE, float: true });
      const playing = { speaker, frames, timers: [], samples: 0, started: Date.now() };
      current = playing;
      const stream = new Readable({ read() {
        try {
          for (;;) {
            const frame = frames.next();
            if (!frame) { frames.dispose(); this.push(null); return; }
            if (!frame.samples.length) continue;
            const time = playing.started + playing.samples / SAMPLE_RATE * 1000 + 100;
            playing.timers.push(setTimeout(() => {
              if (current === playing) callbacks.onFrame(frame.syntaxes);
            }, Math.max(0, time - Date.now())));
            playing.samples += frame.samples.length;
            this.push(Buffer.from(frame.samples.buffer, frame.samples.byteOffset, frame.samples.byteLength));
            return;
          }
        } catch (error) { this.destroy(error); }
      } });
      playing.stream = stream;
      const fail = error => { if (current === playing) { stop(); callbacks.onError(error); } };
      stream.on('error', fail);
      speaker.on('error', fail);
      speaker.on('close', () => {
        if (current === playing) { stop(); callbacks.onEnd(); }
      });
      stream.pipe(speaker);
    },
  };
}

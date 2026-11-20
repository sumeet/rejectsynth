import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, * as reject from '../generated/rejectsynth.js';
import { syntax, songFrames, previewFrames, SAMPLE_RATE } from '../../vscode-extension/engine.mjs';
import { BrowserAudio } from '../audio.mjs';

await init({ module_or_path: await readFile(new URL('../generated/rejectsynth_bg.wasm', import.meta.url)) });

function collect(frames) {
  const result = [];
  try {
    for (let frame; (frame = frames.next());) if (frame.samples.length) result.push(frame);
    return result;
  } finally { frames.dispose(); }
}

test('the existing Kalm song produces finite audio and matching highlight spans', async () => {
  const text = await readFile(new URL('../../songs/kalm.rej', import.meta.url), 'utf8');
  const tokens = syntax(reject, text);
  assert.equal(tokens[0].node_type, 'SetBPM');
  assert.ok(tokens.some(node => node.node_type === 'SetHarmony'));
  const frames = collect(songFrames(reject, text));
  assert.ok(frames.length > 20);
  let samples = 0;
  for (const frame of frames) {
    assert.ok(frame.samples.every(value => Number.isFinite(value) && Math.abs(value) <= 1.01));
    assert.ok(frame.syntaxes.some(node => node.node_type === 'PlayNote'));
    samples += frame.samples.length;
  }
  assert.ok(samples / SAMPLE_RATE > 10);
});

test('selection keeps preceding tempo, key, scale and harmony', () => {
  const text = 'bpm 120\nkey E\nscale minor\nIII: 1 ~2 3';
  const all = collect(songFrames(reject, text));
  const start = text.indexOf('~2');
  const selected = collect(songFrames(reject, text, start, start + 1));
  assert.equal(selected.length, 1);
  assert.equal(selected[0].samples.length, SAMPLE_RATE / 4);
  assert.ok(selected[0].syntaxes.some(node => node.node_type === 'SetHarmony'));
  assert.equal(all[1].samples.length, selected[0].samples.length);
  // Phase differs because preceding notes are skipped; frequency content still
  // changes when the preceding key/harmony changes.
  const other = collect(songFrames(reject, text.replace('key E', 'key C'), start, start + 1));
  assert.notDeepEqual(other[0].samples.subarray(0, 100), selected[0].samples.subarray(0, 100));
});

test('typing previews a note, including its preceding harmony', () => {
  const text = 'bpm 120\nkey E\nscale minor\nIII: 1 ~2 3';
  const pos = text.indexOf('~2');
  const preview = collect(previewFrames(reject, text, pos));
  const selection = collect(songFrames(reject, text, pos, pos + 1));
  assert.deepEqual(preview[0].samples, selection[0].samples);
});

test('incomplete/invalid edits return errors and the same WASM instance recovers', () => {
  for (const text of ['key Q', 'bpm ', 'bpm 0\n1', '0']) {
    assert.throws(() => syntax(reject, text));
    assert.throws(() => songFrames(reject, text));
    assert.equal(collect(previewFrames(reject, text, 0)).length, 0);
    assert.equal(collect(songFrames(reject, 'bpm 120\n1')).length, 1);
  }
  assert.equal(collect(songFrames(reject, '')).length, 0);
});

function fakeAudio() {
  const callbacks = new Set();
  const sources = [];
  const context = {
    state: 'suspended', currentTime: 0, destination: {},
    async resume() { this.state = 'running'; },
    async close() { this.state = 'closed'; },
    createGain() { return { gain: { value: 0, setTargetAtTime() {} }, connect() {} }; },
    createBuffer(channels, length, rate) {
      assert.equal(channels, 1);
      assert.equal(rate, SAMPLE_RATE);
      return { duration: length / rate, copyToChannel(data) { assert.equal(data.length, length); } };
    },
    createBufferSource() {
      const source = { connect() {}, disconnect() {}, stop() { this.stopped = true; }, start(at) { this.at = at; } };
      sources.push(source);
      return source;
    },
  };
  const clock = {
    setInterval(fn) { callbacks.add(fn); return fn; },
    clearInterval(fn) { callbacks.delete(fn); },
  };
  return { context, sources, callbacks,
    audio: new BrowserAudio({ contextFactory: () => context, clock }),
    advance(time) { context.currentTime = time; for (const fn of [...callbacks]) fn(); },
  };
}

test('browser schedules PCM and note highlights against the same audio clock', async () => {
  const rig = fakeAudio();
  await rig.audio.unlock();
  const heard = [];
  let ended = false;
  rig.audio.play(songFrames(reject, 'bpm 120\n1 2'), {
    onFrame: nodes => heard.push(nodes), onEnd: () => { ended = true; }, onError: assert.fail,
  });
  assert.equal(rig.sources.length, 1);
  assert.equal(rig.sources[0].at, 0.04);
  assert.equal(heard.length, 0);
  rig.advance(0.05);
  assert.equal(heard.length, 1);
  rig.advance(0.45);
  assert.equal(rig.sources[1].at, 0.54);
  rig.advance(0.55);
  assert.equal(heard.length, 2);
  rig.advance(1.05);
  assert.equal(ended, true);
  assert.equal(rig.callbacks.size, 0);
});

test('Stop cancels scheduled audio and pending highlights; another song can play', async () => {
  const rig = fakeAudio();
  await rig.audio.unlock();
  let highlighted = 0;
  const callbacks = { onFrame() { highlighted++; }, onEnd() {}, onError: assert.fail };
  rig.audio.play(songFrames(reject, '1 2 3'), callbacks);
  rig.audio.stop();
  assert.equal(rig.sources[0].stopped, true);
  rig.advance(1);
  assert.equal(highlighted, 0);
  rig.audio.play(songFrames(reject, '4'), callbacks);
  rig.advance(1.05);
  assert.equal(highlighted, 1);
  rig.audio.stop();
  assert.equal(rig.callbacks.size, 0);
});

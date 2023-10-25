const { parentPort } = require('worker_threads');
const Speaker = require('speaker');
const { Readable } = require('stream');

let speaker;

function resetSpeaker() {
  if (speaker) speaker.close();
  speaker = new Speaker({
    channels: 1,
    bitDepth: 32,
    sampleRate: 44100,
    float: true,
  });
}

resetSpeaker();

const sampless = [];
let zeros = [];
for (let i = 0; i < 16384; i++) zeros.push(0);
zeros = Buffer.from(new Float32Array(zeros).buffer);

class GlobalStreamer extends Readable {
  _read(size) {
    if (sampless.length === 0) {
      console.log('child pushing zeros....');
      return this.push(zeros.slice(0, size));
    }

    const samples = sampless.shift();
    let slice = samples.slice(0, size);
    console.log('child pushing slice of length', slice.length);
    this.push(slice);
    console.log('child pushed slice of length', slice.length);
    if (samples.length > size) {
      samples = samples.slice(size)
      sampless.unshift(samples);
    }
  }
}

const gs = new GlobalStreamer();
gs.pipe(speaker);

parentPort.on('message', samples => {
  console.log('child: received samples:', samples instanceof Float32Array);
  sampless.push(Buffer.from(samples.buffer));
});

// if (!samples) return resetSpeaker();

// const buf = Buffer.from(samples.buffer);
// const bufStreamer = new BufStreamer(buf);
// resetSpeaker();
// bufStreamer.pipe(speaker);

class BufStreamer extends Readable {
  constructor(buffer) {
    super();
    this.buffer = buffer;
    this.position = 0;
  }

  _read(size) {
    const chunk = this.buffer.slice(this.position, this.position + size);
    console.log(`playing ${chunk.length} audios...`);
    this.push(chunk);
    this.position += size;

    if (this.position >= this.buffer.length) {
      this.push(null);
    }
  }
}

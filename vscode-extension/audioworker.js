const { workerData, parentPort } = require('worker_threads');
const { sharedBuffer, sb2 } = workerData;
const intlock = new Int32Array(sb2);
// const loq = new AsyncLock(sharedBuffer);
const Speaker = require('speaker');
const { Readable } = require('stream');

// needed to keep this thread alive used in combination with waitAsync
// see: https://github.com/nodejs/node/issues/44729
parentPort.ref();

const lock = new Int32Array(sharedBuffer, 0, 1);
const length = new Int32Array(sharedBuffer, 4, 1);

let speaker = new Speaker({
    channels: 1,
    bitDepth: 32,
    sampleRate: 44100,
    float: true,
});

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

// arrays are initialized to 0

// 0: default state
//    parent will write data into the shared buffer
//    and then will set the lock to 1

// 1: child will read data from the shared buffer
//    and send the data to node-speaker. and then
//    will set the lock to 0, and wait again

// (async () => {
//     while (true) {
//         console.log('child: waiting for sender to fix the lock');
//         // let value = await Atomics.waitAsync(lock, 0, 0);
//         console.log(`child: current lock value: ${lock[0]}`);
//         // Atomics.wait(lock, 0, 0);
//         console.log('child: value1', value);
//         // value = await value.value;
//         console.log('child: value2', value);
//         console.log('child: done for sender lock');

//         console.log('child: preparing data for pipe');
//         const len = length[0];
//         const audioData = Buffer.from(sharedBuffer, 8, len * 4);
//         const stream = new BufStreamer(audioData);
//         stream.pipe(speaker);
//         console.log('child: piped');

//         console.log('child: unlocking lock for sender');
//         // Atomics.store(lock, 0, 0);
//         // Atomics.notify(lock, 0);
//     }
// })();

async function pipeAsync(stream, speaker) {
    return new Promise((resolve, reject) => {
        stream.pipe(speaker)
            .on('finish', resolve)
            .on('error', reject);
    });
}

(async function () {
    while (true) {
        console.log('child: waiting for parent to release lock...');
        let value = Atomics.waitAsync(intlock, 0, 0); // wait until lock value becomes non-zero
        if (value.value instanceof Promise) await value.value;
        console.log('child: done waiting for parent to release lock...');

        // Do the work here
        console.log('child: streaming the audio');
        const len = length[0];
        const audioData = Buffer.from(sharedBuffer, 8, len * 4);
        const stream = new BufStreamer(audioData);

        await pipeAsync(stream, speaker);
        // stream.pipe(speaker);

        console.log('child: done streaming the audio and unlocking parent');

        Atomics.store(intlock, 0, 0); // reset lock to zero
        Atomics.notify(intlock, 0);
    }
})();
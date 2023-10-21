class AsyncLock {
    static INDEX = 0;
    static UNLOCKED = 0;
    static LOCKED = 1;

    constructor(sab) {
        this.sab = sab;
        this.i32a = new Int32Array(sab);
    }

    lock() {
        while (true) {
            const oldValue = Atomics.compareExchange(this.i32a, AsyncLock.INDEX,
                        /* old value >>> */  AsyncLock.UNLOCKED,
                        /* new value >>> */  AsyncLock.LOCKED);
            if (oldValue == AsyncLock.UNLOCKED) {
                return;
            }
            Atomics.wait(this.i32a, AsyncLock.INDEX,
                AsyncLock.LOCKED); // <<< expected value at start
        }
    }

    unlock() {
        const oldValue = Atomics.compareExchange(this.i32a, AsyncLock.INDEX,
                            /* old value >>> */  AsyncLock.LOCKED,
                            /* new value >>> */  AsyncLock.UNLOCKED);
        if (oldValue != AsyncLock.LOCKED) {
            throw new Error('Tried to unlock while not holding the mutex');
        }
        Atomics.notify(this.i32a, AsyncLock.INDEX, 1);
    }

    executeLocked(f) {
        const self = this;

        async function tryGetLock() {
            while (true) {
                const oldValue = Atomics.compareExchange(self.i32a, AsyncLock.INDEX,
                                    /* old value >>> */  AsyncLock.UNLOCKED,
                                    /* new value >>> */  AsyncLock.LOCKED);
                if (oldValue == AsyncLock.UNLOCKED) {
                    f();
                    self.unlock();
                    return;
                }
                const result = Atomics.waitAsync(self.i32a, AsyncLock.INDEX,
                    AsyncLock.LOCKED);
                //  ^ expected value at start
                await result.value;
            }
        }

        tryGetLock();
    }
}

const { workerData } = require('worker_threads');
const { sharedBuffer } = workerData;
const loq = new AsyncLock(sharedBuffer);
const Speaker = require('speaker');
const { Readable } = require('stream');

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

(async () => {
    while (true) {
        console.log('child: waiting for sender to fix the lock');
        // let value = await Atomics.waitAsync(lock, 0, 0);
        console.log(`child: current lock value: ${lock[0]}`);
        // Atomics.wait(lock, 0, 0);
        console.log('child: value1', value);
        // value = await value.value;
        console.log('child: value2', value);
        console.log('child: done for sender lock');

        console.log('child: preparing data for pipe');
        const len = length[0];
        const audioData = Buffer.from(sharedBuffer, 8, len * 4);
        const stream = new BufStreamer(audioData);
        stream.pipe(speaker);
        console.log('child: piped');

        console.log('child: unlocking lock for sender');
        // Atomics.store(lock, 0, 0);
        // Atomics.notify(lock, 0);
    }
})();


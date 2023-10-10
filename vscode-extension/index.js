const vscode = require('vscode');
const Speaker = require('speaker');
const { Readable } = require('stream');

const reject = require('../build/wasm/rejectsynth.js');

function activate(context) {
    let samples = reject.sup();

    let speaker = new Speaker({
        channels: 1,
        bitDepth: 32,
        sampleRate: 44100,
        float: true,
    });

    const buffer = Buffer.from(samples.buffer);
    speaker.write(buffer);
}

function deactivate() { }

module.exports = {
    activate,
    deactivate
};

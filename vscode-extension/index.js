// Desktop entry point. The browser uses the same extension with a Web Audio adapter.
const vscode = require('vscode');
const reject = require('../build/wasm/rejectsynth.js');

exports.activate = async function activate(context) {
  const { activateRejectsynth } = await import('./extension.mjs');
  const { createNativeAudio } = await import('./native-audio.mjs');
  return activateRejectsynth(context, {
    vscode,
    reject,
    audio: createNativeAudio(),
    onError: message => vscode.window.showErrorMessage(message),
  });
};

exports.deactivate = function deactivate() {};

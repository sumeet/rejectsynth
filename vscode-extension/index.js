const vscode = require('vscode');
const Speaker = require('speaker');

const reject = require('../build/wasm/rejectsynth.js');

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

// https://www.sublimetext.com/docs/scope_naming.html#keyword
const TOKEN_TYPES = [
  'keyword',
  'number',
  'operator',
  'parameter',
]

const TOKEN_TYPE_INIDICES = {};
for (let i = 0; i < TOKEN_TYPES.length; i++) {
  TOKEN_TYPE_INIDICES[TOKEN_TYPES[i]] = i;
}

class MySemanticTokensProvider {
  async provideDocumentSemanticTokens(doc) {
    const builder = new vscode.SemanticTokensBuilder();
    for (const syntax of reject.syntax(doc.getText())) {
      let token_type = "keyword";
      switch (syntax.node_type) {
        case "SetKey":
        case "SetBPM":
        case "SetScale":
          token_type = "keyword";
          break;
        case "PlayNote":
          token_type = "number";
          break;
        case "SkipToNote":
          token_type = "operator";
          break;
        case "SetHarmony":
          token_type = "parameter";
          break;
        default:
          console.error(new Error("Unknown node type: " + syntax.node_type));
      }
      const t = TOKEN_TYPE_INIDICES[token_type];
      builder.push(syntax.line_no, syntax.col_no, syntax.len, t);
    }
    return builder.build();
  }
}

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

let decorationType;

function clearDecorations() {
  let editor = vscode.window.activeTextEditor;
  if (!editor) return;
  while (highlightTimeouts.length > 0) {
    clearTimeout(highlightTimeouts.pop());
  }
  if (decorationType) {
    editor.setDecorations(decorationType, []);
    decorationType.dispose();
    decorationType = undefined;
  }
}

const highlightTimeouts = [];

function highlight(syntaxes) {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  if (!decorationType) {
    decorationType = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(220, 220, 220, 0.5)'
    });
  }

  const ranges = [];
  for (const syntax of syntaxes) {
    const start = new vscode.Position(syntax.line_no, syntax.col_no);
    const end = editor.document.positionAt(editor.document.offsetAt(start) + syntax.len);
    ranges.push(new vscode.Range(start, end));
  }
  editor.setDecorations(decorationType, ranges);
}

function activate(context) {
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((e) => {
    const changes = e.contentChanges;
    if (changes.length === 0) return;
    if (changes.length > 1) throw new Error("thought changes.length is always 1 or 0, time to update our assumptions");
    const [change] = changes;
    if (/^\s*$/g.test(change.text)) {
      return;
    }

    let l = e.document.offsetAt(change.range.start);
    let r = e.document.offsetAt(change.range.end);
    if (l !== r) throw new Error("expected range to just be a single character, update our assumptions");
    let samples = reject.playback_for_note_input(e.document.getText(), l);
    if (samples.length > 0) {
      sendDataToWorker(samples);
      // let bufStreamer = new BufStreamer(Buffer.from(samples.buffer));
      // resetSpeaker();
      // bufStreamer.pipe(speaker);
    }
  }));

  context.subscriptions.push(
    vscode.languages.registerCodeLensProvider(
      { language: 'rejectsynth' },
      {
        provideCodeLenses: (doc) => {
          // "Play" code lens from beginning to end of doc
          return [
            new vscode.CodeLens(
              new vscode.Range(0, 0, doc.lineCount, 0),
              {
                title: "⏵️Play",
                command: "rejectsynth.playFromHere",
                arguments: [],
              }
            )
          ];
        }
      },
    )
  );

  context.subscriptions.push(vscode.languages.registerDocumentSemanticTokensProvider(
    { language: 'rejectsynth' },


    new MySemanticTokensProvider(),
    new vscode.SemanticTokensLegend(TOKEN_TYPES),
  ));

  context.subscriptions.push(
    vscode.commands.registerCommand('rejectsynth.stopPlaying', () => {
      resetSpeaker();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('rejectsynth.playFromHere', () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      const position = editor.selection.active;

      resetSpeaker();

      const iter = reject.WasmSongIterator.from_song_text(editor.document.getText());
      const bufStreamer = new IterStreamer(iter);

      let disposableStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
      disposableStatusBarItem.text = `$(stop) Stop`;
      disposableStatusBarItem.command = 'rejectsynth.stopPlaying';
      disposableStatusBarItem.show();
      speaker.on('close', () => {
        clearDecorations();
        disposableStatusBarItem.dispose();
      });

      bufStreamer.pipe(speaker);
    })
  );
}

function deactivate() { }

module.exports = {
  activate,
  deactivate
};

const { Readable } = require('stream');

class IterStreamer extends Readable {
  constructor(iter) {
    super();
    this.iter = iter;
    this.buffer = Buffer.alloc(0);
  }

  _read(size) {
    while (this.buffer.length < size && !this.iter.is_done()) {
      const playbackResult = this.iter.play_next();
      highlightTimeouts.push(setTimeout(() => highlight(playbackResult.on_syntaxes), 500));
      this.buffer = Buffer.concat(
        [this.buffer, Buffer.from(playbackResult.samples.buffer)]);
    }
    if (this.buffer.length > 0) {
      this.push(this.buffer.slice(0, size));
      this.buffer = this.buffer.slice(size);
    } else {
      this.push(null);
    }
  }
}

class BufStreamer extends Readable {
  constructor(buffer) {
    super();
    this.buffer = buffer;
    this.position = 0;
  }

  _read(size) {
    const chunk = this.buffer.slice(this.position, this.position + size);
    this.push(chunk);
    this.position += size;

    if (this.position >= this.buffer.length) {
      this.push(null);
    }
  }
}

const { Worker } = require('worker_threads');
const BUFFER_SAMPLE_CAP = 44100;

const sb2 = new SharedArrayBuffer(4);
const intlock = new Int32Array(sb2);

const sharedBuffer = new SharedArrayBuffer(4 + 4 + BUFFER_SAMPLE_CAP * 4);
const lock = new Int32Array(sharedBuffer, 0, 1);
const length = new Int32Array(sharedBuffer, 4, 1);
const sharedAudioData = new Float32Array(sharedBuffer, 8);

const loq = new AsyncLock(sharedBuffer);

// this should return right away because nobody is holding the lock yet
loq.lock();

const worker = new Worker('./audioworker.js', {
  workerData: { sharedBuffer, sb2 }
});

async function sendDataToWorker(audioData) {
  if (intlock[0] !== 0) {
    let result = await Atomics.waitAsync(intlock, 0, 0);
    console.log('parent: waiting2 for lock from child', result);
    if (result.value instanceof Promise) await result.value;
    console.log('parent: done waiting for lock from child');
  }

  console.log('parent: setting shared buffer');
  length[0] = audioData.length;
  sharedAudioData.set(audioData, 0);

  console.log('parent: waking up child');
  Atomics.store(intlock, 0, 1);
  Atomics.notify(intlock, 0);
};

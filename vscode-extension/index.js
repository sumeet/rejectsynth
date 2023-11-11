const vscode = require('vscode');
const Speaker = require('speaker');

const reject = require('../build/wasm/rejectsynth.js');

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

resetSpeaker();

let playbackBGDecorationType;

function audioLengthMs(numSamples) {
  return numSamples / 44100 * 1000;
}

function clearDecorations() {
  let editor = vscode.window.activeTextEditor;
  if (!editor) return;
  while (highlightTimeouts.length > 0) {
    clearTimeout(highlightTimeouts.pop());
  }
  if (playbackBGDecorationType) {
    editor.setDecorations(playbackBGDecorationType, []);
    playbackBGDecorationType.dispose();
    playbackBGDecorationType = undefined;
  }
}

const highlightTimeouts = [];

function highlight(syntaxes) {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  if (!playbackBGDecorationType) {
    playbackBGDecorationType = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(220, 220, 220, 0.5)'
    });
  }

  const ranges = [];
  for (const syntax of syntaxes) {
    const start = new vscode.Position(syntax.line_no, syntax.col_no);
    const end = editor.document.positionAt(editor.document.offsetAt(start) + syntax.len);
    ranges.push(new vscode.Range(start, end));
  }
  editor.setDecorations(playbackBGDecorationType, ranges);
}

let lastPlayMs = 0;

function setLastPlayMs(source, offset=0) {
  console.log(`setLastPlayMs: from ${source}, offset ${offset}`);
  lastPlayMs = ms;
}

function activate(context) {
  context.subscriptions.push(vscode.window.onDidChangeTextEditorSelection(e => {
    vscode.commands.executeCommand('rejectsynth.playSelection');
  }));

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
      lastPlayMs = Date.now();

      let buf = Buffer.from(samples.buffer);
      resetSpeaker();
      speaker.write(buf);
      setLastPlayMs('onDidChangeTextDocument', Date.now() + audioLengthMs());
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
                command: "rejectsynth.playWholeFile",
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
    vscode.commands.registerCommand('rejectsynth.playWholeFile', () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;

      const iter = reject.WasmSongIterator.from_song_text(editor.document.getText());
      const iterStreamer = new IterStreamer(iter);


      let disposableStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
      disposableStatusBarItem.text = `$(stop) Stop`;
      disposableStatusBarItem.command = 'rejectsynth.stopPlaying';
      disposableStatusBarItem.show();
      resetSpeaker();
      speaker.on('close', () => {
        clearDecorations();
        disposableStatusBarItem.dispose();
      });

      iterStreamer.pipe(speaker);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand('rejectsynth.playSelection', () => {
      let afterTypingMs = Date.now() - lastPlayMs;
      if(afterTypingMs < 1000)
        return console.log("cancelled playSelection");

      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      const l = editor.document.offsetAt(editor.selection.start);
      const r = editor.document.offsetAt(editor.selection.end);

      const iter = reject.WasmSongIterator.from_song_text(editor.document.getText(), l, r);
      const iterStreamer = new IterStreamer(iter);

      let disposableStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
      disposableStatusBarItem.text = `$(stop) Stop`;
      disposableStatusBarItem.command = 'rejectsynth.stopPlaying';
      disposableStatusBarItem.show();
      resetSpeaker();
      speaker.on('close', () => {
        clearDecorations();
        disposableStatusBarItem.dispose();
        setLastPlayMs('closing from iter in playSelection');
      });

      iterStreamer.pipe(speaker);
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

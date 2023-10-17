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

let decorationType;

function clearDecorations() {
  let editor = vscode.window.activeTextEditor;
  if (!editor) return;
  while (highlightTimeouts.length > 0) {
    clearTimeout(highlightTimeouts.pop());
  }
  editor.setDecorations(decorationType, []);
  decorationType.dispose();
  decorationType = undefined;
}

const highlightTimeouts = [];

function highlight(syntax) {
  let editor = vscode.window.activeTextEditor;
  if (!editor) return;
  if (!decorationType) {
    decorationType = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(220, 220, 220, 0.5)'
    });
  }

  let start = new vscode.Position(syntax.line_no, syntax.col_no);
  let end = editor.document.positionAt(editor.document.offsetAt(start) + syntax.len);
  editor.setDecorations(decorationType, [new vscode.Range(start, end)]);
}


function activate(context) {
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

//////////////////////////////////////////////////
// cursor moving stuff, which we'll go later
//////////////////////////////////////////////////


//////////////////////////////////////////////////
// song playing stuff, which we'll do later
//////////////////////////////////////////////////


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
      highlightTimeouts.push(setTimeout(() => highlight(playbackResult.syntax), 500));
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

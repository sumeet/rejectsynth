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


function activate(context) {
  ////////////////////////////
  // EXPERIMENT
  ////////////////////////////
  let editor = vscode.window.activeTextEditor;
  let currentPosition = 0;
  const decorationType = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(220, 220, 220, 0.5)'
  });

  if (editor) {
    setInterval(() => {
      const text = editor.document.getText();
      const nextSpace = text.indexOf(' ', currentPosition + 1);
      const start = editor.document.positionAt(currentPosition);
      const end = editor.document.positionAt(nextSpace);

      editor.setDecorations(decorationType, [new vscode.Range(start, end)]);
      currentPosition = nextSpace + 1;

    }, 1000);
  }
  ////////////////////////////
  // END OF EXPERIMENT
  ////////////////////////////

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
      console.log(position);

      resetSpeaker();

      let samples = reject.samples(editor.document.getText());
      const buffer = Buffer.from(samples.buffer);
      // const bufStreamer = new BufStreamer(buffer);
      const iter = reject.WasmSongIterator.from_song_text(editor.document.getText());
      const bufStreamer = new IterStreamer(iter);

      let disposableStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
      disposableStatusBarItem.text = `$(stop) Stop`;
      // Attach an event to stop the song
      disposableStatusBarItem.command = 'rejectsynth.stopPlaying';
      disposableStatusBarItem.show();

      speaker.on('close', () => {
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
    this.buffer = [];
  }

  _read(size) {
    console.log('reading for size', size);
    if (this.buffer.length > 0) {
      this.push(this.buffer.slice(0, size));
      this.buffer = this.buffer.slice(size);
      console.log('pushed', size, 'bytes')
    }

    while (this.buffer.length === 0) {
      const playbackResult = this.iter.play_next();
      this.buffer = Buffer.from(playbackResult.samples);
      if (playbackResult.is_done) return this.push(null);
    }

    console.log("about to push buffer of size", this.buffer.length);
    this.push(this.buffer.slice(0, size));
    this.buffer = this.buffer.slice(size);
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
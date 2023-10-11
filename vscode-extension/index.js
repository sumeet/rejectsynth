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
      let samples = reject.samples(editor.document.getText());

      resetSpeaker();
      const buffer = Buffer.from(samples.buffer);
      const bufStreamer = new BufStreamer(buffer);

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

// let interval = setInterval(() => {
//   let editor = vscode.window.activeTextEditor;
//   if (editor) {
//     let position = editor.selection.active;
//     let newPosition = position.with(position.line, position.character + 1);
//     let newSelection = new vscode.Selection(newPosition, newPosition);
//     editor.selection = newSelection;
//   }
// }, 100);
//
// context.subscriptions.push({
//   dispose: () => clearInterval(interval)
// });

//////////////////////////////////////////////////
// song playing stuff, which we'll do later
//////////////////////////////////////////////////


const { Readable } = require('stream');

class BufStreamer extends Readable {
  constructor(buffer, options) {
    super(options);
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
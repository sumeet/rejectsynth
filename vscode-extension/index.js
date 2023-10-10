const vscode = require('vscode');
const Speaker = require('speaker');

const reject = require('../build/wasm/rejectsynth.js');

const TOKEN_TYPES = [
    'keyword.other.rejectsynth',
]

const TOKEN_TYPE_INIDICES = TOKEN_TYPES.reduce((acc, type, index) => {
    acc[type] = index;
    return acc;
}, {});

class MySemanticTokensProvider {
    async provideDocumentSemanticTokens(doc) {
        console.log(doc.getText());

        const builder = new vscode.SemanticTokensBuilder();
        builder.push(0, 0, 3, TOKEN_TYPE_INIDICES['keyword.other.rejectsynth']);
        return builder.build();
    }
}

function activate(context) {
    let interval = setInterval(() => {
        let editor = vscode.window.activeTextEditor;
        if (editor) {
            let position = editor.selection.active;
            let newPosition = position.with(position.line, position.character + 1);
            let newSelection = new vscode.Selection(newPosition, newPosition);
            editor.selection = newSelection;
        }
    }, 100);

    context.subscriptions.push({
        dispose: () => clearInterval(interval)
    });

    context.subscriptions.push(vscode.languages.registerDocumentSemanticTokensProvider(
        { language: 'rejectsynth' },
        new MySemanticTokensProvider(),
        new vscode.SemanticTokensLegend(["keyword"])
    ));

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

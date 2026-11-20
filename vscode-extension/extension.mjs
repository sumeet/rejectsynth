import { TOKEN_TYPES, TOKEN_INDEX, syntax, syntaxRange, songFrames, previewFrames } from './engine.mjs';

// VS Code event handlers and providers shared by desktop and the embedded editor.
// Only the audio transport and optional page status callbacks differ by host.
export function activateRejectsynth(context, {
  vscode, reject, audio, onError = () => {}, onDiagnostic = () => {}, onPlaying = () => {}, showStatusBar = true,
}) {
  let autoPreview = true;
  let previewCommandSelections = false;
  let suppressSelectionUntil = 0;
  let request = 0;
  let playingEditor;
  let stopItem;
  let isPlaying = false;
  const lensesChanged = new vscode.EventEmitter();
  function setPlaying(value) {
    if (isPlaying !== value) {
      isPlaying = value;
      lensesChanged.fire();
    }
    onPlaying(value);
  }
  const decoration = vscode.window.createTextEditorDecorationType({
    backgroundColor: 'rgba(110, 220, 110, 0.35)',
  });
  const isSong = doc => doc?.languageId === 'rejectsynth';
  const active = () => {
    const editor = vscode.window.activeTextEditor;
    return isSong(editor?.document) ? editor : undefined;
  };

  function clear() {
    if (playingEditor) playingEditor.setDecorations(decoration, []);
    playingEditor = undefined;
    stopItem?.dispose();
    stopItem = undefined;
    setPlaying(false);
  }
  function stop() {
    request++;
    audio.stop();
    clear();
  }
  function fail(error, automatic) {
    const message = String(error?.message ?? error);
    onDiagnostic(message);
    if (!automatic) onError(message);
  }

  async function play({ selection = false, automatic = false, previewPosition } = {}) {
    const editor = active();
    if (!editor || (automatic && (!autoPreview || !audio.enabled))) return;
    stop();
    const ownRequest = request;
    const text = editor.document.getText();
    const version = editor.document.version;
    const start = selection ? editor.document.offsetAt(editor.selection.start) : undefined;
    const end = selection ? editor.document.offsetAt(editor.selection.end) : undefined;
    try {
      await audio.unlock();
      if (ownRequest !== request || editor.document.version !== version) return;
      const frames = previewPosition === undefined
        ? songFrames(reject, text, start, end)
        : previewFrames(reject, text, previewPosition);
      onDiagnostic('');
      playingEditor = editor;
      if (showStatusBar) {
        stopItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
        stopItem.text = '$(stop) Stop';
        stopItem.command = 'rejectsynth.stopPlaying';
        stopItem.show();
      }
      setPlaying(true);
      audio.play(frames, {
        onFrame: nodes => {
          if (ownRequest !== request || editor.document.version !== version) return;
          editor.setDecorations(decoration, nodes.map(node => syntaxRange(editor.document, node, vscode)));
        },
        onEnd: () => { if (ownRequest === request) clear(); },
        onError: error => { if (ownRequest === request) { clear(); fail(error, automatic); } },
      });
    } catch (error) {
      if (ownRequest === request) { stop(); fail(error, automatic); }
    }
  }

  context.subscriptions.push(decoration, lensesChanged, { dispose: () => { stop(); audio.dispose(); } });
  context.subscriptions.push(vscode.languages.registerDocumentSemanticTokensProvider(
    { language: 'rejectsynth' },
    { provideDocumentSemanticTokens(doc) {
      const builder = new vscode.SemanticTokensBuilder();
      try {
        for (const node of syntax(reject, doc.getText())) {
          const range = syntaxRange(doc, node, vscode);
          builder.push(range.start.line, range.start.character,
            range.end.character - range.start.character, TOKEN_INDEX[node.node_type] ?? 0);
        }
        onDiagnostic('');
      } catch (error) { onDiagnostic(String(error?.message ?? error)); }
      return builder.build();
    } },
    new vscode.SemanticTokensLegend(TOKEN_TYPES),
  ));
  context.subscriptions.push(vscode.languages.registerCodeLensProvider(
    { language: 'rejectsynth' },
    { onDidChangeCodeLenses: lensesChanged.event,
      provideCodeLenses: () => [new vscode.CodeLens(new vscode.Range(0, 0, 0, 0), {
      title: isPlaying ? 'Stop' : 'Play', command: 'rejectsynth.togglePlayback', arguments: [],
    })] },
  ));
  context.subscriptions.push(
    vscode.commands.registerCommand('rejectsynth.playWholeFile', () => play()),
    vscode.commands.registerCommand('rejectsynth.togglePlayback', () => isPlaying ? stop() : play()),
    vscode.commands.registerCommand('rejectsynth.playSelection', () => play({ selection: true })),
    vscode.commands.registerCommand('rejectsynth.stopPlaying', stop),
    vscode.window.onDidChangeTextEditorSelection(event => {
      if (!isSong(event.textEditor.document) || Date.now() < suppressSelectionUntil
          || (event.kind === vscode.TextEditorSelectionChangeKind.Command && !previewCommandSelections)) return;
      void play({ selection: true, automatic: true });
    }),
    vscode.workspace.onDidChangeTextDocument(event => {
      if (!isSong(event.document) || event.contentChanges.length === 0) return;
      stop();
      suppressSelectionUntil = Date.now() + 250;
      if (event.contentChanges.length !== 1 || event.document !== active()?.document) return;
      const [change] = event.contentChanges;
      if (!change.text.trim()) return;
      void play({ automatic: true, previewPosition: change.rangeOffset });
    }),
    vscode.window.onDidChangeActiveTextEditor(() => stop()),
  );
  return {
    stop,
    setAutoPreview(value) { autoPreview = value; },
    setCommandSelectionPreview(value) { previewCommandSelections = value; },
  };
}

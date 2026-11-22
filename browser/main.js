import 'vscode/localExtensionHost';
import { initialize } from '@codingame/monaco-vscode-api';
import { registerExtension, ExtensionHostKind } from '@codingame/monaco-vscode-api/extensions';
import getConfigurationServiceOverride, { updateUserConfiguration } from '@codingame/monaco-vscode-configuration-service-override';
import getEditorServiceOverride from '@codingame/monaco-vscode-editor-service-override';
import getExtensionsServiceOverride from '@codingame/monaco-vscode-extensions-service-override';
import getLanguagesServiceOverride from '@codingame/monaco-vscode-languages-service-override';
import getModelServiceOverride from '@codingame/monaco-vscode-model-service-override';
import * as monaco from 'monaco-editor';
import EditorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import init, * as reject from './generated/rejectsynth.js';
import wasmUrl from './generated/rejectsynth_bg.wasm?url';
import manifest from '../vscode-extension/package.json';
import example from '../songs/kalm.rej?raw';
import { activateRejectsynth } from '../vscode-extension/extension.mjs';
import { BrowserAudio } from './audio.mjs';

window.MonacoEnvironment = { getWorker: () => new EditorWorker() };

// Register the existing extension's language and commands. Its shared activation
// function runs in the local browser extension host with the real VS Code API.
const registration = registerExtension({
  name: manifest.name, publisher: 'sumeet', version: manifest.version,
  engines: manifest.engines, contributes: manifest.contributes,
}, ExtensionHostKind.LocalProcess);

let editor;
await initialize({
  ...getConfigurationServiceOverride(),
  ...getLanguagesServiceOverride(),
  ...getModelServiceOverride(),
  ...getEditorServiceOverride(async () => editor),
  ...getExtensionsServiceOverride(),
});
await updateUserConfiguration(JSON.stringify({
  'editor.semanticHighlighting.enabled': true,
  'editor.wordBasedSuggestions': 'off',
  'editor.minimap.enabled': false,
  'editor.lightbulb.enabled': 'off',
  'files.autoSave': 'off',
}));
await init({ module_or_path: wasmUrl });
const vscode = await registration.getApi();
const audio = new BrowserAudio();
const context = { subscriptions: [] };
const status = document.querySelector('#status');
const error = document.querySelector('#error');
const controls = activateRejectsynth(context, {
  vscode, reject, audio, showStatusBar: false,
  onDiagnostic: message => { error.textContent = message; },
  onPlaying: playing => {
    status.textContent = playing ? 'Playing' : 'Ready';
  },
});

monaco.editor.defineTheme('rejectsynth', {
  base: 'vs', inherit: true,
  rules: [
    { token: 'keyword', foreground: '67569A' },
    { token: 'number', foreground: '176B75' },
    { token: 'operator', foreground: '87580B' },
    { token: 'parameter', foreground: '9C4C55' },
  ],
  colors: {
    'editor.background': '#FAFCFF',
    'editorLineNumber.foreground': '#68778B',
    'editor.lineHighlightBackground': '#EAF0F9',
    'editor.selectionBackground': '#CBDCF5',
    'editorCodeLens.foreground': '#525F73',
  },
});
const modelReference = await monaco.editor.createModelReference(monaco.Uri.file('/kalm.rej'), example);
monaco.editor.setModelLanguage(modelReference.object.textEditorModel, 'rejectsynth');
editor = monaco.editor.create(document.querySelector('#editor'), {
  model: modelReference.object.textEditorModel,
  theme: 'rejectsynth',
  fontSize: 14, lineHeight: 22, fontFamily: '"DejaVu Sans Mono", monospace',
  fontLigatures: false,
  minimap: { enabled: false }, scrollBeyondLastLine: false,
  scrollbar: { verticalScrollbarSize: 8, verticalSliderSize: 6, horizontalScrollbarSize: 8, horizontalSliderSize: 6 },
  folding: false, glyphMargin: false, lineNumbers: 'off',
  lineDecorationsWidth: 12,
  padding: { top: 10, bottom: 10 }, overviewRulerLanes: 0,
  codeLens: true, codeLensFontSize: 14,
  renderLineHighlight: 'none', wordWrap: 'on', contextmenu: true,
  ariaLabel: 'rejectsynth song editor',
});

// Unlock from interaction with the editor itself, including typing/selection.
// Merely scrolling the portfolio past the iframe never starts audio.
const enableAudio = () => { void audio.unlock().catch(cause => { error.textContent = String(cause?.message ?? cause); }); };
const editorElement = document.querySelector('#editor');
editorElement.addEventListener('pointerdown', enableAudio, { capture: true });
editorElement.addEventListener('keydown', enableAudio, { capture: true });

let vimMode;
const vimButton = document.querySelector('#vim-toggle');
const vimStatus = document.querySelector('#vim-status');
vimButton.addEventListener('click', async () => {
  vimButton.disabled = true;
  try {
    if (vimMode) {
      vimMode.dispose();
      vimMode = undefined;
      vimStatus.hidden = true;
    } else {
      const { initVimMode } = await import('monaco-vim');
      vimStatus.hidden = false;
      vimMode = initVimMode(editor, vimStatus);
    }
    const enabled = Boolean(vimMode);
    controls.setCommandSelectionPreview(enabled);
    vimButton.setAttribute('aria-pressed', String(enabled));
    vimButton.title = enabled ? 'Disable Vim keybindings' : 'Enable Vim keybindings';
    vimButton.setAttribute('aria-label', vimButton.title);
    editor.focus();
  } catch (cause) { error.textContent = String(cause?.message ?? cause); }
  finally { vimButton.disabled = false; }
});

// Fit the iframe to the music, and notify the portfolio of wrapping/errors.
let layoutPending = false;
function resize() {
  if (layoutPending) return;
  layoutPending = true;
  requestAnimationFrame(() => {
    layoutPending = false;
    const height = Math.max(340, Math.min(640, Math.ceil(editor.getContentHeight())));
    if (editorElement.style.height !== `${height}px`) {
      editorElement.style.height = `${height}px`;
    }
    // ResizeObserver reports fractional CSS pixels at some browser zoom levels.
    // Monaco truncates them, creating a phantom 1px overflow and a full-height
    // scrollbar. Our resize observer supplies the element's integer dimensions.
    editor.layout({ width: editorElement.clientWidth, height: editorElement.clientHeight });
    if (window.parent !== window) window.parent.postMessage({
      type: 'rejectsynth:resize', height: document.querySelector('main').getBoundingClientRect().height,
    }, window.location.origin);
  });
}
editor.onDidContentSizeChange(resize);
new ResizeObserver(resize).observe(document.querySelector('main'));
window.addEventListener('message', event => {
  if (event.origin === location.origin && event.source === window.parent && event.data?.type === 'rejectsynth:stop') controls.stop();
});
document.addEventListener('visibilitychange', () => { if (document.hidden) controls.stop(); });
window.addEventListener('pagehide', () => controls.stop());
status.textContent = 'Ready';
vimButton.disabled = false;
document.querySelector('#loading').hidden = true;
resize();

if (import.meta.hot) import.meta.hot.dispose(() => {
  vimMode?.dispose();
  for (const item of context.subscriptions.reverse()) item.dispose();
  editor.dispose();
  modelReference.dispose();
  void registration.dispose();
});

# rejectsynth browser editor

The VS Code extension embedded in Monaco through `@codingame/monaco-vscode-api`.
Desktop and browser entry points call the same `vscode-extension/extension.mjs`;
syntax analysis and synthesis come from the existing Rust engine. The sample is
imported from `songs/kalm.rej`.

## Run

Requires Node 22.12+ (tested with 24.6), the repository's nightly Rust toolchain,
the `wasm32-unknown-unknown` target, and `wasm-bindgen-cli` **0.2.100**, matching
`rejectsynth/Cargo.toml`.

```sh
cd ~/rejectsynth/browser
npm ci
npm run wasm
npm run build
npm run preview
```

Open <http://127.0.0.1:5174/>. For JS/CSS development, use `npm run dev` instead
of build/preview. Rerun `npm run wasm` after Rust edits. Desktop WASM bindings are
still generated separately with `./bindgen.sh` from the repository root.

## Interaction

The inline **Play** CodeLens switches to **Stop** during playback. Click/select
notes or type to audition them; browser audio is unlocked by those interactions.
Audio gain is fixed at 0.5. There is no toolbar, volume slider, or preview checkbox.
The small Vim logo toggles `monaco-vim` keybindings (off by default); its mode and
command input appear alongside the icon when enabled. Vim selection changes also
use the extension's selection playback. Refresh restores the original sample.

Changes stay in the in-memory document. Incomplete/invalid edits display a parser
error; correcting the input restores playback without a page reload.

## Portfolio integration

`dist/` is a static application suitable for a same-origin iframe. It sends
`rejectsynth:resize` messages to its parent and accepts `rejectsynth:stop` from
that same-origin parent. It never focuses itself on load. Stop is also called
when the browser tab is hidden. The portfolio stops it when scrolled offscreen.

The sumeet.me repository's `make rejectsynth` builds this project and exports
`dist/`, build provenance, and dependency notices into its checked-in assets.
Only that static output is copied; this repository remains the source of truth
for the editor. Neither Node nor Rust is needed on the portfolio's Dokku build.

## Porting changes

- Shared VS Code providers, commands, and selection/edit event handlers, with
  host-specific audio transports (`native-audio.mjs` and `browser/audio.mjs`).
- Web Audio uses scheduled audio time for highlights, replacing the old fixed
  500 ms timer. Stop cancels both audio and pending highlights.
- Replaced the original undefined `ms` guard and missing sample-count argument
  with edit-event suppression and cancellable playback. Replacements and
  multiple changes no longer throw; handlers only process rejectsynth files.
- Released wasm-bindgen objects after extracting their data and converted Rust
  byte offsets to editor UTF-16 positions.
- Pinned wasm-bindgen to 0.2.100 for a reproducible build with the matching CLI.
- Parse errors return JS errors instead of panicking. Zero tempo, zero scale
  degree and zero duration denominator are rejected before playback. Notation
  and synthesis algorithms are otherwise unchanged.
- Enabled the document-model resolver service, fixing “Model not found” at boot.

## Verification

`npm test` checks the actual compiled WASM song, selection context, note previews,
invalid-edit recovery, and browser scheduling/cancellation with a simulated clock.
Helium/CDP checks cover visible syntax colors, editor interactions, Play/Stop,
selection/typing playback, invalid-input recovery, and Vim mode. A temporary
Web Audio analyser verified a nonzero signal while playback was running.
Native VS Code playback and mobile behavior have not been separately verified.

Monaco plus the compatibility layer is around 2.1 MB gzip; rejectsynth's WASM is
about 112 KB uncompressed. The portfolio iframe is lazy-loaded, and Vim loads on
request. Vite reports an optional Node filesystem import in the upstream diff
library; its browser branch uses fetch.

Vim logo: https://github.com/vim/vim/blob/master/runtime/vimlogo.svg (license in
`VIM-LICENSE.txt`). `monaco-vim` and its CodeMirror code carry MIT notices.

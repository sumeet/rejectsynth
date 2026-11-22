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

The inline **▸ Play** CodeLens switches to **■ Stop** during playback. Click/select
notes or type to audition them; browser audio is unlocked by those interactions.
Audio gain is fixed at 0.5. There is no toolbar, volume slider, or preview checkbox.
The small Vim logo toggles `monaco-vim` keybindings (off by default); its mode and
command input appear alongside the icon when enabled. Vim selection changes also
use the extension's selection playback. Refresh restores the original sample.

The editor has no line-number or marker gutter, with 12px left padding. Code and
legend examples use the original DejaVu Sans Mono / monospace stack. Editor
ligatures are off so repeated music notation stays visually distinct. Font and
palette comparisons live separately in `~/rejectsynth-comparison-2026-11-21/`.
The selected combination is DejaVu Sans Mono with the original palette
(21 November 2026); the comparison's alternative fonts and colors stay out
of the production build.

Changes stay in the in-memory document. Incomplete/invalid edits get native
VS Code error diagnostics at the offending input, including unfinished tokens
and validation failures such as zero tempo. The shared parser exports UTF-8 byte
ranges, which the extension converts to UTF-16 editor positions. Correcting the
input clears the diagnostics and restores playback without a page reload. The
parser currently reports its first error; it does not recover to find multiple
errors in a single pass. Rebuild the WASM bindings for both browser and desktop
when updating the shared parser API.

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
  degree and zero duration denominator are rejected before playback.
- Enabled the document-model resolver service, fixing “Model not found” at boot.

The notation legend uses spaced grey text below the editor. Repeated `~` now
halves/doubles duration at each step (2, 4, 8, 16…), with overflow reported as a
parse error. A dot adds half the note's duration. `V>:` and `V>>:` raise
chord inversions, complementing `<V:` and `<<V:`. The old `>V:` prefix form
is still accepted for existing songs; a bare `>` remains the playback-start
marker. `VIII:` and `viii:` select the octave chord roots. These changes use the
shared text parser and synthesis engine used by the browser and VS Code.
Plain `7` adds a minor seventh (10 semitones above the chord root): uppercase
chords become dominant sevenths, lowercase chords become minor sevenths.
This corrects the former major-seventh sound of uppercase `7` chords.
Plain `9` includes that seventh and a major ninth (14 semitones above the root).
Both `V9:` and `v9:` support the same inversions, selection playback and previews
as other chords. The legend groups these as "7th or 9th chords".

## Verification

`npm test` checks the actual compiled WASM song, selection context, note previews,
invalid-edit recovery, and browser scheduling/cancellation with a simulated clock.
It also checks note durations and the new chord syntax. Run
`cargo test --manifest-path ../rejectsynth/Cargo.toml --lib --locked` from this
directory for parser, chord-voicing, and playback-marker regression tests.
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

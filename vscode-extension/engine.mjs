// Copy values out of wasm-bindgen objects, then release their Rust allocations.
export const SAMPLE_RATE = 44100;
export const TOKEN_TYPES = ['keyword', 'number', 'operator', 'parameter'];
export const TOKEN_INDEX = {
  SetKey: 0, SetBPM: 0, SetScale: 0, PlayNote: 1, SkipToNote: 2, SetHarmony: 3,
};

export function takeSyntax(nodes) {
  return nodes.map(node => {
    try {
      return { line_no: node.line_no, col_no: node.col_no, len: node.len, node_type: node.node_type };
    } finally {
      node.free();
    }
  });
}

export function syntax(reject, text) {
  return takeSyntax(reject.syntax(text));
}

// Rust positions are UTF-8 byte offsets; VS Code positions count UTF-16 units.
const encoder = new TextEncoder();
const decoder = new TextDecoder();
export const byteOffset = (text, offset) => encoder.encode(text.slice(0, offset)).length;
export function syntaxRange(doc, node, vscode) {
  const line = encoder.encode(doc.lineAt(node.line_no).text);
  const col = decoder.decode(line.subarray(0, node.col_no)).length;
  const start = new vscode.Position(node.line_no, col);
  const tail = doc.getText().slice(doc.offsetAt(start));
  const length = decoder.decode(encoder.encode(tail).subarray(0, node.len)).length;
  return new vscode.Range(start, doc.positionAt(doc.offsetAt(start) + length));
}

export function songFrames(reject, text, start, end) {
  const iter = reject.WasmSongIterator.from_song_text(text,
    start === undefined ? undefined : byteOffset(text, start),
    end === undefined ? undefined : byteOffset(text, end));
  let disposed = false;
  return {
    next() {
      if (disposed || iter.is_done()) return null;
      const result = iter.play_next();
      try {
        return { samples: result.samples, syntaxes: takeSyntax(result.on_syntaxes) };
      } finally {
        result.free();
      }
    },
    dispose() { if (!disposed) { disposed = true; iter.free(); } },
  };
}

export function previewFrames(reject, text, position) {
  const samples = reject.playback_for_note_input(text, byteOffset(text, position));
  let consumed = false;
  return {
    next() {
      if (consumed || !samples.length) return null;
      consumed = true;
      return { samples, syntaxes: [] };
    },
    dispose() { consumed = true; },
  };
}

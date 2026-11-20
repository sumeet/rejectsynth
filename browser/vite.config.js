import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  base: './',
  build: { target: 'es2022' },
  worker: { format: 'es' },
  // This package's browser export is UMD; use its ESM build with our Monaco alias.
  resolve: { alias: { 'monaco-vim': fileURLToPath(new URL('./node_modules/monaco-vim/dist/index.mjs', import.meta.url)) } },
  optimizeDeps: { include: ['monaco-editor', 'vscode/localExtensionHost'] },
});

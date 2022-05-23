import { resolve } from 'path';
import { defineConfig } from 'vite';
import wasmPack from 'vite-plugin-wasm-pack';

module.exports = defineConfig({
  plugins: [wasmPack('../crate')],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/webview/webview_main.ts'),
      formats: ['es'],
      name: 'webview',
      fileName: `webview_main`,
    },
  },
});

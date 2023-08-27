import { resolve } from "path";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";
import basicSsl from "@vitejs/plugin-basic-ssl";

export default defineConfig({
  plugins: [wasmPack("../core"), basicSsl()],
  resolve: { alias: { "~": resolve(__dirname, "./src") } },
  build: {
    watch: {
      include: ["../core/pkg/**/*"],
    },
  },
  server: {
    // Enable Cross-Origin-Embedder-Policy and Cross-Origin-Opener-Policy
    // headers for SharedArrayBuffer support. You also need HTTPS support via
    // the plugin-basic-ssl vite plugin.
    // See
    // https://github.com/Ciantic/rust-shared-wasm-experiments/tree/master
    // https://github.com/wasm-rs/shared-channel/
    headers: {
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Cross-Origin-Opener-Policy": "same-origin",
    },
  },
});

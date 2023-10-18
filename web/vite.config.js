import { resolve } from "path";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  plugins: [wasmPack("../core"), wasmPack("../core-plugin")],
  resolve: { alias: { "~": resolve(__dirname, "./src") } },
});

import { resolve } from "path";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  plugins: [wasmPack("../crate")],
  resolve: { alias: { "~": resolve(__dirname, "./src") } },
  build: {
    watch: {
      include: ["../crate/pkg/**/*"],
    },
  },
});

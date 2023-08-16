import { resolve } from "path";
import { defineConfig } from "vite";
import wasmPack from "vite-plugin-wasm-pack";

const common = {
  plugins: [wasmPack("../core")],
  resolve: { alias: { "~": resolve(__dirname, "./src") } },
};

export default defineConfig(({ command, mode, ssrBuild }) => {
  if (command === "serve") {
    return {
      ...common,
      build: {
        watch: {
          include: ["../core/pkg"],
        },
      },
    };
  } else {
    return {
      ...common,
      build: {
        lib: {
          entry: resolve(__dirname, "src/lib.ts"),
          name: "logic-paint-sys-js",
          fileName: "lp_sys_js",
        },
      },
    };
  }
});

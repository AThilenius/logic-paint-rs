/// Entry point for the dev site, used to develop/test the library.
import initCore, { Host, register_plugin } from "core";
import initPlugin, { get_plugin_impl } from "core-plugin";

async function main() {
  await initCore();
  await initPlugin();

  const host = Host.from_parent_element(
    document.getElementById("canvas-container")!
  );

  const plugin = get_plugin_impl();

  // Time how long this call takes.
  const start = performance.now();
  register_plugin(plugin);
  const end = performance.now();

  const frame = () => {
    host.frame();
    requestAnimationFrame(frame);
  };

  requestAnimationFrame(frame);
}

// Load wasm from binary path using Vite.

// async function main() {
//   let instance = await init();
//   const hello = instance.exports.hello as CallableFunction;
//   const result = hello();
//   console.log("Got:", result);
// }

main();

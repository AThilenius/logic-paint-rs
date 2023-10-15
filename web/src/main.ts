/// Entry point for the dev site, used to develop/test the library.
import init, { spawn } from "crate";

async function main() {
  await init();
  spawn(document.getElementById("canvas-container")!);
}

main();

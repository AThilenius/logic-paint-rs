/// Entry point for the dev site, used to develop/test the library.
import init, { Host } from "core";

async function main() {
  await init();
  const host = Host.from_parent_element(
    document.getElementById("canvas-container")!
  );

  const frame = () => {
    host.frame();
    requestAnimationFrame(frame);
  };

  requestAnimationFrame(frame);
}

main();

/// Entry point for the dev site, used to develop/test the library.
import init, { Host } from "core";

async function main() {
  await init();

  const container = document.getElementById("canvas-container")!;
  const host = Host.from_parent_element(container);

  container.onmousemove = (e) => {
    host.mouse_move(e.offsetX, e.offsetY);
  };

  const frame = () => {
    host.frame();
    requestAnimationFrame(frame);
  };

  requestAnimationFrame(frame);
}

main();

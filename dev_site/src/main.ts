import init, { LogicPaint } from "crate";

async function main() {
  await init();

  const logicPaint = new LogicPaint(
    document.getElementById("root") as HTMLCanvasElement,
    () => {},
    () => {},
    () => {},
    () => {}
  );
}

void main();

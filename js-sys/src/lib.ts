import init, { LogicPaint } from "core";

export async function doTheThings() {
  await init();
  console.log("Hello from lib!", LogicPaint.name);
}

// async function main() {
//   await init();

//   const logicPaint = new LogicPaint(
//     document.getElementById("root") as HTMLCanvasElement,
//     () => {},
//     () => {},
//     () => {},
//     () => {}
//   );
// }

// void main();

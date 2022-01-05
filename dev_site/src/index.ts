import { LogicPaint } from '../../pkg/index.js';

const logicPaint = new LogicPaint(
  document.getElementById('wasm-canvas') as HTMLCanvasElement
);

// DEV
(window as any).logicPaint = logicPaint;

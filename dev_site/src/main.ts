import { LogicPaint } from '../../pkg/index.js';

const logicPaint = new LogicPaint(
  document.getElementById('root') as HTMLCanvasElement
);

// DEV
(window as any).logicPaint = logicPaint;

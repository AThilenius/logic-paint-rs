import init, { LogicPaint } from 'crate';

async function main() {
  await init();

  const logicPaint = new LogicPaint(
    document.getElementById('root') as HTMLCanvasElement
  );

  // DEV
  (window as any).logicPaint = logicPaint;
}

void main();

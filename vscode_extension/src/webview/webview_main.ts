// Get a reference to the VS Code webview api.
// We use this API to post messages back to our extension.
import init, { LogicPaint } from 'crate';

async function main() {
  // @ts-ignore
  const vscode = acquireVsCodeApi();

  // The Wasm URI needs to be provided by VSCode itself. So we assign the URI
  // in a script to the window object.
  const wasmUrl = (window as any).wasmUri;
  await init(wasmUrl);

  const onLpUpdatedBlueprint = (blueprintString: string) => {
    vscode.postMessage({
      type: 'SET_BLUEPRINT_STRING',
      blueprintString,
    });
  };

  const logicPaint = new LogicPaint(
    document.getElementById('root') as HTMLCanvasElement,
    onLpUpdatedBlueprint
  );

  // Handle messages sent from the extension to the webview
  window.addEventListener('message', (event) => {
    const message = event.data;
    switch (message.type) {
      case 'SET_BLUEPRINT_STRING': {
        let blueprintString: string = message.blueprintString;

        if (!blueprintString.trim().length) {
          blueprintString = '{}';
        }

        const err =
          logicPaint.set_partial_blueprint_from_json_string(blueprintString);

        if (err) {
          console.log(err);
        }

        // Then persist state information.
        // This state is returned in the call to `vscode.getState` below when a webview is reloaded.
        // vscode.setState({ blueprintString });

        return;
      }
      case 'TRIGGER_COPY': {
        vscode.postMessage({ type: 'SET_CLIPBOARD', value: logicPaint.copy() });
        return;
      }
      case 'PASTE': {
        const value = message.value;
        const err = logicPaint.paste(value);
        if (err) {
          console.error(err);
        }
        return;
      }
    }
  });

  // const state = vscode.getState();

  // if (state?.blueprintString) {
  //   logicPaint.set_partial_blueprint_from_json_string(state.blueprintString);
  // }

  // DEV
  (window as any).logicPaint = logicPaint;

  vscode.postMessage({
    type: 'READY',
  });
}

void main();

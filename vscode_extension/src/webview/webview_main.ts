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

  const onLpUpdatedEditorState = (editorStateString: string) => {
    vscode.setState({ editorStateString });
  };

  const onLpRequestedClipboard = () => {
    vscode.postMessage({
      type: 'REQUEST_CLIPBOARD',
    });
  };

  const onLpSetClipboard = (content: string) => {
    vscode.postMessage({
      type: 'SET_CLIPBOARD',
      content,
    });
  };

  const logicPaint = new LogicPaint(
    document.getElementById('root') as HTMLCanvasElement,
    onLpUpdatedBlueprint,
    onLpUpdatedEditorState,
    onLpRequestedClipboard,
    onLpSetClipboard
  );

  // Handle messages sent from the extension to the webview
  window.addEventListener('message', (event) => {
    const message = event.data;
    switch (message.type) {
      case 'SET_BLUEPRINT_STRING': {
        let blueprintString: string = message.blueprintString;

        if (!blueprintString.trim().length) {
          blueprintString = '{"chunks":[], "modules":[]}';
        }

        const err = logicPaint.set_blueprint_from_json_string(blueprintString);

        if (err) {
          console.log(err);
        }

        return;
      }
      case 'RETURN_REQUEST_CLIPBOARD': {
        const err = logicPaint.set_clipboard(message.content);
        if (err) {
          console.error(err);
        }
        return;
      }
    }
  });

  const state = vscode.getState();
  if (state?.editorStateString) {
    logicPaint.set_editor_state_from_json_string(state.editorStateString);
  }

  vscode.postMessage({
    type: 'READY',
  });
}

void main();

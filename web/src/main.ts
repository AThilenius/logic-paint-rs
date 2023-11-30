// Get a reference to the VS Code webview api.
// We use this API to post messages back to our extension.
import init, { LogicPaint } from "core";

async function main() {
  await init();

  const onLpUpdatedBlueprint = (blueprintString: string) => {
    // Todo
  };

  const onLpUpdatedEditorState = (editorStateString: string) => {
    // Todo
  };

  const onLpRequestedClipboard = () => {
    // Todo
  };

  const onLpSetClipboard = (content: string) => {
    // Todo
  };

  const logicPaint = new LogicPaint(
    document.getElementById("canvas-container") as HTMLCanvasElement,
    onLpUpdatedBlueprint,
    onLpUpdatedEditorState,
    onLpRequestedClipboard,
    onLpSetClipboard
  );

  // Handle messages sent from the extension to the webview
  window.addEventListener("message", (event) => {
    const message = event.data;
    switch (message.type) {
      case "SET_BLUEPRINT_STRING": {
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
      case "RETURN_REQUEST_CLIPBOARD": {
        const err = logicPaint.set_clipboard(message.content);
        if (err) {
          console.error(err);
        }
        return;
      }
    }
  });

  const state: any = null;
  if (state?.editorStateString) {
    logicPaint.set_editor_state_from_json_string(state.editorStateString);
  }
}

void main();

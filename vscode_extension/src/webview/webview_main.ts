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

// errorContainer.style.display = 'none';

/**
 * Render the document in the webview.
 */
// function updateContent(text: string) {
//   let json;
//   try {
//     if (!text) {
//       text = '{}';
//     }
//     json = JSON.parse(text);
//   } catch {
//     notesContainer.style.display = 'none';
//     errorContainer.innerText = 'Error: Document is not valid json';
//     errorContainer.style.display = '';
//     return;
//   }
//   notesContainer.style.display = '';
//   // errorContainer.style.display = 'none';

//   // Render the scratches
//   notesContainer.innerHTML = '';
//   for (const note of json.scratches || []) {
//     const element = document.createElement('div');
//     element.className = 'note';
//     notesContainer.appendChild(element);

//     const text = document.createElement('div');
//     text.className = 'text';
//     const textContent = document.createElement('span');
//     textContent.innerText = note.text;
//     text.appendChild(textContent);
//     element.appendChild(text);

//     const created = document.createElement('div');
//     created.className = 'created';
//     created.innerText = new Date(note.created).toUTCString();
//     element.appendChild(created);

//     const deleteButton = document.createElement('button');
//     deleteButton.className = 'delete-button';
//     deleteButton.addEventListener('click', () => {
//       vscode.postMessage({ type: 'delete', id: note.id });
//     });
//     element.appendChild(deleteButton);
//   }

//   notesContainer.appendChild(addButtonContainer);
// }

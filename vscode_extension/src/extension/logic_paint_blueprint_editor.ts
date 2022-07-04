import * as vscode from 'vscode';

export class LogicPaintBlueprintEditorProvider
  implements vscode.CustomTextEditorProvider
{
  private static readonly viewType = 'logicPaint.logicPaintBlueprint';

  constructor(private readonly context: vscode.ExtensionContext) {}

  /**
   * Called when our custom editor is opened.
   */
  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    webviewPanel.webview.options = { enableScripts: true };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    const lpModulesPath = vscode.Uri.joinPath(
      document.uri,
      '../lp-modules.json'
    );

    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
      (e) => {
        if (e.document.uri.toString() === document.uri.toString()) {
          webviewPanel.webview.postMessage({
            type: 'SET_BLUEPRINT_STRING',
            blueprintString: document.getText(),
          });
        }
      }
    );

    const savedDocumentSubscription = vscode.workspace.onDidSaveTextDocument(
      (doc) => {
        if (doc.uri.toString() === lpModulesPath.toString()) {
          webviewPanel.webview.postMessage({
            type: 'SET_BLUEPRINT_STRING',
            blueprintString: doc.getText(),
          });
        }
      }
    );

    // Receive message from the webview.
    const webviewMessageSubscription = webviewPanel.webview.onDidReceiveMessage(
      async (e) => {
        switch (e.type) {
          case 'READY': {
            // Try to load the lp-modules.json file. Ignore if it doesn't exist.
            try {
              await vscode.workspace.fs.stat(lpModulesPath);
              const doc = await vscode.workspace.openTextDocument(
                lpModulesPath
              );

              webviewPanel.webview.postMessage({
                type: 'SET_BLUEPRINT_STRING',
                blueprintString: doc.getText(),
              });
            } catch {
              // Ignore
            }

            // Also update chunks.
            webviewPanel.webview.postMessage({
              type: 'SET_BLUEPRINT_STRING',
              blueprintString: document.getText(),
            });
            return;
          }
          case 'SET_BLUEPRINT_STRING': {
            // Remove module info from the Blueprint.
            let jsonString = e.blueprintString;
            const blueprint = JSON.parse(jsonString);
            delete blueprint.modules;
            jsonString = JSON.stringify(blueprint, null, 2);

            // Then apply the edit.
            const edit = new vscode.WorkspaceEdit();
            edit.replace(
              document.uri,
              new vscode.Range(0, 0, document.lineCount, 0),
              jsonString
            );

            vscode.workspace.applyEdit(edit);
            return;
          }
        }
      }
    );

    // const doc = await vscode.workspace.openTextDocument(path);
    // const pos1 = new vscode.Position(3, 8);
    // const editor = await vscode.window.showTextDocument(doc, {
    //   viewColumn: vscode.ViewColumn.Beside,
    //   preview: true,
    // });
    // editor.selections = [new vscode.Selection(pos1, pos1)];
    // var range = new vscode.Range(pos1, pos1);
    // editor.revealRange(range);

    // setTimeout(async () => {
    //   const editor = await vscode.window.showTextDocument(doc, {
    //     viewColumn: vscode.ViewColumn.Beside,
    //     preview: true,
    //     selection: new vscode.Range(
    //       new vscode.Position(3, 1),
    //       new vscode.Position(3, 10)
    //     ),
    //   });
    //   vscode.window.showTextDocument(editor.document);
    //   editor.selections = [new vscode.Selection(pos1, pos1)];
    //   var range = new vscode.Range(pos1, pos1);
    //   editor.revealRange(range);
    // }, 5_000);

    // Make sure we get rid of the listener when our editor is closed.
    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
      savedDocumentSubscription.dispose();
      webviewMessageSubscription.dispose();
    });
  }

  /**
   * Get the static html used for the editor webviews.
   */
  private getHtmlForWebview(webview: vscode.Webview): string {
    // Local path to script and css for the webview
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.context.extensionUri,
        'dist',
        'webview_main.es.js'
      )
    );

    const styleMainUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.context.extensionUri,
        'media',
        'logic_paint_blueprint.css'
      )
    );

    const wasmUri = webview.asWebviewUri(
      vscode.Uri.joinPath(
        this.context.extensionUri,
        'dist',
        'assets',
        'crate_bg.wasm'
      )
    );

    // <link href="${styleResetUri}" rel="stylesheet" />
    // <link href="${styleVSCodeUri}" rel="stylesheet" />

    return /* html */ `
			<!DOCTYPE html>
			<html lang="en">
			<head>
				<meta charset="UTF-8">

				<!--
				Use a content security policy to only allow loading images from https or from our extension directory,
				and only allow scripts that have a specific nonce.
				-->
				<meta name="viewport" content="width=device-width, initial-scale=1.0">

				<link href="${styleMainUri}" rel="stylesheet" />
        <link href="${wasmUri}" rel="prefetch">

				<title>Logic Paint</title>
			</head>
			<body>
        <div id="root" class="sample-mount"></div>

        <script>
          window.wasmUri = "${wasmUri}";
        </script>

				<script src="${scriptUri}" type="module"></script>
			</body>
			</html>`;
  }

  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    const provider = new LogicPaintBlueprintEditorProvider(context);
    const providerRegistration = vscode.window.registerCustomEditorProvider(
      LogicPaintBlueprintEditorProvider.viewType,
      provider
    );
    return providerRegistration;
  }
}

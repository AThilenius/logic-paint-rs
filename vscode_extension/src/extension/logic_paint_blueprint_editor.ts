import {
  CancellationToken,
  CustomTextEditorProvider,
  Disposable,
  ExtensionContext,
  Range,
  RelativePattern,
  TextDocument,
  Uri,
  Webview,
  WebviewPanel,
  WorkspaceEdit,
  env,
  workspace,
  window,
} from 'vscode';
import path from 'path';

export class LogicPaintBlueprintEditorProvider
  implements CustomTextEditorProvider
{
  private static readonly viewType = 'logicPaint.logicPaintBlueprint';

  constructor(private readonly context: ExtensionContext) {}

  /**
   * Called when our custom editor is opened.
   */
  public async resolveCustomTextEditor(
    document: TextDocument,
    webviewPanel: WebviewPanel,
    _token: CancellationToken
  ): Promise<void> {
    const disposableArray: Disposable[] = [];

    webviewPanel.webview.options = { enableScripts: true };
    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    // Migrate from old format.
    let text = document.getText();
    const obj = JSON.parse(text);

    if (Array.isArray(obj.chunks)) {
      const chunks: { [key: string]: string } = {};
      for (const chunk of obj.chunks) {
        chunks[`${chunk.chunk_coord[0]}:${chunk.chunk_coord[1]}`] = chunk.cells;
      }

      obj.chunks = chunks;
      obj.modules = {};

      const edit = new WorkspaceEdit();
      text = JSON.stringify(obj, null, 2);
      edit.replace(document.uri, new Range(0, 0, document.lineCount, 0), text);

      workspace.applyEdit(edit);
    }

    disposableArray.push(
      workspace.onDidChangeTextDocument((e) => {
        if (e.document.uri.toString() === document.uri.toString()) {
          webviewPanel.webview.postMessage({
            type: 'SET_BLUEPRINT_STRING',
            blueprintString: document.getText(),
          });
        }
      })
    );

    // Receive message from the webview.
    disposableArray.push(
      webviewPanel.webview.onDidReceiveMessage(async (e) => {
        switch (e.type) {
          case 'READY': {
            // Update the Blueprint
            webviewPanel.webview.postMessage({
              type: 'SET_BLUEPRINT_STRING',
              blueprintString: document.getText(),
            });
            return;
          }
          case 'SET_BLUEPRINT_STRING': {
            const edit = new WorkspaceEdit();
            const json = e.blueprintString;
            edit.replace(
              document.uri,
              new Range(0, 0, document.lineCount, 0),
              json
            );

            workspace.applyEdit(edit);
            return;
          }
          case 'SET_CLIPBOARD': {
            void env.clipboard.writeText(e.content);
            return;
          }
          case 'REQUEST_CLIPBOARD': {
            (async () => {
              const content = await env.clipboard.readText();
              if (content) {
                webviewPanel.webview.postMessage({
                  type: 'RETURN_REQUEST_CLIPBOARD',
                  content,
                });
              }
            })();
            return;
          }
        }
      })
    );

    // Make sure we get rid of the listener when our editor is closed.
    webviewPanel.onDidDispose(() => {
      for (const disposable of disposableArray) {
        disposable.dispose();
      }
    });
  }

  /**
   * Get the static html used for the editor webviews.
   */
  private getHtmlForWebview(webview: Webview): string {
    // Local path to script and css for the webview
    const scriptUri = webview.asWebviewUri(
      Uri.joinPath(this.context.extensionUri, 'dist', 'webview_main.es.js')
    );

    const styleMainUri = webview.asWebviewUri(
      Uri.joinPath(
        this.context.extensionUri,
        'media',
        'logic_paint_blueprint.css'
      )
    );

    const wasmUri = webview.asWebviewUri(
      Uri.joinPath(this.context.extensionUri, 'dist', 'assets', 'crate_bg.wasm')
    );

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

  public static register(context: ExtensionContext): Disposable {
    const provider = new LogicPaintBlueprintEditorProvider(context);
    const providerRegistration = window.registerCustomEditorProvider(
      LogicPaintBlueprintEditorProvider.viewType,
      provider
    );
    return providerRegistration;
  }
}

import * as vscode from 'vscode';
import { getNonce } from './util';

export class LogicPaintBlueprintEditorProvider
  implements vscode.CustomTextEditorProvider
{
  public static register(context: vscode.ExtensionContext): vscode.Disposable {
    const provider = new LogicPaintBlueprintEditorProvider(context);
    const providerRegistration = vscode.window.registerCustomEditorProvider(
      LogicPaintBlueprintEditorProvider.viewType,
      provider
    );
    return providerRegistration;
  }

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

    function updateWebview() {
      webviewPanel.webview.postMessage({
        type: 'SET_SESSION_STRING',
        sessionString: document.getText(),
      });
    }

    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(
      (e) => {
        if (e.document.uri.toString() === document.uri.toString()) {
          updateWebview();
        }
      }
    );

    // Make sure we get rid of the listener when our editor is closed.
    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
    });

    // Receive message from the webview.
    webviewPanel.webview.onDidReceiveMessage((e) => {
      switch (e.type) {
        case 'READY': {
          updateWebview();
          return;
        }
        case 'SET_SESSION_STRING': {
          const edit = new vscode.WorkspaceEdit();
          edit.replace(
            document.uri,
            new vscode.Range(0, 0, document.lineCount, 0),
            e.sessionString
          );

          vscode.workspace.applyEdit(edit);
          return;
        }
      }
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

    const styleResetUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, 'media', 'reset.css')
    );

    const styleVSCodeUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, 'media', 'vscode.css')
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

    // Use a nonce to whitelist which scripts can be run
    const nonce = getNonce();

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

				<link href="${styleResetUri}" rel="stylesheet" />
				<link href="${styleVSCodeUri}" rel="stylesheet" />
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
}

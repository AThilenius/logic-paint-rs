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

    const documentName = path.basename(
      document.uri.path,
      path.extname(document.uri.path)
    );
    const documentDirectory = path.dirname(document.uri.path);
    const modulesFileName = `${documentName}.lpm.json`;
    const modulesFilePath = Uri.file(
      path.join(documentDirectory, modulesFileName)
    );

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

    const filesystemWatcher = workspace.createFileSystemWatcher(
      new RelativePattern(
        workspace.getWorkspaceFolder(Uri.joinPath(document.uri, '../'))!,
        `{${documentName}.lpbp,${modulesFileName}}`
      )
    );
    disposableArray.push(filesystemWatcher);

    const fsOnChange = async (uri: Uri) => {
      // Both the *.lpbp and *.lpm.json files will trigger a 'blueprint set' on
      // the viewport.
      if (
        uri.toString() === modulesFilePath.toString() ||
        uri.toString() === document.uri.toString()
      ) {
        const doc = await workspace.openTextDocument(uri);
        webviewPanel.webview.postMessage({
          type: 'SET_BLUEPRINT_STRING',
          blueprintString: doc.getText(),
        });
      }
    };

    filesystemWatcher.onDidChange(fsOnChange);
    filesystemWatcher.onDidCreate(fsOnChange);
    filesystemWatcher.onDidDelete(fsOnChange);

    // Receive message from the webview.
    disposableArray.push(
      webviewPanel.webview.onDidReceiveMessage(async (e) => {
        switch (e.type) {
          case 'READY': {
            // Try to load the lp-modules.json file. Ignore if it doesn't exist.
            try {
              await workspace.fs.stat(modulesFilePath);
              const doc = await workspace.openTextDocument(modulesFilePath);

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
            const edit = new WorkspaceEdit();
            edit.replace(
              document.uri,
              new Range(0, 0, document.lineCount, 0),
              jsonString
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

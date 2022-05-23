import * as vscode from 'vscode';
import { LogicPaintBlueprintEditorProvider } from './logic_paint_blueprint_editor';

export function activate(context: vscode.ExtensionContext) {
  // Register our custom editor providers
  context.subscriptions.push(
    LogicPaintBlueprintEditorProvider.register(context)
  );
}

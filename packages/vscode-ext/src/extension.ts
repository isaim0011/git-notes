import * as vscode from 'vscode';
import { GutterProvider } from './gutterProvider';
import { InlayHintsProvider } from './inlayProvider';
import { registerCommands } from './commands';

let autoSyncInterval: NodeJS.Timeout | undefined;

export function activate(context: vscode.ExtensionContext) {
    const gutterProvider = new GutterProvider();
    context.subscriptions.push(gutterProvider);

    const inlayProvider = new InlayHintsProvider(gutterProvider);
    context.subscriptions.push(
        vscode.languages.registerInlayHintsProvider({ scheme: 'file' }, inlayProvider)
    );

    registerCommands(context, gutterProvider);

    // Initial sync
    vscode.commands.executeCommand('git-notes.sync');

    // Refresh decorations when active editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor) {
                gutterProvider.refresh(editor);
            }
        })
    );

    // Refresh on save if configured
    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument(() => {
            const config = vscode.workspace.getConfiguration('git-notes');
            if (config.get<boolean>('autoSync', false)) {
                vscode.commands.executeCommand('git-notes.sync');
            }
        })
    );
    
    // Auto sync interval
    const config = vscode.workspace.getConfiguration('git-notes');
    if (config.get<boolean>('autoSync', false)) {
        autoSyncInterval = setInterval(() => {
            vscode.commands.executeCommand('git-notes.sync');
        }, 5 * 60 * 1000); // 5 minutes
    }
}

export function deactivate() {
    if (autoSyncInterval) {
        clearInterval(autoSyncInterval);
    }
}

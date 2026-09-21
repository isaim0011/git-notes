import * as vscode from 'vscode';
import { GutterProvider } from './gutterProvider';

export class InlayHintsProvider implements vscode.InlayHintsProvider {
    constructor(private gutterProvider: GutterProvider) {}

    provideInlayHints(document: vscode.TextDocument, range: vscode.Range, token: vscode.CancellationToken): vscode.ProviderResult<vscode.InlayHint[]> {
        const hints: vscode.InlayHint[] = [];
        const fsPath = document.uri.fsPath;
        
        for (let line = range.start.line; line <= range.end.line; line++) {
            const notes = this.gutterProvider.getNotesForLine(fsPath, line + 1);
            if (notes.length > 0) {
                const position = new vscode.Position(line, document.lineAt(line).text.length);
                const hint = new vscode.InlayHint(position, `[💬 ${notes.length}]`, vscode.InlayHintKind.Type);
                hint.paddingLeft = true;
                
                const command = {
                    title: "Show Note Thread",
                    command: "git-notes.showPanel",
                    arguments: [fsPath, line + 1]
                };
                
                const labelPart = new vscode.InlayHintLabelPart(`[💬 ${notes.length}]`);
                labelPart.command = command;
                hint.label = [labelPart];
                hint.tooltip = "Click to view thread";
                
                hints.push(hint);
            }
        }
        return hints;
    }
}

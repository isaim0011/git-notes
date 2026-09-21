import * as vscode from 'vscode';
import { Note } from './types';

export class NotePanel {
    public static currentPanel: NotePanel | undefined;
    private readonly _panel: vscode.WebviewPanel;
    private _disposables: vscode.Disposable[] = [];

    public static createOrShow(extensionUri: vscode.Uri, notes: Note[]) {
        const column = vscode.window.activeTextEditor ? vscode.window.activeTextEditor.viewColumn : undefined;

        if (NotePanel.currentPanel) {
            NotePanel.currentPanel._panel.reveal(column);
            NotePanel.currentPanel._update(notes);
            return;
        }

        const panel = vscode.window.createWebviewPanel(
            'gitNotesPanel',
            'Note Thread',
            column || vscode.ViewColumn.One,
            {
                enableScripts: true,
                localResourceRoots: [extensionUri]
            }
        );

        NotePanel.currentPanel = new NotePanel(panel, notes);
    }

    private constructor(panel: vscode.WebviewPanel, notes: Note[]) {
        this._panel = panel;
        this._update(notes);

        this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

        this._panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'reply':
                        vscode.commands.executeCommand('git-notes.add', message.text, message.parentId);
                        return;
                    case 'resolve':
                        vscode.window.showInformationMessage(`Resolve note ${message.noteId}`);
                        // Add actual resolving logic here if needed
                        return;
                }
            },
            null,
            this._disposables
        );
    }

    public dispose() {
        NotePanel.currentPanel = undefined;
        this._panel.dispose();
        while (this._disposables.length) {
            const x = this._disposables.pop();
            if (x) {
                x.dispose();
            }
        }
    }

    private _update(notes: Note[]) {
        this._panel.title = `Note Thread`;
        this._panel.webview.html = this._getHtmlForWebview(notes);
    }

    private _getHtmlForWebview(notes: Note[]) {
        const notesHtml = notes.map(note => `
            <div class="note" style="margin-bottom: 10px; padding: 10px; border: 1px solid var(--vscode-widget-border);">
                <div><strong>${note.author}</strong> - ${new Date(note.timestamp).toLocaleString()}</div>
                <div>${note.body}</div>
                <div style="margin-top: 5px;">
                    <button onclick="resolve('${note.id}')">Resolve</button>
                    <button onclick="reply('${note.id}')">Reply</button>
                </div>
            </div>
        `).join('');

        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Note Thread</title>
            </head>
            <body style="padding: 10px;">
                <div id="notes-container">
                    ${notesHtml}
                </div>
                <div style="margin-top: 20px;">
                    <textarea id="reply-box" style="width: 100%; height: 60px;" placeholder="Add a new reply..."></textarea>
                    <button style="margin-top: 10px;" onclick="addReply()">Add Reply</button>
                </div>
                <script>
                    const vscode = acquireVsCodeApi();
                    function resolve(noteId) {
                        vscode.postMessage({ command: 'resolve', noteId });
                    }
                    function reply(parentId) {
                        const text = prompt("Enter your reply:");
                        if (text) {
                            vscode.postMessage({ command: 'reply', text, parentId });
                        }
                    }
                    function addReply() {
                        const box = document.getElementById('reply-box');
                        const text = box.value;
                        if (text) {
                            vscode.postMessage({ command: 'reply', text });
                            box.value = '';
                        }
                    }
                </script>
            </body>
            </html>`;
    }
}

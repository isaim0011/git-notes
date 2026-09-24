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
                        vscode.commands.executeCommand('git-notes.resolve', message.noteId, 'resolved');
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
        const notesHtml = notes.map(note => {
            const statusClass = note.status === 'Resolved' || note.status === 'Approved' ? 'color: var(--vscode-charts-green);' : 'color: var(--vscode-charts-yellow);';
            return `
            <div class="note" style="margin-bottom: 12px; padding: 12px; border-radius: 6px; background: var(--vscode-editor-background); border: 1px solid var(--vscode-widget-border);">
                <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                    <strong>${note.author}</strong>
                    <span style="${statusClass} font-size: 11px; font-weight: bold; text-transform: uppercase;">[${note.status}]</span>
                </div>
                <div style="font-size: 11px; color: var(--vscode-descriptionForeground); margin-bottom: 8px;">${new Date(note.timestamp).toLocaleString()}</div>
                <div style="line-height: 1.5; white-space: pre-wrap;">${note.body}</div>
                <div style="margin-top: 10px; display: flex; gap: 8px;">
                    ${note.status !== 'Resolved' ? `<button style="background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 4px 10px; border-radius: 4px; cursor: pointer;" onclick="resolve('${note.id}')">✓ Mark Resolved</button>` : ''}
                    <button style="background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); border: none; padding: 4px 10px; border-radius: 4px; cursor: pointer;" onclick="reply('${note.id}')">💬 Reply</button>
                </div>
            </div>
            `;
        }).join('');

        return `<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Note Thread</title>
                <style>
                    body {
                        font-family: var(--vscode-font-family);
                        color: var(--vscode-foreground);
                        padding: 16px;
                    }
                    textarea {
                        font-family: inherit;
                        background: var(--vscode-input-background);
                        color: var(--vscode-input-foreground);
                        border: 1px solid var(--vscode-input-border);
                        border-radius: 4px;
                        padding: 8px;
                        box-sizing: border-box;
                    }
                    button:hover {
                        opacity: 0.9;
                    }
                </style>
            </head>
            <body>
                <div id="notes-container">
                    ${notesHtml || '<div style="color: var(--vscode-descriptionForeground);">No notes found for this line.</div>'}
                </div>
                <div style="margin-top: 20px;">
                    <textarea id="reply-box" style="width: 100%; height: 70px;" placeholder="Add a new note / reply..."></textarea>
                    <div style="margin-top: 8px; display: flex; justify-content: space-between; align-items: center;">
                        <button style="background: var(--vscode-button-background); color: var(--vscode-button-foreground); border: none; padding: 6px 14px; border-radius: 4px; cursor: pointer;" onclick="addReply()">Post Comment</button>
                        <div style="display: flex; gap: 8px; font-size: 11px;">
                            <a href="https://github.com/isaim0011/git-notes/discussions" style="color: var(--vscode-textLink-foreground); text-decoration: none; display: flex; align-items: center; gap: 4px;">💬 Give Feedback</a>
                            <span style="color: var(--vscode-descriptionForeground);">•</span>
                            <a href="https://github.com/isaim0011/git-notes" style="color: var(--vscode-textLink-foreground); text-decoration: none;">⭐ Star Repo</a>
                        </div>
                    </div>
                </div>
                <script>
                    const vscode = acquireVsCodeApi();
                    function resolve(noteId) {
                        vscode.postMessage({ command: 'resolve', noteId });
                    }
                    function reply(parentId) {
                        const text = prompt("Enter your reply message:");
                        if (text) {
                            vscode.postMessage({ command: 'reply', text, parentId });
                        }
                    }
                    function addReply() {
                        const box = document.getElementById('reply-box');
                        const text = box.value.trim();
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

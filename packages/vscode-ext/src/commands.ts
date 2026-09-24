import * as vscode from 'vscode';
import { exec } from 'child_process';
import { NotePanel } from './notePanel';
import { GutterProvider } from './gutterProvider';
import { Note } from './types';
import * as path from 'path';

export function registerCommands(context: vscode.ExtensionContext, gutterProvider: GutterProvider) {
    const runGitNotesCommand = (args: string[], workspacePath: string): Promise<string> => {
        const binaryPath = vscode.workspace.getConfiguration('git-notes').get<string>('binaryPath', 'git-notes');
        return new Promise((resolve, reject) => {
            exec(`"${binaryPath}" ${args.join(' ')}`, { cwd: workspacePath }, (error, stdout, stderr) => {
                if (error) reject(new Error(stderr || stdout || error.message));
                else resolve(stdout.trim());
            });
        });
    };

    const getActiveWorkspacePath = () => {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
            const workspaceFolder = vscode.workspace.getWorkspaceFolder(editor.document.uri);
            if (workspaceFolder) return workspaceFolder.uri.fsPath;
        }
        if (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders.length > 0) {
            return vscode.workspace.workspaceFolders[0].uri.fsPath;
        }
        return process.cwd();
    };

    context.subscriptions.push(vscode.commands.registerCommand('git-notes.add', async (text?: string, parentId?: string) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active editor.');
            return;
        }

        const workspacePath = getActiveWorkspacePath();
        const relPath = path.relative(workspacePath, editor.document.uri.fsPath).replace(/\\/g, '/');
        const line = editor.selection.active.line + 1;

        let message = text;
        if (!message) {
            message = await vscode.window.showInputBox({ prompt: 'Enter your note message' });
        }
        if (!message) return;

        const defaultNamespace = vscode.workspace.getConfiguration('git-notes').get<string>('defaultNamespace', 'comments');

        try {
            const args = ['add', '-f', relPath, '-l', line.toString(), '-m', `"${message}"`, '-n', defaultNamespace];
            // FIX: CLI uses -t / --thread for parent note ID
            if (parentId) {
                args.push('-t', parentId);
            }
            await runGitNotesCommand(args, workspacePath);
            vscode.window.showInformationMessage('Note added successfully!');
            vscode.commands.executeCommand('git-notes.sync');
        } catch (e: any) {
            vscode.window.showErrorMessage(`Failed to add note: ${e.message}`);
        }
    }));

    context.subscriptions.push(vscode.commands.registerCommand('git-notes.list', async () => {
        const workspacePath = getActiveWorkspacePath();
        try {
            const stdout = await runGitNotesCommand(['list', '--json'], workspacePath);
            const notes: Note[] = JSON.parse(stdout || '[]');
            
            const items = notes.map(n => {
                const nsStr = typeof n.namespace === 'string' ? n.namespace : (n.namespace?.Custom || 'comments');
                const filePath = n.file || 'unknown';
                const lineNum = n.line_start || 1;
                return {
                    label: `[${nsStr}] ${n.body.substring(0, 50)}`,
                    description: `${filePath}:${lineNum} by ${n.author}`,
                    note: n
                };
            });

            const selected = await vscode.window.showQuickPick(items, { placeHolder: 'Select a note to jump to' });
            if (selected && selected.note.file) {
                const docUri = vscode.Uri.file(path.join(workspacePath, selected.note.file));
                const doc = await vscode.workspace.openTextDocument(docUri);
                const editor = await vscode.window.showTextDocument(doc);
                const lineIndex = Math.max(0, (selected.note.line_start || 1) - 1);
                const pos = new vscode.Position(lineIndex, 0);
                editor.selection = new vscode.Selection(pos, pos);
                editor.revealRange(new vscode.Range(pos, pos), vscode.TextEditorRevealType.InCenter);
            }
        } catch (e: any) {
            vscode.window.showErrorMessage(`Failed to list notes: ${e.message}`);
        }
    }));

    context.subscriptions.push(vscode.commands.registerCommand('git-notes.sync', async () => {
        const workspacePath = getActiveWorkspacePath();
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: "Syncing git-notes",
                cancellable: false
            }, async (progress) => {
                try {
                    await runGitNotesCommand(['sync', 'auto'], workspacePath);
                } catch (e) {
                    // Sync auto might fail if no remote is configured, ignore
                }
                progress.report({ increment: 50, message: "Fetching notes..." });
                const stdout = await runGitNotesCommand(['list', '--json'], workspacePath);
                const notes: Note[] = JSON.parse(stdout || '[]');
                
                const notesByFile = new Map<string, Note[]>();
                for (const note of notes) {
                    if (!note.file) continue;
                    const fullPath = path.join(workspacePath, note.file);
                    if (!notesByFile.has(fullPath)) {
                        notesByFile.set(fullPath, []);
                    }
                    notesByFile.get(fullPath)!.push(note);
                }

                for (const [file, fileNotes] of notesByFile.entries()) {
                    gutterProvider.updateNotes(file, fileNotes);
                }
                
                if (vscode.window.activeTextEditor) {
                    gutterProvider.refresh(vscode.window.activeTextEditor);
                }
            });
        } catch (e: any) {
            vscode.window.showErrorMessage(`Sync failed: ${e.message}`);
        }
    }));

    context.subscriptions.push(vscode.commands.registerCommand('git-notes.resolve', async (noteId: string, status: string = 'resolved') => {
        const workspacePath = getActiveWorkspacePath();
        try {
            await runGitNotesCommand(['resolve', '-s', status, noteId], workspacePath);
            vscode.window.showInformationMessage(`Note ${noteId.substring(0, 8)} marked as ${status}`);
            vscode.commands.executeCommand('git-notes.sync');
        } catch (e: any) {
            vscode.window.showErrorMessage(`Failed to resolve note: ${e.message}`);
        }
    }));

    context.subscriptions.push(vscode.commands.registerCommand('git-notes.showPanel', (fsPath?: string, line?: number) => {
        const editor = vscode.window.activeTextEditor;
        if (!editor && (!fsPath || !line)) return;

        const targetPath = fsPath || editor!.document.uri.fsPath;
        const targetLine = line || (editor!.selection.active.line + 1);

        const notes = gutterProvider.getNotesForLine(targetPath, targetLine);
        NotePanel.createOrShow(context.extensionUri, notes);
    }));
}

import * as vscode from 'vscode';
import { Note } from './types';

export class GutterProvider {
    private fileNotes = new Map<string, Note[]>();
    private decorationType: vscode.TextEditorDecorationType;

    constructor() {
        this.decorationType = vscode.window.createTextEditorDecorationType({
            gutterIconPath: vscode.Uri.parse('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><text x="0" y="12" font-size="12">💬</text></svg>'),
            gutterIconSize: 'contain'
        });
    }

    public updateNotes(file: string, notes: Note[]) {
        this.fileNotes.set(file, notes);
    }

    public refresh(editor: vscode.TextEditor, notes?: Note[]) {
        if (!editor) return;
        const file = editor.document.uri.fsPath;
        if (notes) {
            this.fileNotes.set(file, notes);
        }
        const fileNotes = this.fileNotes.get(file) || [];
        
        const decorations: vscode.DecorationOptions[] = [];
        
        for (const note of fileNotes) {
            const lineNum = note.line_start || 1;
            const line = lineNum - 1; // 0-indexed in VS Code
            if (line >= 0 && line < editor.document.lineCount) {
                const range = new vscode.Range(line, 0, line, 0);
                const hoverMessage = new vscode.MarkdownString(`**${note.author}**: ${note.body.substring(0, 100)}${note.body.length > 100 ? '...' : ''}`);
                decorations.push({ range, hoverMessage });
            }
        }
        
        editor.setDecorations(this.decorationType, decorations);
    }

    public getNotesForLine(file: string, line: number): Note[] {
        const fileNotes = this.fileNotes.get(file) || [];
        return fileNotes.filter(n => (n.line_start || 1) === line);
    }

    public dispose() {
        this.decorationType.dispose();
    }
}

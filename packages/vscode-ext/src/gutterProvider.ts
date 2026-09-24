import * as vscode from 'vscode';
import { Note } from './types';

export class GutterProvider {
    private fileNotes = new Map<string, Note[]>();
    private fileNotesByLine = new Map<string, Map<number, Note[]>>();
    private decorationType: vscode.TextEditorDecorationType;

    constructor() {
        this.decorationType = vscode.window.createTextEditorDecorationType({
            gutterIconPath: vscode.Uri.parse('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><text x="0" y="12" font-size="12">💬</text></svg>'),
            gutterIconSize: 'contain'
        });
    }

    public updateNotes(file: string, notes: Note[]) {
        this.fileNotes.set(file, notes);
        this.indexNotesByLine(file, notes);
    }

    private indexNotesByLine(file: string, notes: Note[]) {
        const lineMap = new Map<number, Note[]>();
        for (const note of notes) {
            let lineNotes = lineMap.get(note.line_number);
            if (!lineNotes) {
                lineNotes = [];
                lineMap.set(note.line_number, lineNotes);
            }
            lineNotes.push(note);
        }
        this.fileNotesByLine.set(file, lineMap);
    }

    public refresh(editor: vscode.TextEditor, notes?: Note[]) {
        if (!editor) return;
        const file = editor.document.uri.fsPath;
        if (notes) {
            this.updateNotes(file, notes);
        }
        const fileNotes = this.fileNotes.get(file) || [];
        
        const decorations: vscode.DecorationOptions[] = [];
        
        for (const note of fileNotes) {
            const line = note.line_number - 1; // 0-indexed in VS Code
            if (line >= 0 && line < editor.document.lineCount) {
                const range = new vscode.Range(line, 0, line, 0);
                const hoverMessage = new vscode.MarkdownString(`**${note.author}**: ${note.body.substring(0, 100)}${note.body.length > 100 ? '...' : ''}`);
                decorations.push({ range, hoverMessage });
            }
        }
        
        editor.setDecorations(this.decorationType, decorations);
    }

    public getNotesForLine(file: string, line: number): Note[] {
        const lineMap = this.fileNotesByLine.get(file);
        return lineMap?.get(line) || [];
    }

    public dispose() {
        this.decorationType.dispose();
    }
}

import { test, expect, mock, describe } from 'bun:test';

mock.module('vscode', () => ({
  window: { createTextEditorDecorationType: () => ({ dispose: () => {} }) },
  Uri: { parse: (s: string) => s },
  Position: class Position {
    constructor(public line: number, public character: number) {}
  },
  Range: class Range {
    constructor(
      public start: { line: number; character: number },
      public end: { line: number; character: number }
    ) {}
  },
  InlayHint: class InlayHint {
    paddingLeft?: boolean;
    label?: any;
    tooltip?: string;
    constructor(public position: any, public labelText: string, public kind: any) {}
  },
  InlayHintKind: { Type: 1 },
  InlayHintLabelPart: class InlayHintLabelPart {
    command?: any;
    constructor(public value: string) {}
  }
}));

describe('GutterProvider and InlayHintsProvider correctness and performance', () => {
  test('Correctness: getNotesForLine returns expected notes', async () => {
    const { GutterProvider } = await import('./gutterProvider');
    type Note = import('./types').Note;

    const provider = new GutterProvider();
    const file = '/src/app.ts';

    const note1: Note = { id: '1', file_path: 'app.ts', line_number: 10, body: 'First note', author: 'Alice', created_at: '2025-01-01', namespace: 'comments' };
    const note2: Note = { id: '2', file_path: 'app.ts', line_number: 10, body: 'Second note', author: 'Bob', created_at: '2025-01-01', namespace: 'comments' };
    const note3: Note = { id: '3', file_path: 'app.ts', line_number: 25, body: 'Third note', author: 'Charlie', created_at: '2025-01-01', namespace: 'comments' };

    provider.updateNotes(file, [note1, note2, note3]);

    // Query line with multiple notes
    const line10Notes = provider.getNotesForLine(file, 10);
    expect(line10Notes.length).toBe(2);
    expect(line10Notes[0].id).toBe('1');
    expect(line10Notes[1].id).toBe('2');

    // Query line with single note
    const line25Notes = provider.getNotesForLine(file, 25);
    expect(line25Notes.length).toBe(1);
    expect(line25Notes[0].id).toBe('3');

    // Query line with no notes
    const line5Notes = provider.getNotesForLine(file, 5);
    expect(line5Notes.length).toBe(0);

    // Query non-existent file
    const unknownFileNotes = provider.getNotesForLine('/src/unknown.ts', 10);
    expect(unknownFileNotes.length).toBe(0);

    // Update notes (overwrite)
    provider.updateNotes(file, [note3]);
    expect(provider.getNotesForLine(file, 10).length).toBe(0);
    expect(provider.getNotesForLine(file, 25).length).toBe(1);
  });

  test('Performance Benchmark: InlayHintsProvider with large file and notes', async () => {
    const { GutterProvider } = await import('./gutterProvider');
    const { InlayHintsProvider } = await import('./inlayProvider');
    type Note = import('./types').Note;

    const gutterProvider = new GutterProvider();
    const file = '/test/file.ts';
    const totalLines = 1000;
    const notesCount = 5000;

    const notes: Note[] = [];
    for (let i = 0; i < notesCount; i++) {
      notes.push({
        id: `note-${i}`,
        file_path: 'file.ts',
        line_number: (i % totalLines) + 1,
        body: `Note body ${i}`,
        author: `User ${i}`,
        created_at: '2025-01-01',
        namespace: 'comments'
      });
    }

    gutterProvider.updateNotes(file, notes);

    const mockDocument: any = {
      uri: { fsPath: file },
      lineAt: (line: number) => ({ text: `line ${line} content` })
    };

    const range: any = {
      start: { line: 0, character: 0 },
      end: { line: totalLines - 1, character: 0 }
    };

    const provider = new InlayHintsProvider(gutterProvider);

    // Measure time
    const start = performance.now();
    const iterations = 100;
    let hintsCount = 0;
    for (let i = 0; i < iterations; i++) {
      const hints = provider.provideInlayHints(mockDocument, range, {} as any) as any[];
      hintsCount = hints.length;
    }
    const duration = performance.now() - start;

    console.log(`Benchmark completed: ${iterations} iterations took ${duration.toFixed(2)}ms (${(duration / iterations).toFixed(4)}ms per call)`);
    expect(hintsCount).toBe(totalLines);
  });
});

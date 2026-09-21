import { h } from 'preact';
import { Note } from '../types';

export function FileTree({ notes, selectedFile, onSelectFile }: { notes: Note[], selectedFile: string | null, onSelectFile: (file: string) => void }) {
    const fileMap = new Map<string, number>();
    notes.forEach(note => {
        fileMap.set(note.file_path, (fileMap.get(note.file_path) || 0) + 1);
    });

    const files = Array.from(fileMap.keys()).sort();

    return (
        <div style={{ width: '250px', borderRight: '1px solid #ddd', padding: '10px', height: '100vh', overflowY: 'auto', background: '#fff' }}>
            <h3 style={{ marginTop: 0 }}>Files</h3>
            <ul style={{ listStyle: 'none', padding: 0 }}>
                {files.map(file => (
                    <li key={file} 
                        style={{ padding: '8px', cursor: 'pointer', background: selectedFile === file ? '#e0e0e0' : 'transparent', borderRadius: '4px', marginBottom: '4px', display: 'flex', justifyContent: 'space-between' }}
                        onClick={() => onSelectFile(file)}>
                        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }} title={file}>{file.split('/').pop()}</span>
                        <span style={{ background: '#007acc', color: 'white', borderRadius: '10px', padding: '2px 6px', fontSize: '12px' }}>{fileMap.get(file)}</span>
                    </li>
                ))}
            </ul>
        </div>
    );
}

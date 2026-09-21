import { h } from 'preact';
import { useState, useEffect } from 'preact/hooks';
import { Note } from './types';
import { FileTree } from './components/FileTree';
import { NamespaceFilter } from './components/NamespaceFilter';
import { NoteCard } from './components/NoteCard';

export function App() {
    const [notes, setNotes] = useState<Note[]>([]);
    const [selectedFile, setSelectedFile] = useState<string | null>(null);
    const [namespace, setNamespace] = useState<string>('All');
    const [search, setSearch] = useState<string>('');

    useEffect(() => {
        fetch('./notes.json')
            .then(res => res.json())
            .then(data => setNotes(data))
            .catch(err => {
                console.error("Failed to load notes.json", err);
                setNotes([
                    { id: '1', file_path: 'src/main.tsx', line_number: 10, namespace: 'comments', author: 'Dev', timestamp: new Date().toISOString(), body: 'Test note!', status: 'open' }
                ]);
            });
    }, []);

    const filteredNotes = notes.filter(n => {
        const matchNs = namespace === 'All' || n.namespace === namespace;
        const matchSearch = n.body.toLowerCase().includes(search.toLowerCase()) || n.author.toLowerCase().includes(search.toLowerCase());
        return matchNs && matchSearch;
    });

    const fileNotes = selectedFile ? filteredNotes.filter(n => n.file_path === selectedFile) : [];
    
    // Get top-level notes for the selected file
    const rootNotes = fileNotes.filter(n => !n.parent_id);

    return (
        <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
            <FileTree notes={filteredNotes} selectedFile={selectedFile} onSelectFile={setSelectedFile} />
            
            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', padding: '20px', background: '#fafafa', overflowY: 'auto' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
                    <NamespaceFilter current={namespace} onChange={setNamespace} />
                    <input 
                        type="text" 
                        placeholder="Search notes..." 
                        value={search} 
                        onInput={(e) => setSearch((e.target as HTMLInputElement).value)} 
                        style={{ padding: '8px', border: '1px solid #ccc', borderRadius: '4px', width: '250px' }}
                    />
                </div>

                {selectedFile ? (
                    <div>
                        <h2 style={{ marginTop: 0 }}>{selectedFile}</h2>
                        {rootNotes.length === 0 ? (
                            <p>No notes matching the filter criteria.</p>
                        ) : (
                            rootNotes.map(note => <NoteCard key={note.id} note={note} allNotes={filteredNotes} />)
                        )}
                    </div>
                ) : (
                    <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%', color: '#999' }}>
                        <h3>Select a file from the sidebar to view notes</h3>
                    </div>
                )}
            </div>
        </div>
    );
}

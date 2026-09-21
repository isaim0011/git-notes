import { h, Fragment } from 'preact';
import { useState } from 'preact/hooks';
import { Note } from '../types';

export function NoteCard({ note, allNotes, depth = 0 }: { note: Note, allNotes: Note[], depth?: number }) {
    const [collapsed, setCollapsed] = useState(false);
    const replies = allNotes.filter(n => n.parent_id === note.id);

    return (
        <div style={{ marginLeft: `${depth * 20}px`, marginBottom: '10px', border: '1px solid #ddd', borderRadius: '6px', background: '#fff', padding: '10px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                <div>
                    <strong style={{ marginRight: '10px' }}>{note.author}</strong>
                    <span style={{ color: '#666', fontSize: '0.9em' }}>{new Date(note.timestamp).toLocaleString()}</span>
                    <span style={{ marginLeft: '10px', background: note.status === 'resolved' ? '#2ea043' : '#dbab09', color: 'white', padding: '2px 6px', borderRadius: '10px', fontSize: '12px' }}>{note.status || 'open'}</span>
                    <span style={{ marginLeft: '10px', background: '#eee', padding: '2px 6px', borderRadius: '4px', fontSize: '12px' }}>{note.namespace}</span>
                    <span style={{ marginLeft: '10px', color: '#666', fontSize: '0.9em' }}>Line: {note.line_number}</span>
                </div>
                <button onClick={() => setCollapsed(!collapsed)} style={{ border: 'none', background: 'transparent', cursor: 'pointer' }}>
                    {collapsed ? '▼' : '▲'}
                </button>
            </div>
            {!collapsed && (
                <Fragment>
                    <div style={{ whiteSpace: 'pre-wrap', marginBottom: '10px' }}>{note.body}</div>
                    {replies.length > 0 && (
                        <div style={{ marginTop: '10px', borderTop: '1px solid #eee', paddingTop: '10px' }}>
                            {replies.map(reply => <NoteCard key={reply.id} note={reply} allNotes={allNotes} depth={depth + 1} />)}
                        </div>
                    )}
                </Fragment>
            )}
        </div>
    );
}

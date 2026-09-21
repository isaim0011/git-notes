import { h } from 'preact';

const namespaces = ['All', 'comments', 'review', 'todos'];

export function NamespaceFilter({ current, onChange }: { current: string, onChange: (ns: string) => void }) {
    return (
        <div style={{ display: 'flex', gap: '10px', marginBottom: '20px' }}>
            {namespaces.map(ns => (
                <button
                    key={ns}
                    style={{
                        padding: '6px 12px',
                        border: '1px solid #ccc',
                        background: current === ns ? '#007acc' : '#fff',
                        color: current === ns ? '#fff' : '#333',
                        borderRadius: '4px',
                        cursor: 'pointer'
                    }}
                    onClick={() => onChange(ns)}
                >
                    {ns}
                </button>
            ))}
        </div>
    );
}

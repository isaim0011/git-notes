export type NoteStatus = 'open' | 'resolved';

export interface Note {
    id: string;
    parent_id?: string;
    file_path: string;
    line_number: number;
    namespace: string;
    author: string;
    timestamp: string;
    body: string;
    status: NoteStatus;
}

export interface Note {
    id: string;
    commit: string;
    file?: string | null;
    line_start?: number | null;
    line_end?: number | null;
    namespace: string | { Custom?: string };
    author: string;
    timestamp: string;
    body: string;
    thread_id?: string | null;
    status: 'Open' | 'Resolved' | 'Approved' | 'Rejected';
    tags?: string[];
}

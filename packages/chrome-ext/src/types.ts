export interface Note {
  id: string;
  body: string;
  file: string;
  line: number;
  namespace: string;
  author?: string;
  timestamp?: string;
}

export interface ExtensionConfig {
  bridgeUrl: string;
  token: string;
}

export type MessageType =
  | { type: 'GET_NOTES'; file?: string; line?: number; namespace?: string }
  | { type: 'ADD_NOTE'; file: string; line: number; body: string; namespace?: string }
  | { type: 'GET_CONFIG' };

export type MessageResponse =
  | { type: 'NOTES_RESULT'; data: Note[] }
  | { type: 'ADD_NOTE_RESULT'; success: boolean; error?: string }
  | { type: 'CONFIG_RESULT'; data: ExtensionConfig }
  | { type: 'ERROR'; error: string };

import { ExtensionConfig, MessageType, MessageResponse, Note } from './types';

const DEFAULT_CONFIG: ExtensionConfig = {
  bridgeUrl: 'http://localhost:7477',
  token: ''
};

chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.sync.get(['bridgeUrl', 'token'], (result) => {
    if (!result.bridgeUrl) {
      chrome.storage.sync.set(DEFAULT_CONFIG);
    }
  });

  chrome.contextMenus.create({
    id: 'add-git-note',
    title: 'Add git-note',
    contexts: ['selection', 'page']
  });
});

async function getConfig(): Promise<ExtensionConfig> {
  return new Promise((resolve) => {
    chrome.storage.sync.get(['bridgeUrl', 'token'], (result) => {
      resolve({
        bridgeUrl: result.bridgeUrl || DEFAULT_CONFIG.bridgeUrl,
        token: result.token || DEFAULT_CONFIG.token
      });
    });
  });
}

chrome.runtime.onMessage.addListener((request: MessageType, sender, sendResponse) => {
  if (request.type === 'GET_CONFIG') {
    getConfig().then(config => {
      sendResponse({ type: 'CONFIG_RESULT', data: config });
    });
    return true; // Keep channel open
  }

  if (request.type === 'GET_NOTES') {
    handleGetNotes(request).then(sendResponse);
    return true;
  }

  if (request.type === 'ADD_NOTE') {
    handleAddNote(request).then(sendResponse);
    return true;
  }
});

async function handleGetNotes(req: { file?: string; line?: number; namespace?: string }): Promise<MessageResponse> {
  try {
    const config = await getConfig();
    const url = new URL('/api/notes', config.bridgeUrl);
    
    if (req.file) url.searchParams.append('file', req.file);
    if (req.line) url.searchParams.append('line', req.line.toString());
    if (req.namespace) url.searchParams.append('namespace', req.namespace);

    const headers: Record<string, string> = {};
    if (config.token) headers['Authorization'] = `Bearer ${config.token}`;

    const res = await fetch(url.toString(), { headers });
    
    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }
    
    const notes: Note[] = await res.json();
    return { type: 'NOTES_RESULT', data: notes || [] };
  } catch (error: any) {
    console.error('Error fetching notes:', error);
    return { type: 'ERROR', error: error.message };
  }
}

async function handleAddNote(req: { file: string; line: number; body: string; namespace?: string }): Promise<MessageResponse> {
  try {
    const config = await getConfig();
    const url = new URL('/api/notes', config.bridgeUrl);
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    };
    if (config.token) headers['Authorization'] = `Bearer ${config.token}`;

    const payload = {
      file: req.file,
      line: req.line,
      body: req.body,
      namespace: req.namespace || 'default'
    };

    const res = await fetch(url.toString(), {
      method: 'POST',
      headers,
      body: JSON.stringify(payload)
    });
    
    if (!res.ok) {
      throw new Error(`HTTP error! status: ${res.status}`);
    }
    
    return { type: 'ADD_NOTE_RESULT', success: true };
  } catch (error: any) {
    console.error('Error adding note:', error);
    return { type: 'ADD_NOTE_RESULT', success: false, error: error.message };
  }
}

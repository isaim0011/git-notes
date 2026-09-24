import { Note, MessageType, MessageResponse } from './types';

// Inject CSS
const style = document.createElement('style');
style.textContent = `
  .gn-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background-color: #ffd33d;
    color: #24292e;
    border-radius: 12px;
    padding: 2px 8px;
    font-size: 12px;
    font-weight: 600;
    margin-left: 8px;
    cursor: pointer;
    user-select: none;
    box-shadow: 0 1px 3px rgba(0,0,0,0.12);
  }
  .gn-badge:hover {
    background-color: #f9c513;
  }
  .gn-popup {
    position: absolute;
    background: white;
    border: 1px solid #d1d5da;
    border-radius: 6px;
    padding: 12px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
    z-index: 9999;
    max-width: 300px;
    font-family: -apple-system,BlinkMacSystemFont,"Segoe UI",Helvetica,Arial,sans-serif;
  }
  .gn-popup-header {
    font-weight: 600;
    margin-bottom: 8px;
    color: #24292e;
    display: flex;
    justify-content: space-between;
  }
  .gn-popup-close {
    cursor: pointer;
    color: #586069;
  }
  .gn-popup-close:hover {
    color: #24292e;
  }
  .gn-note-item {
    padding: 8px;
    background: #f6f8fa;
    border-radius: 6px;
    margin-bottom: 8px;
    font-size: 13px;
    color: #24292e;
  }
  .gn-note-item:last-child {
    margin-bottom: 0;
  }
`;
document.head.appendChild(style);

function init() {
  if (!window.location.pathname.includes('/pull/') || !window.location.pathname.endsWith('/files')) {
    // Only run on PR files page
    return;
  }

  // Find all file containers
  const fileContainers = document.querySelectorAll('.js-file');
  
  fileContainers.forEach(container => {
    const fileHeader = container.querySelector('.file-info a.Link--primary');
    if (!fileHeader) return;
    const filePath = fileHeader.getAttribute('title') || fileHeader.textContent?.trim();
    if (!filePath) return;

    const diffLines = container.querySelectorAll('tr.js-addition, tr.js-deletion, tr.js-context');
    
    diffLines.forEach(lineRow => {
      // Get right side line number for additions/context, left side for deletions
      const lineNumCell = lineRow.querySelector('td.blob-num[data-line-number]');
      if (!lineNumCell) return;
      
      const lineNumStr = lineNumCell.getAttribute('data-line-number');
      if (!lineNumStr) return;
      
      const lineNum = parseInt(lineNumStr, 10);
      if (isNaN(lineNum)) return;

      const codeCell = lineRow.querySelector('td.blob-code');
      if (!codeCell) return;

      // Listen for right clicks to add note context menu
      lineRow.addEventListener('contextmenu', (e) => {
        // We'll let the background script handle context menus via standard Chrome API
        // For standard Chrome context menu, we need to pass data
        // For now we just add an attribute
        lineRow.setAttribute('data-gn-file', filePath);
        lineRow.setAttribute('data-gn-line', lineNum.toString());
      });

      // Request notes for this file/line
      chrome.runtime.sendMessage(
        { type: 'GET_NOTES', file: filePath, line: lineNum } as MessageType,
        (response: MessageResponse) => {
          if (response && response.type === 'NOTES_RESULT' && response.data.length > 0) {
            injectBadge(codeCell as HTMLElement, response.data);
          }
        }
      );
    });
  });
}

let activePopup: HTMLElement | null = null;

function closePopup() {
  if (activePopup) {
    activePopup.remove();
    activePopup = null;
  }
}

document.addEventListener('click', (e) => {
  if (activePopup && !activePopup.contains(e.target as Node)) {
    closePopup();
  }
});

function injectBadge(codeElement: HTMLElement, notes: Note[]) {
  // Prevent duplicate badges
  if (codeElement.querySelector('.gn-badge')) return;

  const badge = document.createElement('span');
  badge.className = 'gn-badge';
  badge.textContent = `💬 ${notes.length}`;
  
  badge.addEventListener('click', (e) => {
    e.stopPropagation();
    closePopup();

    const popup = document.createElement('div');
    popup.className = 'gn-popup';
    
    const header = document.createElement('div');
    header.className = 'gn-popup-header';
    header.innerHTML = `
      <span>git-notes (${notes.length})</span>
      <span class="gn-popup-close">✕</span>
    `;
    header.querySelector('.gn-popup-close')?.addEventListener('click', closePopup);
    popup.appendChild(header);

    notes.forEach(note => {
      const item = document.createElement('div');
      item.className = 'gn-note-item';
      
      if (note.author) {
        const authorEl = document.createElement('strong');
        authorEl.textContent = note.author;
        item.appendChild(authorEl);
        item.appendChild(document.createTextNode(':'));
        item.appendChild(document.createElement('br'));
      }

      item.appendChild(document.createTextNode(note.body));
      popup.appendChild(item);
    });

    document.body.appendChild(popup);
    
    const rect = badge.getBoundingClientRect();
    popup.style.top = `${window.scrollY + rect.bottom + 5}px`;
    popup.style.left = `${window.scrollX + rect.left}px`;
    
    activePopup = popup;
  });

  // Inject at the end of the line content
  const innerText = codeElement.querySelector('.blob-code-inner');
  if (innerText) {
    innerText.appendChild(badge);
  } else {
    codeElement.appendChild(badge);
  }
}

// Observe DOM for AJAX loaded diffs
const observer = new MutationObserver((mutations) => {
  let shouldRun = false;
  for (const mutation of mutations) {
    if (mutation.addedNodes.length > 0) {
      shouldRun = true;
      break;
    }
  }
  if (shouldRun) {
    // Debounce to prevent excessive calls
    clearTimeout((window as any).gnTimeout);
    (window as any).gnTimeout = setTimeout(init, 500);
  }
});

observer.observe(document.body, { childList: true, subtree: true });

// Initial run
init();

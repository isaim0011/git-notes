document.addEventListener('DOMContentLoaded', () => {
  const urlInput = document.getElementById('bridge-url') as HTMLInputElement;
  const tokenInput = document.getElementById('token') as HTMLInputElement;
  const saveBtn = document.getElementById('save-btn') as HTMLButtonElement;
  const statusMsg = document.getElementById('status-msg') as HTMLDivElement;
  const dot = document.getElementById('connection-dot') as HTMLSpanElement;

  // Load saved config
  chrome.storage.sync.get(['bridgeUrl', 'token'], (result) => {
    if (result.bridgeUrl) urlInput.value = result.bridgeUrl;
    if (result.token) tokenInput.value = result.token;
    
    testConnection(urlInput.value, tokenInput.value);
  });

  // Save config
  saveBtn.addEventListener('click', () => {
    const bridgeUrl = urlInput.value.trim();
    const token = tokenInput.value.trim();

    chrome.storage.sync.set({ bridgeUrl, token }, () => {
      statusMsg.textContent = 'Settings saved.';
      statusMsg.style.color = '#28a745';
      setTimeout(() => {
        statusMsg.textContent = '';
      }, 2000);
      
      testConnection(bridgeUrl, token);
    });
  });

  async function testConnection(url: string, token: string) {
    if (!url) return;
    
    try {
      // Just check health endpoint if it exists, or notes endpoint
      const checkUrl = new URL('/health', url).toString();
      const headers: Record<string, string> = {};
      if (token) headers['Authorization'] = `Bearer ${token}`;

      const res = await fetch(checkUrl, { headers });
      if (res.ok) {
        dot.className = 'status-dot connected';
        dot.title = 'Connected to bridge API';
      } else {
        throw new Error('Bad response');
      }
    } catch (e) {
      dot.className = 'status-dot error';
      dot.title = 'Cannot connect to bridge API';
    }
  }
});

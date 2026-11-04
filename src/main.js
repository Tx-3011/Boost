import { invoke } from '@tauri-apps/api/core';
import './style.css';

const adaptersElement = document.querySelector('#adapters');
const statusElement = document.querySelector('#status');
const startButton = document.querySelector('#start');
const stopButton = document.querySelector('#stop');
const refreshButton = document.querySelector('#refresh');
let running = false;
let busy = false;

function selectedAddresses() {
  return [...adaptersElement.querySelectorAll('input:checked')].map((input) => input.value);
}

function syncButtons() {
  startButton.disabled = busy || running || selectedAddresses().length === 0;
  stopButton.disabled = busy || !running;
  refreshButton.disabled = busy;
}

function showStatus(message) { statusElement.textContent = message; }

async function refreshAdapters() {
  busy = true;
  syncButtons();
  const previous = new Set(selectedAddresses());
  try {
    const adapters = await invoke('list_adapters');
    adaptersElement.replaceChildren();
    let count = 0;
    for (const adapter of adapters) {
      for (const address of adapter.ipv4) {
        const label = document.createElement('label');
        label.className = 'adapter';
        const input = document.createElement('input');
        input.type = 'checkbox';
        input.value = address;
        input.checked = previous.has(address);
        input.disabled = running;
        const text = document.createElement('span');
        const title = document.createElement('strong');
        title.textContent = adapter.name;
        const subtitle = document.createElement('small');
        subtitle.textContent = address;
        text.append(title, subtitle);
        label.append(input, text);
        adaptersElement.append(label);
        count++;
      }
    }
    if (!count) adaptersElement.textContent = 'No usable IPv4 addresses found. Check USB tethering and refresh.';
  } catch (error) {
    adaptersElement.textContent = `Cannot read adapters: ${error}`;
  } finally {
    busy = false;
    syncButtons();
  }
}

async function updateStatus() {
  try {
    const status = await invoke('proxy_status');
    running = status.running;
    if (running) showStatus(status.ready ? 'Proxy listening on 127.0.0.1:1080 (Steam not routed automatically).' : 'Dispatch started; waiting for proxy listener.');
    else showStatus(status.message || 'Not running');
    adaptersElement.querySelectorAll('input').forEach((input) => { input.disabled = running; });
  } catch (error) { showStatus(`Status error: ${error}`); }
  syncButtons();
}

startButton.addEventListener('click', async () => {
  busy = true; syncButtons();
  try { await invoke('start_proxy', { addresses: selectedAddresses() }); await updateStatus(); }
  catch (error) { showStatus(`Start failed: ${error}`); }
  finally { busy = false; syncButtons(); }
});
stopButton.addEventListener('click', async () => {
  busy = true; syncButtons();
  try { await invoke('stop_proxy'); await updateStatus(); await refreshAdapters(); }
  catch (error) { showStatus(`Stop failed: ${error}`); }
  finally { busy = false; syncButtons(); }
});
refreshButton.addEventListener('click', refreshAdapters);
adaptersElement.addEventListener('change', syncButtons);
void (async () => { await updateStatus(); await refreshAdapters(); })();
setInterval(() => { if (!busy) void updateStatus(); }, 3000);

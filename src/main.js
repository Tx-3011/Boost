import { invoke } from '@tauri-apps/api/core';
import { createTrafficTracker } from './diagnostics.js';
import './style.css';

const adaptersElement = document.querySelector('#adapters');
const statusElement = document.querySelector('#status');
const startButton = document.querySelector('#start');
const stopButton = document.querySelector('#stop');
const refreshButton = document.querySelector('#refresh');
const checkButton = document.querySelector('#check-connections');
const diagnosticsElement = document.querySelector('#diagnostics');
const diagnosticStatus = document.querySelector('#diagnostic-status');
const chooseApplicationButton = document.querySelector('#choose-application');
const boostApplicationButton = document.querySelector('#boost-application');
const selectedApplicationElement = document.querySelector('#selected-application');
const routingSetup = document.querySelector('#routing-setup');
const refreshSetupButton = document.querySelector('#refresh-setup');
const installRoutingButton = document.querySelector('#install-routing');
const trafficTracker = createTrafficTracker();
const adapterNames = new Map();
const probeResults = new Map();
let running = false;
let busy = false;
let checking = false;
let trafficBusy = false;
let activeAddresses = [];
let lastTraffic = [];
let diagnosticsVersion = 0;
let application = null;
let setupReady = false;
let statusError = '';

function selectedAddresses() {
  return [...adaptersElement.querySelectorAll('input:checked')].map((input) => input.value);
}

function diagnosticAddresses() { return running ? activeAddresses : selectedAddresses(); }

function syncButtons() {
  startButton.disabled = busy || running || selectedAddresses().length === 0;
  stopButton.disabled = busy || !running;
  refreshButton.disabled = busy;
  checkButton.disabled = busy || checking || diagnosticAddresses().length === 0;
  chooseApplicationButton.disabled = busy || running;
  refreshSetupButton.disabled = busy || running;
  installRoutingButton.disabled = busy || running;
  const interfaces = new Set([...adaptersElement.querySelectorAll('input:checked')].map((input) => input.dataset.index));
  boostApplicationButton.disabled = busy || running || !application || !setupReady || interfaces.size < 2;
  boostApplicationButton.title = interfaces.size < 2 ? 'Select at least two different network adapters.' : !setupReady ? 'Open Routing component setup to finish installation.' : '';
}

function showStatus(message) { statusElement.textContent = message; }

function formatBytes(bytes) {
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  const unit = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 3);
  return `${(bytes / 1024 ** unit).toFixed(1)} ${['B', 'KiB', 'MiB', 'GiB'][unit]}`;
}

function appendLine(parent, text, className = '') {
  const line = document.createElement('p');
  line.textContent = text;
  if (className) line.className = className;
  parent.append(line);
}

function renderDiagnostics() {
  diagnosticsElement.replaceChildren();
  const addresses = diagnosticAddresses();
  if (!addresses.length) {
    diagnosticsElement.textContent = 'Select an adapter to see its traffic.';
    return;
  }
  const byAddress = new Map(lastTraffic.map((item) => [item.address, item]));
  for (const address of addresses) {
    const item = byAddress.get(address);
    const row = document.createElement('div');
    row.className = 'diagnostic';
    const title = document.createElement('strong');
    title.textContent = adapterNames.get(address) || 'Selected adapter';
    const subtitle = document.createElement('small');
    subtitle.textContent = address;
    row.append(title, subtitle);
    if (item?.error) appendLine(row, item.error, 'error');
    else if (item?.reading) {
      const { receivedPerSecond, sentPerSecond, receivedSince, sentSince } = item.reading;
      appendLine(row, receivedPerSecond == null ? 'Sampling adapter traffic…' : `Received ${formatBytes(receivedPerSecond)}/s · Sent ${formatBytes(sentPerSecond)}/s`);
      appendLine(row, `Since selection: ${formatBytes(receivedSince)} received · ${formatBytes(sentSince)} sent`, 'hint');
    } else appendLine(row, 'Waiting for the first adapter counter sample…');
    const probe = probeResults.get(address);
    appendLine(row, probe ? `Connection check: ${probe.message}` : 'Connection check: Not checked', probe ? (probe.reachable ? 'ok' : 'error') : 'hint');
    diagnosticsElement.append(row);
  }
}

async function refreshTraffic() {
  if (trafficBusy) return;
  const addresses = [...diagnosticAddresses()];
  const version = diagnosticsVersion;
  if (!addresses.length) { trafficTracker.reset(); lastTraffic = []; renderDiagnostics(); return; }
  trafficBusy = true;
  try {
    const rows = await invoke('adapter_traffic', { addresses });
    if (version !== diagnosticsVersion || addresses.join('|') !== diagnosticAddresses().join('|')) return;
    lastTraffic = trafficTracker.record(rows, performance.now());
    for (const row of rows) if (row.error) probeResults.delete(row.address);
    diagnosticStatus.textContent = '';
  } catch (error) {
    if (version === diagnosticsVersion && addresses.join('|') === diagnosticAddresses().join('|')) {
      trafficTracker.reset();
      lastTraffic = [];
      diagnosticStatus.textContent = `Cannot read adapter traffic: ${error}`;
    }
  } finally {
    trafficBusy = false;
    renderDiagnostics();
  }
}

async function refreshAdapters() {
  busy = true;
  syncButtons();
  const previous = new Set(selectedAddresses());
  try {
    const adapters = await invoke('list_adapters');
    adaptersElement.replaceChildren();
    diagnosticsVersion++;
    adapterNames.clear();
    probeResults.clear();
    trafficTracker.reset();
    lastTraffic = [];
    diagnosticStatus.textContent = '';
    let count = 0;
    for (const adapter of adapters) {
      for (const address of adapter.ipv4) {
        adapterNames.set(address, adapter.name);
        const label = document.createElement('label');
        label.className = 'adapter';
        const input = document.createElement('input');
        input.type = 'checkbox';
        input.value = address;
        input.dataset.index = adapter.index;
        input.checked = (running ? new Set(activeAddresses) : previous).has(address);
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
    renderDiagnostics();
    void refreshTraffic();
  }
}

async function updateStatus() {
  try {
    const status = await invoke('proxy_status');
    running = status.running;
    activeAddresses = status.addresses;
    if (running && status.routing) {
      application = status.application;
      selectedApplicationElement.textContent = `${application.name} — ${application.path}`;
      showStatus(`Routing engine active for ${application.name}. Start a fresh download to test both links; traffic capture is not yet verified.`);
    } else if (running) showStatus('Local proxy ready on 127.0.0.1:1080. Application routing is off.');
    else showStatus(statusError || status.message || 'Not running');
    adaptersElement.querySelectorAll('input').forEach((input) => { input.disabled = running; });
  } catch (error) { showStatus(`Status error: ${error}`); }
  syncButtons();
}

startButton.addEventListener('click', async () => {
  const addresses = selectedAddresses();
  statusError = '';
  showStatus('Starting local proxy…');
  busy = true; syncButtons();
  try { await invoke('start_proxy', { addresses }); activeAddresses = addresses; await updateStatus(); void refreshTraffic(); }
  catch (error) { statusError = `Start failed: ${error}`; showStatus(statusError); }
  finally { busy = false; syncButtons(); }
});
stopButton.addEventListener('click', async () => {
  statusError = '';
  busy = true; syncButtons();
  try { await invoke('stop_proxy'); activeAddresses = []; await updateStatus(); await refreshAdapters(); }
  catch (error) { statusError = `Stop failed: ${error}`; showStatus(statusError); }
  finally { busy = false; syncButtons(); }
});
refreshButton.addEventListener('click', refreshAdapters);
adaptersElement.addEventListener('change', () => {
  diagnosticsVersion++;
  probeResults.clear();
  trafficTracker.reset();
  lastTraffic = [];
  diagnosticStatus.textContent = '';
  syncButtons();
  renderDiagnostics();
  void refreshTraffic();
});
checkButton.addEventListener('click', async () => {
  const addresses = [...diagnosticAddresses()];
  const version = diagnosticsVersion;
  checking = true;
  diagnosticStatus.textContent = 'Checking selected connections…';
  syncButtons();
  try {
    const results = await invoke('check_connections', { addresses });
    if (version !== diagnosticsVersion || addresses.join('|') !== diagnosticAddresses().join('|')) return;
    for (const result of results) probeResults.set(result.address, result);
    diagnosticStatus.textContent = `Checked ${results.length} connection${results.length === 1 ? '' : 's'}.`;
  } catch (error) {
    if (version === diagnosticsVersion) diagnosticStatus.textContent = `Connection check failed: ${error}`;
  }
  finally { checking = false; syncButtons(); renderDiagnostics(); }
});
async function refreshSetup() {
  try {
    const result = await invoke('routing_prerequisites');
    setupReady = result.engine_available && result.driver_available && result.setup_complete;
    routingSetup.textContent = result.message;
  } catch (error) { setupReady = false; routingSetup.textContent = `Cannot check routing setup: ${error}`; }
  syncButtons();
}
refreshSetupButton.addEventListener('click', refreshSetup);
installRoutingButton.addEventListener('click', async () => {
  busy = true; syncButtons();
  routingSetup.textContent = 'Downloading and verifying routing components… Approve the Windows administrator prompt to finish network setup.';
  try {
    await invoke('install_routing');
    await refreshSetup();
  } catch (error) { routingSetup.textContent = String(error); setupReady = false; }
  finally { busy = false; syncButtons(); }
});
chooseApplicationButton.addEventListener('click', async () => {
  busy = true; syncButtons();
  try {
    const selected = await invoke('choose_application');
    if (selected) {
      application = selected;
      selectedApplicationElement.textContent = `${selected.name} — ${selected.path}`;
    }
  } catch (error) { statusError = `Cannot select application: ${error}`; showStatus(statusError); }
  finally { busy = false; syncButtons(); }
});
boostApplicationButton.addEventListener('click', async () => {
  busy = true; statusError = ''; syncButtons();
  showStatus('Starting Dispatch and activating the application rule…');
  try {
    await invoke('start_application', { addresses: selectedAddresses(), path: application.path });
    await updateStatus();
    void refreshTraffic();
  } catch (error) { statusError = `Could not boost application: ${error}`; showStatus(statusError); }
  finally { busy = false; syncButtons(); }
});
void (async () => { await updateStatus(); await refreshAdapters(); await refreshSetup(); })();
setInterval(() => { if (!busy) void updateStatus(); }, 3000);
setInterval(() => { void refreshTraffic(); }, 2000);

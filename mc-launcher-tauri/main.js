// Modern Minecraft Launcher Frontend
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { open } from '@tauri-apps/plugin-shell';
import { listen } from '@tauri-apps/api/event';
import { resolve } from '@tauri-apps/api/path';

console.log('🎮 Minecraft Launcher Starting...');

// App state
const state = {
  isLoggedIn: false,
  currentAccount: null,
  activeProcesses: [],
  versions: [],
  currentProcess: null,
  processMonitorInterval: null,
  settings: {
    memoryMin: 4096,
    memoryMax: 8192,
    concurrentDownloads: 8,
    downloadTimeout: 300,
    gcType: 'G1GC',
    theme: 'auto',
    gcFlags: ['-XX:+UseG1GC'],
    customJvmArgs: [],
    minecraftDir: '',
    javaPath: ''
  }
};

// Initialize the application
document.addEventListener('DOMContentLoaded', async () => {
  console.log('DOM loaded, initializing app...');
  
  // Set up backend log listener
  await setupLogListener();
  
  try {
    initializeElements();
    setupEventListeners();
    loadSettings();
    
    // Check if we have saved credentials
    const savedAccount = localStorage.getItem('minecraft-account');
    if (savedAccount) {
      try {
        state.currentAccount = JSON.parse(savedAccount);
        state.isLoggedIn = true;
        await initializeLauncher(); // Initialize launcher with saved account
        await loadVersions();
        showMainScreen();
      } catch (error) {
        console.warn('Invalid saved account data:', error);
        localStorage.removeItem('minecraft-account');
        showLoginScreen();
      }
    } else {
      showLoginScreen();
    }
  } catch (error) {
    console.error('Failed to initialize app:', error);
    showNotification('Failed to initialize app: ' + error.message, 'error');
  }
});

// DOM elements
const elements = {};

function initializeElements() {
  // Screens
  elements.loginScreen = document.getElementById('login-screen');
  elements.mainScreen = document.getElementById('main-screen');
  elements.loadingOverlay = document.getElementById('loading-overlay');
  elements.settingsModal = document.getElementById('settings-modal');
  elements.versionModal = document.getElementById('version-modal');
  
  // Login elements
  elements.loginBtn = document.getElementById('login-btn');
  elements.authCodeInput = document.getElementById('auth-code-input');
  elements.completeAuthBtn = document.getElementById('complete-auth-btn');
  elements.manualAuthLink = document.getElementById('manual-auth-link');
  
  // Main screen elements
  elements.userInfo = document.getElementById('user-info');
  elements.launchBtn = document.getElementById('launch-btn');
  elements.selectedVersion = document.getElementById('selected-version');
  elements.versionDropdownBtn = document.getElementById('version-dropdown-btn');
  elements.statusDot = document.getElementById('status-dot');
  elements.statusText = document.getElementById('status-text');
  elements.logsContent = document.getElementById('logs-content');
  elements.logsHeader = document.getElementById('logs-header');
  elements.toggleLogs = document.getElementById('toggle-logs');
  
  // Version modal elements
  elements.closeVersionModal = document.getElementById('close-version-modal');
  
  // Settings elements
  elements.settingsMemoryMin = document.getElementById('settings-memory-min');
  elements.settingsMemoryMax = document.getElementById('settings-memory-max');
  elements.settingsMemoryMinDisplay = document.getElementById('settings-memory-min-display');
  elements.settingsMemoryMaxDisplay = document.getElementById('settings-memory-max-display');
  elements.settingsGc = document.getElementById('settings-gc');
  elements.settingsConcurrent = document.getElementById('settings-concurrent');
  elements.settingsConcurrentDisplay = document.getElementById('settings-concurrent-display');
  elements.settingsTimeout = document.getElementById('settings-timeout');
  elements.settingsTimeoutDisplay = document.getElementById('settings-timeout-display');
  
  // Controls
  elements.settingsBtn = document.getElementById('settings-btn');
  elements.logoutBtn = document.getElementById('logout-btn');
  elements.closeSettingsBtn = document.getElementById('close-settings-btn');
  elements.saveSettingsBtn = document.getElementById('save-settings-btn');
}

function setupEventListeners() {
  // Login flow
  elements.loginBtn?.addEventListener('click', handleStartAuth);
  elements.completeAuthBtn?.addEventListener('click', handleCompleteAuth);
  
  // Main screen controls
  elements.launchBtn?.addEventListener('click', handleLaunch);
  elements.logoutBtn?.addEventListener('click', handleLogout);
  elements.settingsBtn?.addEventListener('click', showSettingsModal);
  
  // Version selection
  elements.versionDropdownBtn?.addEventListener('click', showVersionModal);
  elements.closeVersionModal?.addEventListener('click', hideVersionModal);
  
  // Logs are always visible (no toggle)
  
  // Settings controls
  elements.closeSettingsBtn?.addEventListener('click', hideSettingsModal);
  elements.saveSettingsBtn?.addEventListener('click', saveSettings);
  
  // Settings sliders
  elements.settingsMemoryMin?.addEventListener('input', updateMemoryMinDisplay);
  elements.settingsMemoryMax?.addEventListener('input', updateMemoryMaxDisplay);
  elements.settingsConcurrent?.addEventListener('input', updateConcurrentDisplay);
  elements.settingsTimeout?.addEventListener('input', updateTimeoutDisplay);
  
  // Log controls
  elements.clearLogsBtn?.addEventListener('click', clearLogs);
}

// Launcher initialization
async function initializeLauncher() {
  showLoading('Initializing Launcher...', 'Setting up the Minecraft launcher');
  
  try {
    // Resolve the path to the minecraft directory relative to the app's CWD
    const minecraftDir = await resolve('./minecraft');
    
    const result = await window.__TAURI__.core.invoke('initialize_launcher', {
      request: {
        minecraft_dir: minecraftDir,
        memory_min: state.settings.memoryMin,
        memory_max: state.settings.memoryMax,
        java_path: state.settings.javaPath || null,
        concurrent_downloads: state.settings.concurrentDownloads,
        download_timeout: state.settings.downloadTimeout
      }
    });
    
    if (result.success) {
      state.isInitialized = true;
      showNotification('Launcher initialized successfully!', 'success');
    } else {
      throw new Error(result.error);
    }
  } catch (error) {
    console.error('Failed to initialize launcher:', error);
    showNotification('Failed to initialize launcher: ' + error.message, 'error');
  } finally {
    hideLoading();
  }
}

// Authentication flow
async function handleStartAuth() {
  showLoading('Setting up Authentication...', 'Configuring Microsoft OAuth');
  
  try {
    // Setup authenticator
    const authResult = await invoke('setup_authenticator', {
      authConfig: {
        client_id: '00000000402b5328', // Microsoft Minecraft client ID (same as existing launcher)
        redirect_uri: 'https://login.live.com/oauth20_desktop.srf'
      }
    });
    
    if (!authResult.success) {
      throw new Error(authResult.error);
    }
    
    // Get auth URL
    const urlResult = await invoke('get_auth_url');
    
    if (urlResult.success) {
      const authUrl = urlResult.data;
      elements.manualAuthLink.href = authUrl;
      
      // Use the exact same approach as Microsoft.js but with manual URL entry
      console.log('Starting browser authentication...');
      
      // Show URL step directly
      const loginStep1 = document.getElementById('login-step-1');
      const urlAuthStep = document.getElementById('url-auth-step');
      const authUrlDisplay = document.getElementById('auth-url-display');
      const authUrlLink = document.getElementById('auth-url-link');
      
      if (loginStep1) loginStep1.classList.add('hidden');
      if (urlAuthStep) urlAuthStep.classList.remove('hidden');
      
      // Display the auth URL (same format as Microsoft.js)
      if (authUrlDisplay) authUrlDisplay.textContent = authUrl;
      if (authUrlLink) authUrlLink.href = authUrl;
      
      // Open browser automatically
      try {
        await open(authUrl);
        showNotification('Browser opened - complete login and copy the final URL', 'info');
      } catch (browserError) {
        console.warn('Could not open browser:', browserError);
        showNotification('Please manually open the URL shown below', 'warning');
      }
    } else {
      throw new Error(urlResult.error);
    }
  } catch (error) {
    console.error('Authentication setup failed:', error);
    showNotification('Authentication setup failed: ' + error.message, 'error');
  } finally {
    hideLoading();
  }
}

async function handleCompleteAuth() {
  const fullUrl = elements.authCodeInput.value.trim();
  
  if (!fullUrl) {
    showNotification('Please paste the complete URL from your browser', 'warning');
    return;
  }
  
  // Extract code the same way as Terminal.js: split("code=")[1].split("&")[0]
  let authCode;
  try {
    if (fullUrl.includes('code=')) {
      authCode = fullUrl.split("code=")[1].split("&")[0];
    } else {
      throw new Error('No code found in URL');
    }
  } catch (error) {
    showNotification('Invalid URL format. Please paste the complete URL that contains "code="', 'error');
    return;
  }
  
  if (!authCode) {
    showNotification('Could not extract authorization code from URL', 'error');
    return;
  }
  
  console.log('Extracted authorization code:', authCode);
  await completeAuthWithCode(authCode);
}



async function completeAuthWithCode(authCode) {
  showLoading('Completing Authentication...', 'Verifying with Microsoft');
  
  try {
    const result = await invoke('authenticate_with_code', { authCode });
    
    if (result.success) {
      state.isLoggedIn = true;
      state.currentAccount = result.data;
      
      // Save account data
      localStorage.setItem('minecraft-account', JSON.stringify(result.data));
      
      showNotification(`Welcome, ${result.data.name}!`, 'success');
      await initializeLauncher(); // Initialize launcher after login
      await loadVersions();
      showMainScreen();
    } else {
      throw new Error(result.error);
    }
  } catch (error) {
    console.error('Authentication failed:', error);
    showNotification('Authentication failed: ' + error.message, 'error');
    
    // Show manual input as fallback
    const loginStep2 = document.getElementById('login-step-2');
    const loginStep3 = document.getElementById('login-step-3');
    
    if (loginStep2) loginStep2.classList.add('hidden');
    if (loginStep3) loginStep3.classList.remove('hidden');
  } finally {
    hideLoading();
  }
}

// Version management
async function loadVersions() {
  try {
    updateStatus('loading', 'Loading Minecraft versions...');
    
    // ViaFabricPlus compatibility versions (all run on 1.21.4 base with backward compatibility)
    const hardcodedVersions = [
      { id: '1.21.8', type: 'release', releaseTime: '2025-01-15T12:00:00Z', description: 'Latest (Trial Chambers Update)', recommended: true },
      { id: '1.21', type: 'release', releaseTime: '2024-06-13T12:00:00Z', description: 'Trial Chambers' },
      { id: '1.20', type: 'release', releaseTime: '2023-06-07T12:00:00Z', description: 'Camels & Archaeology' },
      { id: '1.19', type: 'release', releaseTime: '2022-06-07T12:00:00Z', description: 'Caves & Cliffs II' },
      { id: '1.18', type: 'release', releaseTime: '2021-11-30T12:00:00Z', description: 'Caves & Cliffs I' },
      { id: '1.17', type: 'release', releaseTime: '2021-06-08T12:00:00Z', description: 'Caves & Cliffs' },
      { id: '1.16', type: 'release', releaseTime: '2020-06-23T12:00:00Z', description: 'Nether Update' },
      { id: '1.15', type: 'release', releaseTime: '2019-12-10T12:00:00Z', description: 'Buzzy Bees' },
      { id: '1.14', type: 'release', releaseTime: '2019-04-23T12:00:00Z', description: 'Village & Pillage (First Fabric)' },
      { id: '1.13', type: 'release', releaseTime: '2018-07-18T12:00:00Z', description: 'Aquatic Update' },
      { id: '1.12', type: 'release', releaseTime: '2017-06-07T12:00:00Z', description: 'World of Color (Popular Modding)', popular: true },
      { id: '1.11', type: 'release', releaseTime: '2016-11-14T12:00:00Z', description: 'Exploration Update' },
      { id: '1.10', type: 'release', releaseTime: '2016-06-08T12:00:00Z', description: 'Frostburn Update' },
      { id: '1.9', type: 'release', releaseTime: '2016-02-29T12:00:00Z', description: 'Combat Update (PvP Changes)' },
      { id: '1.8.9', type: 'release', releaseTime: '2015-12-09T12:00:00Z', description: 'Bountiful Update (Recommended)', recommended: true },
      { id: '1.8', type: 'release', releaseTime: '2014-09-02T12:00:00Z', description: 'Bountiful Update' }
    ];
    
    state.versions = hardcodedVersions;
    populateVersionModal();
    
    // Set default version in dropdown
    if (elements.selectedVersion) {
      elements.selectedVersion.textContent = '1.8.9';
    }
    
    updateStatus('online', 'Ready to launch');
    showNotification('Client ready to launch', 'success');
  } catch (error) {
    console.error('Failed to load versions:', error);
    showNotification('Failed to load versions: ' + error.message, 'error');
    updateStatus('error', 'Failed to load versions');
  }
}

// New UI functions
function populateVersionModal() {
  const versionList = document.querySelector('.version-list');
  if (!versionList) return;
  
  versionList.innerHTML = '';
  
  state.versions.forEach(version => {
    const versionItem = document.createElement('div');
    versionItem.className = 'version-item';
    versionItem.dataset.version = version.id;
    
    // Add special styling for recommended/popular versions
    if (version.recommended) {
      versionItem.classList.add('recommended');
    }
    if (version.popular) {
      versionItem.classList.add('popular');
    }
    
    // Default to 1.8.9 as selected (recommended)
    if (version.id === '1.8.9') {
      versionItem.classList.add('selected');
    }
    
    // Create badges for special versions
    let badges = '';
    if (version.recommended) {
      badges += '<span class="version-badge recommended-badge">Recommended</span>';
    }
    if (version.popular) {
      badges += '<span class="version-badge popular-badge">Popular</span>';
    }
    
    versionItem.innerHTML = `
      <div class="version-info">
        <div class="version-header">
          <span class="version-name">${version.id}</span>
          ${badges}
        </div>
        <span class="version-description">${version.description || version.type}</span>
      </div>
      <span class="version-status">${version.id === '1.8.9' ? '✓' : ''}</span>
    `;
    
    versionItem.addEventListener('click', () => selectVersion(version.id));
    versionList.appendChild(versionItem);
  });
}

function selectVersion(versionId) {
  // Update selected version display
  if (elements.selectedVersion) {
    elements.selectedVersion.textContent = versionId;
  }
  
  // Update version modal selection
  document.querySelectorAll('.version-item').forEach(item => {
    item.classList.remove('selected');
    item.querySelector('.version-status').textContent = '';
  });
  
  const selectedItem = document.querySelector(`[data-version="${versionId}"]`);
  if (selectedItem) {
    selectedItem.classList.add('selected');
    selectedItem.querySelector('.version-status').textContent = '✓';
  }
  
  hideVersionModal();
  showNotification(`Selected Minecraft ${versionId}`, 'success');
}

function showVersionModal() {
  if (elements.versionModal) {
    elements.versionModal.classList.remove('hidden');
  }
}

function hideVersionModal() {
  if (elements.versionModal) {
    elements.versionModal.classList.add('hidden');
  }
}

// Logs are always visible - toggle function removed

// Settings slider update functions
function updateMemoryMinDisplay() {
  const value = elements.settingsMemoryMin?.value;
  if (elements.settingsMemoryMinDisplay && value) {
    elements.settingsMemoryMinDisplay.textContent = `${value} GB`;
    
    // Ensure max is always >= min
    if (elements.settingsMemoryMax && parseInt(elements.settingsMemoryMax.value) < parseInt(value)) {
      elements.settingsMemoryMax.value = value;
      updateMemoryMaxDisplay();
    }
  }
}

function updateMemoryMaxDisplay() {
  const value = elements.settingsMemoryMax?.value;
  if (elements.settingsMemoryMaxDisplay && value) {
    elements.settingsMemoryMaxDisplay.textContent = `${value} GB`;
  }
}

function updateConcurrentDisplay() {
  const value = elements.settingsConcurrent?.value;
  if (elements.settingsConcurrentDisplay && value) {
    elements.settingsConcurrentDisplay.textContent = value;
  }
}

function updateTimeoutDisplay() {
  const value = elements.settingsTimeout?.value;
  if (elements.settingsTimeoutDisplay && value) {
    elements.settingsTimeoutDisplay.textContent = `${value}s`;
  }
}

function hideSettingsModal() {
  if (elements.settingsModal) {
    elements.settingsModal.classList.add('hidden');
  }
}


function populateVersionSelect() {
  elements.versionSelect.innerHTML = '<option value="">Select version...</option>';
  
  // Group versions by type
  const releases = state.versions.filter(v => v.type === 'release');
  const snapshots = state.versions.filter(v => v.type === 'snapshot');
  
  // Add releases
  if (releases.length > 0) {
    const releaseGroup = document.createElement('optgroup');
    releaseGroup.label = 'Releases';
    releases.slice(0, 20).forEach(version => {
      const option = document.createElement('option');
      option.value = version.id;
      option.textContent = version.id;
      releaseGroup.appendChild(option);
    });
    elements.versionSelect.appendChild(releaseGroup);
  }
  
  // Enable launch button if version is selected
  elements.versionSelect.addEventListener('change', () => {
    elements.launchBtn.disabled = !elements.versionSelect.value;
  });
}

// Minecraft launching
async function handleLaunch() {
  const selectedVersion = elements.selectedVersion?.textContent || '1.8.9';
  
  try {
    // Get a valid Java runtime for the selected version, downloading if necessary.
    updateStatus('checking-java', `Verifying Java for ${selectedVersion}...`);
    showLoading('Verifying Java Runtime', `Checking for a compatible Java installation...`);

    let javaResult = await invoke('get_java_runtime', {
        version: selectedVersion,
    });

    // If Java is not found, it will be downloaded automatically
    if (!javaResult.success && javaResult.error.includes('download')) {
        updateStatus('downloading-java', `Downloading Java Runtime...`);
        showLoading('Downloading Java Runtime', `Installing Java for Minecraft ${selectedVersion}...`);
        appendLog(`📦 Downloading Java runtime for ${selectedVersion}...`);
        
        // The download happens automatically, so we just need to wait and check again
        await new Promise(resolve => setTimeout(resolve, 2000)); // Give it a moment
        
        javaResult = await invoke('get_java_runtime', {
            version: selectedVersion,
        });
    }

    if (!javaResult.success) {
      throw new Error(`Failed to get Java runtime: ${javaResult.error}`);
    }

    const javaPath = javaResult.data.path;
    appendLog(`✅ Using Java runtime at: ${javaPath}`);

    // Now, launch Minecraft with the obtained Java path.
    updateStatus('launching', `Launching Minecraft ${selectedVersion}...`);
    showLoading('Launching Minecraft Client...', `Starting Minecraft ${selectedVersion}`);
    
    // Launch the selected version directly
    const result = await invoke('launch_minecraft', {
      request: {
        version: selectedVersion, // Use the actual selected version
        instance: 'Fabric1214',
        memory_min: Math.floor(state.settings.memoryMin / 1024),
        memory_max: Math.floor(state.settings.memoryMax / 1024),
        gc_type: state.settings.gcType || 'G1GC',
        account: state.currentAccount, // Pass the logged-in account
        java_path: javaPath // Use the path returned from the backend
      }
    });
    
    if (result.success) {
      updateStatus('online', `Minecraft is running (${selectedVersion})`);
      showNotification(`Minecraft ${selectedVersion} launched successfully!`, 'success');
      
      // Get the actual PID for display purposes
      try {
        const statusResult = await invoke('get_process_status', {
          processId: result.data
        });
        const displayPid = statusResult.success && statusResult.data.pid ? statusResult.data.pid : result.data;
        appendLog(`✅ Minecraft ${selectedVersion} started successfully (Process ID: ${displayPid})`);
      } catch (error) {
        // Fallback to showing the internal ID if PID fetch fails
        appendLog(`✅ Minecraft ${selectedVersion} started successfully (Process ID: ${result.data})`);
      }
      
      // Update UI to show running state
      updateLaunchButtonState(true, selectedVersion, result.data);
      
      // Start monitoring the process
      monitorMinecraftProcess(result.data, selectedVersion);
    } else {
      throw new Error(result.error || 'Launch failed');
    }
  } catch (error) {
    console.error('Launch failed:', error);
    updateStatus('error', 'Launch failed');
    showNotification('Failed to launch Minecraft: ' + error.message, 'error');
    appendLog('❌ Launch failed: ' + error.message);
  } finally {
    hideLoading();
  }
}

// Minecraft killing
async function handleKill() {
  if (!state.currentProcess) {
    appendLog('❌ No process to kill');
    return;
  }

  try {
    updateStatus('killing', 'Terminating Minecraft...');
    showLoading('Stopping Minecraft', 'Terminating the game process...');
    
    const result = await invoke('kill_minecraft', {
      processId: state.currentProcess
    });

    if (result.success) {
      updateStatus('offline', 'Minecraft terminated');
      showNotification('Minecraft terminated successfully', 'success');
      appendLog('✅ Minecraft process terminated');
      
      // Reset UI to launch state
      updateLaunchButtonState(false);
      
      // Clear process monitoring
      if (state.processMonitorInterval) {
        clearInterval(state.processMonitorInterval);
        state.processMonitorInterval = null;
      }
    } else {
      throw new Error(result.error || 'Kill failed');
    }
  } catch (error) {
    console.error('Kill failed:', error);
    updateStatus('error', 'Kill failed');
    showNotification('Failed to terminate Minecraft: ' + error.message, 'error');
    appendLog('❌ Kill failed: ' + error.message);
  } finally {
    hideLoading();
  }
}

// Process monitoring
async function monitorMinecraftProcess(processId, version) {
  // Check every 5 seconds if the process is still running
  const checkInterval = setInterval(async () => {
    try {
      const result = await invoke('get_process_status', {
        processId: processId
      });
      
      if (!result.success || !result.data.is_running) {
        // Process has ended
        clearInterval(checkInterval);
        updateStatus('offline', 'Minecraft has closed');
        appendLog(`🔄 Minecraft ${version} process ended`);
        updateLaunchButtonState(false);
      }
    } catch (error) {
      // Error checking status, assume process ended
      clearInterval(checkInterval);
      updateStatus('offline', 'Minecraft connection lost');
      appendLog(`❌ Lost connection to Minecraft process`);
      updateLaunchButtonState(false);
    }
  }, 5000); // Check every 5 seconds
  
  // Store the interval ID so we can clear it if needed
  state.processMonitorInterval = checkInterval;
}

// UI helpers
function showLoginScreen() {
  elements.loginScreen.classList.remove('hidden');
  elements.mainScreen.classList.add('hidden');
}

function updateLaunchButtonState(isRunning, version, processData) {
  if (isRunning) {
    // Change button to "Kill Minecraft" and disable version selector
    elements.launchBtn.textContent = 'Kill Minecraft';
    elements.launchBtn.classList.add('kill-btn');
    elements.versionDropdownBtn.style.pointerEvents = 'none';
    elements.versionDropdownBtn.style.opacity = '0.5';
    
    // Store process data for killing later
    state.currentProcess = processData;
    
    // Change click handler to kill instead of launch
    elements.launchBtn.removeEventListener('click', handleLaunch);
    elements.launchBtn.addEventListener('click', handleKill);
  } else {
    // Reset button to "Launch Minecraft" and enable version selector
    elements.launchBtn.textContent = 'Launch Minecraft';
    elements.launchBtn.classList.remove('kill-btn');
    elements.versionDropdownBtn.style.pointerEvents = 'auto';
    elements.versionDropdownBtn.style.opacity = '1';
    
    // Clear process data
    state.currentProcess = null;
    
    // Change click handler back to launch
    elements.launchBtn.removeEventListener('click', handleKill);
    elements.launchBtn.addEventListener('click', handleLaunch);
  }
}

function showMainScreen() {
  elements.loginScreen.classList.add('hidden');
  elements.mainScreen.classList.remove('hidden');
  
  if (state.currentAccount) {
    elements.userInfo.textContent = `👤 ${state.currentAccount.name}`;
  }
}

function showLoading(title, message) {
  document.getElementById('loading-title').textContent = title;
  document.getElementById('loading-message').textContent = message;
  elements.loadingOverlay.classList.remove('hidden');
}

function hideLoading() {
  elements.loadingOverlay.classList.add('hidden');
}

function updateStatus(type, message) {
  elements.statusDot.className = `status-dot ${type}`;
  elements.statusText.textContent = message;
}

function showNotification(message, type = 'info') {
  const notification = document.createElement('div');
  notification.className = `notification ${type}`;
  notification.textContent = message;
  
  const container = document.getElementById('notifications') || createNotificationContainer();
  container.appendChild(notification);
  
  setTimeout(() => {
    notification.remove();
  }, 5000);
}

function createNotificationContainer() {
  const container = document.createElement('div');
  container.id = 'notifications';
  container.className = 'notifications-container';
  document.body.appendChild(container);
  return container;
}

// Settings management
function loadSettings() {
  const saved = localStorage.getItem('minecraft-launcher-settings');
  if (saved) {
    Object.assign(state.settings, JSON.parse(saved));
    applySettings();
  }
}

function applySettings() {
  // Apply memory settings
  if (elements.memorySlider) {
    elements.memorySlider.value = state.settings.memoryMax;
    updateMemoryDisplay();
  }
}

function updateMemoryDisplay() {
  const value = elements.memorySlider.value;
  const gb = (value / 1024).toFixed(1);
  elements.memoryDisplay.textContent = `${gb}GB`;
  state.settings.memoryMax = parseInt(value);
}

function showSettingsModal() {
  // Create settings modal content
  const modalHtml = `
    <div class="modal-overlay" id="settings-modal-overlay">
      <div class="modal-content">
        <div class="modal-header">
          <h2>🔧 Launcher Settings</h2>
          <button class="modal-close" id="close-settings">&times;</button>
        </div>
        <div class="modal-body">
          
          <div class="settings-section">
            <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z"/></svg> Memory Settings</h3>
            <div class="memory-setting">
              <label>Minimum Memory: <span id="memory-min-display">${Math.floor(state.settings.memoryMin / 1024)}GB</span></label>
              <input type="range" id="settings-memory-min" min="1" max="16" step="1" value="${Math.floor(state.settings.memoryMin / 1024)}" oninput="updateMemoryMinSlider()">
            </div>
            <div class="memory-setting">
              <label>Maximum Memory: <span id="memory-max-display">${Math.floor(state.settings.memoryMax / 1024)}GB</span></label>
              <input type="range" id="settings-memory-max" min="2" max="32" step="1" value="${Math.floor(state.settings.memoryMax / 1024)}" oninput="updateMemoryMaxSlider()">
            </div>
          </div>
          
          <div class="settings-section">
            <h3>☕ Java Settings</h3>
            <label>Java Path (optional): <input type="text" id="settings-java-path" placeholder="Auto-detect" value="${state.settings.javaPath || ''}"></label>
            <button id="browse-java-path">Browse</button>
          </div>
          
          <div class="settings-section">
            <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M9 3V4H4V6H5V19C5 20.1 5.9 21 7 21H17C18.1 21 19 20.1 19 19V6H20V4H15V3H9M7 6H17V19H7V6M9 8V17H11V8H9M13 8V17H15V8H13Z"/></svg> Garbage Collector</h3>
            <div class="gc-flags">
              <label><input type="radio" name="gc-type" value="G1GC" ${(state.settings.gcType || 'G1GC') === 'G1GC' ? 'checked' : ''}> G1 Garbage Collector (Recommended)</label>
              <label><input type="radio" name="gc-type" value="ZGC" ${state.settings.gcType === 'ZGC' ? 'checked' : ''}> ZGC (Java 17+)</label>
              <label><input type="radio" name="gc-type" value="ShenandoahGC" ${state.settings.gcType === 'ShenandoahGC' ? 'checked' : ''}> Shenandoah GC</label>
              <label><input type="radio" name="gc-type" value="ParallelGC" ${state.settings.gcType === 'ParallelGC' ? 'checked' : ''}> Parallel GC</label>
            </div>
          </div>
          
          <div class="settings-section">
            <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.22,8.95 2.27,9.22 2.46,9.37L4.57,11C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.22,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.68 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z"/></svg> Advanced JVM Arguments</h3>
            <textarea id="settings-custom-jvm-args" placeholder="Enter custom JVM arguments (one per line)">${(state.settings.customJvmArgs || []).join('\n')}</textarea>
          </div>
          
          <div class="settings-section">
            <h3><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M5,20H19V18H5M19,9H15V3H9V9H5L12,16L19,9Z"/></svg> Download Settings</h3>
            <label>Concurrent Downloads: <input type="number" id="settings-concurrent-downloads" value="${state.settings.concurrentDownloads}" min="1" max="16"></label>
            <label>Download Timeout (seconds): <input type="number" id="settings-download-timeout" value="${state.settings.downloadTimeout}" min="30" max="600"></label>
          </div>
        </div>
        <div class="modal-footer">
          <button id="save-settings" class="btn-primary"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M15,9H5V5H15M12,19A3,3 0 0,1 9,16A3,3 0 0,1 12,13A3,3 0 0,1 15,16A3,3 0 0,1 12,19M17,3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V7L17,3Z"/></svg> Save Settings</button>
          <button id="cancel-settings" class="btn-secondary"><svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z"/></svg> Cancel</button>
        </div>
      </div>
    </div>
  `;
  
  // Add modal to page
  document.body.insertAdjacentHTML('beforeend', modalHtml);
  
  // Add event listeners
  document.getElementById('close-settings').addEventListener('click', closeSettingsModal);
  document.getElementById('cancel-settings').addEventListener('click', closeSettingsModal);
  document.getElementById('save-settings').addEventListener('click', saveSettings);
  
  // Close on overlay click
  document.getElementById('settings-modal-overlay').addEventListener('click', (e) => {
    if (e.target.id === 'settings-modal-overlay') {
      closeSettingsModal();
    }
  });
}

function closeSettingsModal() {
  const modal = document.getElementById('settings-modal-overlay');
  if (modal) {
    modal.remove();
  }
}

async function saveSettings() {
  try {
    // Get values from form
    const memoryMin = parseInt(document.getElementById('settings-memory-min').value) * 1024; // Convert GB to MB
    const memoryMax = parseInt(document.getElementById('settings-memory-max').value) * 1024; // Convert GB to MB
    const javaPath = document.getElementById('settings-java-path')?.value?.trim() || '';
    const concurrentDownloads = parseInt(document.getElementById('settings-concurrent-downloads').value);
    const downloadTimeout = parseInt(document.getElementById('settings-download-timeout').value);
    const customJvmArgs = document.getElementById('settings-custom-jvm-args').value
      .split('\n')
      .map(arg => arg.trim())
      .filter(arg => arg.length > 0);
    
    // Get selected GC type
    const gcRadio = document.querySelector('.gc-flags input[type="radio"]:checked');
    const gcType = gcRadio ? gcRadio.value : 'G1GC';
    
    // Validate
    if (memoryMin >= memoryMax) {
      showNotification('Maximum memory must be greater than minimum memory', 'error');
      return;
    }
    
    // Update state
    state.settings = {
      ...state.settings,
      memoryMin,
      memoryMax,
      javaPath,
      concurrentDownloads,
      downloadTimeout,
      gcType,
      customJvmArgs
    };
    
    // Save to localStorage
    localStorage.setItem('minecraft-launcher-settings', JSON.stringify(state.settings));
    
    showNotification('Settings saved successfully!', 'success');
    closeSettingsModal();
  } catch (error) {
    console.error('Failed to save settings:', error);
    showNotification('Failed to save settings: ' + error.message, 'error');
  }
}

// Browse functions removed - minecraft dir setting removed, java path is optional

function handleLogout() {
  state.isLoggedIn = false;
  state.currentAccount = null;
  state.activeProcesses = [];
  showLoginScreen();
  showNotification('Logged out successfully', 'info');
}

function clearLogs() {
  elements.logsContent.innerHTML = '<div class="log-welcome">Logs cleared 🧹</div>';
}

// Utility functions
async function getHomeDirectory() {
  try {
    const result = await invoke('get_home_directory');
    if (result.success) {
      return result.data;
    } else {
      console.warn('Failed to get home directory:', result.error);
      // Fallback to platform-specific default
      const platform = navigator.platform.toLowerCase();
      if (platform.includes('mac') || platform.includes('darwin')) {
        return '/Users/user';
      } else if (platform.includes('win')) {
        return 'C:\\Users\\user';
      } else {
        return '/home/user';
      }
    }
  } catch (error) {
    console.warn('Error getting home directory:', error);
    // Fallback to platform-specific default
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('mac') || platform.includes('darwin')) {
      return '/Users/user';
    } else if (platform.includes('win')) {
      return 'C:\\Users\\user';
    } else {
      return '/home/user';
    }
  }
}

// Logging function
function appendLog(message, type = 'info') {
  const logsContent = elements.logsContent;
  if (!logsContent) return;
  
  const timestamp = new Date().toLocaleTimeString();
  const logEntry = document.createElement('div');
  logEntry.className = `log-entry log-${type}`;
  logEntry.innerHTML = `<span class="log-timestamp">[${timestamp}]</span> ${message}`;
  
  logsContent.appendChild(logEntry);
  logsContent.scrollTop = logsContent.scrollHeight;
}

// Memory slider update functions for settings modal
function updateMemoryMinSlider() {
  const slider = document.getElementById('settings-memory-min');
  const display = document.getElementById('memory-min-display');
  if (slider && display) {
    const value = parseInt(slider.value);
    display.textContent = `${value}GB`;
    
    // Ensure max is always >= min
    const maxSlider = document.getElementById('settings-memory-max');
    if (maxSlider && parseInt(maxSlider.value) < value) {
      maxSlider.value = value;
      updateMemoryMaxSlider();
    }
  }
}

function updateMemoryMaxSlider() {
  const slider = document.getElementById('settings-memory-max');
  const display = document.getElementById('memory-max-display');
  if (slider && display) {
    const value = parseInt(slider.value);
    display.textContent = `${value}GB`;
  }
}

// Set up backend log listener
async function setupLogListener() {
  try {
    await listen('launcher-log', (event) => {
      const logData = event.payload;
      const level = logData.level || 'info';
      const message = logData.message || 'Unknown log message';
      
      // Map backend log levels to frontend types
      let logType = 'info';
      if (level === 'success') logType = 'success';
      else if (level === 'error') logType = 'error';
      else if (level === 'warn') logType = 'warning';
      
      appendLog(message, logType);
    });
    console.log('✅ Backend log listener setup complete');
  } catch (error) {
    console.warn('Failed to setup backend log listener:', error);
  }
}

console.log('Minecraft Launcher JavaScript loaded!');
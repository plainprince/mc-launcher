// NW.js Minecraft Launcher Renderer
const path = require('path');
const fs = require('fs');

// Import launcher components
const MinecraftLauncher = require('./launcher-wrapper');
const SettingsManager = require('./settings');

class LauncherApp {
    constructor() {
        this.launcher = MinecraftLauncher.launcher;
        this.settingsManager = new SettingsManager();
        this.isAuthenticated = false;
        this.currentAccount = null;
        this.gameState = {
            isRunning: false,
            isLaunching: false,
            progress: 0
        };

        this.initializeApp();
        this.setupEventListeners();
        this.checkAuthentication();
        this.loadModList();
    }

    initializeApp() {
        console.log('Initializing Minecraft Launcher...');
        
        // Setup window controls
        this.setupWindowControls();
        
        // Setup tab navigation
        this.setupTabNavigation();
        
        // Setup launcher controls
        this.setupLauncherControls();
        
        // Setup console
        this.setupConsole();
        
        // Update status
        this.updateStatus('Ready');
    }

    setupWindowControls() {
        const minimizeBtn = document.getElementById('minimizeBtn');
        const closeBtn = document.getElementById('closeBtn');
        
        minimizeBtn.addEventListener('click', () => {
            nw.Window.get().minimize();
        });
        
        closeBtn.addEventListener('click', () => {
            this.handleAppClose();
        });
    }

    setupTabNavigation() {
        const navItems = document.querySelectorAll('.nav-item');
        const tabContents = document.querySelectorAll('.tab-content');
        
        navItems.forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                const targetTab = item.getAttribute('data-tab');
                
                // Update active nav item
                navItems.forEach(nav => nav.classList.remove('active'));
                item.classList.add('active');
                
                // Update active tab content
                tabContents.forEach(tab => tab.classList.remove('active'));
                document.getElementById(targetTab).classList.add('active');
                
                // Refresh content based on tab
                if (targetTab === 'logs') {
                    this.refreshLogs();
                } else if (targetTab === 'mods') {
                    this.loadModList();
                }
            });
        });
    }

    setupLauncherControls() {
        const launchBtn = document.getElementById('launchBtn');
        const killBtn = document.getElementById('killBtn');
        const loginBtn = document.getElementById('loginBtn');
        const settingsBtn = document.getElementById('settingsBtn');
        
        launchBtn.addEventListener('click', () => this.launchGame());
        killBtn.addEventListener('click', () => this.killGame());
        loginBtn.addEventListener('click', () => this.handleLogin());
        settingsBtn.addEventListener('click', () => this.openSettings());
        
        // Quick action buttons
        document.getElementById('openInstanceBtn').addEventListener('click', () => {
            this.openInstanceFolder();
        });
        
        document.getElementById('viewLogsBtn').addEventListener('click', () => {
            // Switch to logs tab
            document.querySelector('[data-tab="logs"]').click();
        });
        
        document.getElementById('manageModsBtn').addEventListener('click', () => {
            // Switch to mods tab
            document.querySelector('[data-tab="mods"]').click();
        });
        
        document.getElementById('refreshModsBtn').addEventListener('click', () => {
            this.loadModList();
        });
        
        document.getElementById('refreshLogsBtn').addEventListener('click', () => {
            this.refreshLogs();
        });
        
        document.getElementById('clearLogsBtn').addEventListener('click', () => {
            this.clearLogs();
        });
    }

    setupConsole() {
        const toggleBtn = document.getElementById('toggleConsoleBtn');
        const consoleEl = document.getElementById('console');
        
        toggleBtn.addEventListener('click', () => {
            consoleEl.classList.toggle('collapsed');
            toggleBtn.textContent = consoleEl.classList.contains('collapsed') ? '+' : '−';
        });
        
        // Setup launcher event listeners
        this.launcher.on('data', (data) => {
            this.addConsoleOutput(data.toString(), 'info');
        });
        
        this.launcher.on('progress', (progress, total, element) => {
            const percent = Math.round((progress / total) * 100);
            this.updateProgress(percent, `Downloading: ${element}`);
        });
        
        this.launcher.on('error', (error) => {
            this.addConsoleOutput(`Error: ${error.error || error}`, 'error');
            this.updateStatus('Error', 'error');
            this.gameState.isLaunching = false;
            this.updateUI();
        });
        
        this.launcher.on('close', () => {
            this.addConsoleOutput('Game closed', 'info');
            this.updateStatus('Ready');
            this.gameState.isRunning = false;
            this.gameState.isLaunching = false;
            this.updateUI();
        });
    }

    async checkAuthentication() {
        const accountPath = path.join(__dirname, 'account.json');
        
        if (fs.existsSync(accountPath)) {
            try {
                const account = JSON.parse(fs.readFileSync(accountPath, 'utf-8'));
                this.currentAccount = account;
                this.isAuthenticated = true;
                this.updateProfileUI(account);
                this.addConsoleOutput('Authentication loaded from cache', 'success');
            } catch (error) {
                this.addConsoleOutput('Failed to load cached authentication', 'warning');
            }
        }
        
        this.updateUI();
    }

    async handleLogin() {
        try {
            this.addConsoleOutput('Starting authentication...', 'info');
            this.updateStatus('Authenticating...', 'running');
            
            const account = await MinecraftLauncher.login(true, false);
            this.currentAccount = account;
            this.isAuthenticated = true;
            this.updateProfileUI(account);
            this.addConsoleOutput('Authentication successful!', 'success');
            this.updateStatus('Ready');
            this.updateUI();
            
        } catch (error) {
            this.addConsoleOutput(`Authentication failed: ${error.message}`, 'error');
            this.updateStatus('Authentication failed', 'error');
        }
    }

    async launchGame() {
        if (this.gameState.isRunning || this.gameState.isLaunching) {
            return;
        }
        
        if (!this.isAuthenticated) {
            this.addConsoleOutput('Please login first', 'warning');
            return;
        }
        
        try {
            this.gameState.isLaunching = true;
            this.updateStatus('Launching...', 'running');
            this.updateUI();
            this.showProgress();
            
            // Get settings for launch options
            const settings = this.settingsManager.getSettings();
            const launchOptions = this.settingsManager.toLauncherOptions();
            
            // Configure for Fabric 1.21.4
            launchOptions.version = "1.21.4";
            launchOptions.instance = "Fabric1214";
            launchOptions.workingDirectory = path.resolve(__dirname, "../minecraft");
            
            this.addConsoleOutput('Starting Minecraft with Fabric 1.21.4...', 'info');
            
            const gameControls = await MinecraftLauncher.launch({
                ...launchOptions,
                quiet: false,
                downloadQuiet: false
            });
            
            this.gameState.isRunning = true;
            this.gameState.isLaunching = false;
            this.updateStatus('Running', 'running');
            this.hideProgress();
            this.updateUI();
            this.addConsoleOutput('Game launched successfully!', 'success');
            
        } catch (error) {
            this.addConsoleOutput(`Launch failed: ${error.message}`, 'error');
            this.updateStatus('Launch failed', 'error');
            this.gameState.isLaunching = false;
            this.hideProgress();
            this.updateUI();
        }
    }

    async killGame() {
        if (!this.gameState.isRunning) {
            return;
        }
        
        try {
            this.addConsoleOutput('Stopping game...', 'info');
            await MinecraftLauncher.kill(false);
            this.addConsoleOutput('Game stopped', 'success');
        } catch (error) {
            this.addConsoleOutput(`Failed to stop game: ${error.message}`, 'error');
        }
    }

    openInstanceFolder() {
        const instancePath = path.resolve(__dirname, "../minecraft/instances/Fabric1214");
        const os = require('os');
        const { exec } = require('child_process');
        
        let command;
        switch (os.platform()) {
            case 'darwin':
                command = `open "${instancePath}"`;
                break;
            case 'win32':
                command = `explorer "${instancePath}"`;
                break;
            default:
                command = `xdg-open "${instancePath}"`;
        }
        
        exec(command, (error) => {
            if (error) {
                this.addConsoleOutput(`Failed to open instance folder: ${error.message}`, 'error');
            } else {
                this.addConsoleOutput('Opened instance folder', 'info');
            }
        });
    }

    loadModList() {
        const modsPath = path.resolve(__dirname, "../minecraft/instances/Fabric1214/mods");
        const modsList = document.getElementById('modsList');
        const modCount = document.getElementById('modCount');
        
        try {
            if (!fs.existsSync(modsPath)) {
                modsList.innerHTML = '<p>Mods folder not found</p>';
                modCount.textContent = '0';
                return;
            }
            
            const modFiles = fs.readdirSync(modsPath)
                .filter(file => file.endsWith('.jar'))
                .sort();
            
            modCount.textContent = modFiles.length.toString();
            
            if (modFiles.length === 0) {
                modsList.innerHTML = '<p>No mods installed</p>';
                return;
            }
            
            modsList.innerHTML = modFiles.map(modFile => {
                const modName = modFile.replace('.jar', '');
                const firstLetter = modName.charAt(0).toUpperCase();
                
                return `
                    <div class="mod-item">
                        <div class="mod-info">
                            <div class="mod-icon">${firstLetter}</div>
                            <div class="mod-details">
                                <h4>${modName}</h4>
                                <p>${modFile}</p>
                            </div>
                        </div>
                        <div class="mod-actions">
                            <button class="btn btn-secondary btn-small" onclick="app.openModFile('${modFile}')">📁</button>
                        </div>
                    </div>
                `;
            }).join('');
            
        } catch (error) {
            modsList.innerHTML = '<p>Error loading mods</p>';
            modCount.textContent = '?';
            this.addConsoleOutput(`Failed to load mods: ${error.message}`, 'error');
        }
    }

    openModFile(modFile) {
        const modPath = path.resolve(__dirname, "../minecraft/instances/Fabric1214/mods", modFile);
        const { exec } = require('child_process');
        const os = require('os');
        
        let command;
        const dir = path.dirname(modPath);
        
        switch (os.platform()) {
            case 'darwin':
                command = `open -R "${modPath}"`;
                break;
            case 'win32':
                command = `explorer /select,"${modPath}"`;
                break;
            default:
                command = `xdg-open "${dir}"`;
        }
        
        exec(command, (error) => {
            if (error) {
                this.addConsoleOutput(`Failed to open mod file: ${error.message}`, 'error');
            }
        });
    }

    refreshLogs() {
        try {
            const logs = MinecraftLauncher.inspectLogs();
            document.getElementById('logsContent').textContent = logs;
        } catch (error) {
            document.getElementById('logsContent').textContent = `Error loading logs: ${error.message}`;
        }
    }

    clearLogs() {
        document.getElementById('logsContent').textContent = 'Logs cleared.';
    }

    openSettings() {
        // For now, show a simple alert. In a full implementation,
        // this would open the settings modal with the settings manager
        this.addConsoleOutput('Settings functionality coming soon...', 'info');
    }

    updateProfileUI(account) {
        const profileName = document.getElementById('profileName');
        const profileStatus = document.getElementById('profileStatus');
        const profileInitials = document.getElementById('profileInitials');
        const loginBtn = document.getElementById('loginBtn');
        
        profileName.textContent = account.name || 'Player';
        profileStatus.textContent = 'Online';
        profileInitials.textContent = (account.name || 'P').charAt(0).toUpperCase();
        loginBtn.textContent = 'Logged In';
        loginBtn.disabled = true;
    }

    updateStatus(text, type = '') {
        const statusText = document.getElementById('statusText');
        const statusDot = document.getElementById('statusDot');
        
        statusText.textContent = text;
        statusDot.className = 'status-dot';
        
        if (type) {
            statusDot.classList.add(type);
        }
    }

    updateUI() {
        const launchBtn = document.getElementById('launchBtn');
        const killBtn = document.getElementById('killBtn');
        
        if (this.gameState.isRunning) {
            launchBtn.classList.add('hidden');
            killBtn.classList.remove('hidden');
        } else {
            launchBtn.classList.remove('hidden');
            killBtn.classList.add('hidden');
        }
        
        launchBtn.disabled = this.gameState.isLaunching || this.gameState.isRunning || !this.isAuthenticated;
        killBtn.disabled = !this.gameState.isRunning;
    }

    showProgress() {
        document.getElementById('progressSection').classList.remove('hidden');
    }

    hideProgress() {
        document.getElementById('progressSection').classList.add('hidden');
        this.updateProgress(0, '');
    }

    updateProgress(percent, text = '') {
        const progressFill = document.getElementById('progressFill');
        const progressText = document.getElementById('progressText');
        const progressPercent = document.getElementById('progressPercent');
        
        progressFill.style.width = `${percent}%`;
        progressText.textContent = text || 'Preparing...';
        progressPercent.textContent = `${percent}%`;
    }

    addConsoleOutput(message, type = 'info') {
        const consoleContent = document.getElementById('consoleContent');
        const line = document.createElement('div');
        line.className = `console-line ${type}`;
        line.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
        
        consoleContent.appendChild(line);
        consoleContent.scrollTop = consoleContent.scrollHeight;
        
        // Limit console history
        const lines = consoleContent.querySelectorAll('.console-line');
        if (lines.length > 100) {
            lines[0].remove();
        }
    }

    async handleAppClose() {
        if (this.gameState.isRunning) {
            const confirmed = confirm('Minecraft is still running. Do you want to stop it and close the launcher?');
            if (confirmed) {
                await this.killGame();
                nw.App.quit();
            }
        } else {
            nw.App.quit();
        }
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.app = new LauncherApp();
});

// Global function for mod management
window.openModFile = (modFile) => {
    if (window.app) {
        window.app.openModFile(modFile);
    }
};

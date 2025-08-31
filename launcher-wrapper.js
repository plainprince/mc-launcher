const Launch = require("./Launcher/Launch");
const { inspectLogs, utilsEmitter } = require("./Launcher/utils");
const MicrosoftAuth = require("./Launcher/Authenticator/Microsoft");
const { EventEmitter } = require("events");
const fs = require("fs");
const path = require("path");

class MinecraftLauncher extends EventEmitter {
    constructor() {
        super();
        this.launcher = new Launch();
        this.microsoftAuth = new MicrosoftAuth();
        this.gameControls = null;
        this.isRunning = false;
        
        // Set up event listeners for log events from launcher and utils
        this.launcher.on('log', (level, message) => {
            this.emit('log', level, message);
        });
        
        utilsEmitter.on('log', (level, message) => {
            this.emit('log', level, message);
        });
    }

    // Simple login function
    async login(useGUI = true, quiet = false) {
        const accountPath = path.join(__dirname, "account.json");

        if (fs.existsSync(accountPath)) {
            const account = JSON.parse(fs.readFileSync(accountPath, "utf-8"));
            const refreshedAccount = await this.microsoftAuth.refresh(account);
            if (refreshedAccount.error) {
                if (!quiet) console.log("Refreshing account...");
                return this._authenticate(useGUI, quiet);
            }
            fs.writeFileSync(accountPath, JSON.stringify(refreshedAccount, null, 4));
            if (!quiet) console.log("Logged in successfully!");
            return refreshedAccount;
        } else {
            return this._authenticate(useGUI, quiet);
        }
    }

    async _authenticate(useGUI, quiet = false) {
        if (!quiet) console.log("🔐 Opening authentication...");
        const account = await this.microsoftAuth.getAuth({ useGUI });
        if (account.error) {
            throw new Error(account.error);
        }
        fs.writeFileSync(path.join(__dirname, "account.json"), JSON.stringify(account, null, 4));
        if (!quiet) console.log("Authentication successful!");
        return account;
    }

    // Simple launch function
    async launch(options = {}) {
        if (this.isRunning) {
            throw new Error("Minecraft is already running!");
        }

        const defaultOptions = {
            version: "1.21.4",
            instance: "Fabric1214",
            quiet: false,
            downloadQuiet: false,
            ...options
        };

        if (!defaultOptions.quiet) {
            console.log("Starting Minecraft...");
        }
        
        const mc = await this.login(true, defaultOptions.quiet);

        // Auto-detect Intel emulation need
        const versionParts = defaultOptions.version.split('.');
        const majorVersion = parseInt(versionParts[0]);
        const minorVersion = parseInt(versionParts[1]);
        const needsIntelEmulation = process.platform === 'darwin' && 
                                  (majorVersion < 1 || (majorVersion === 1 && minorVersion <= 16));

        if (!defaultOptions.quiet) {
            console.log(`Platform: ${process.platform}`);
            console.log(`Minecraft Version: ${defaultOptions.version}`);
            console.log(`Intel Emulation: ${needsIntelEmulation ? 'Enabled' : 'Disabled'}`);
        }

        // Build launch options - merge default options with user settings
        const launchOptions = {
            path: defaultOptions.workingDirectory || "./minecraft",
            authenticator: mc,
            version: defaultOptions.version,
            intelEnabledMac: needsIntelEmulation,
            instance: defaultOptions.instance,
            ignored: ["config", "logs", "resourcepacks", "options.txt", "optionsof.txt"],
            loader: { type: "fabric", build: "0.16.10", enable: true },
            memory: defaultOptions.memory || { min: "4G", max: "8G" },
            quiet: defaultOptions.quiet,

            mods: {
                downloadUrls: [
                    // Performance mods
                    "https://cdn.modrinth.com/data/gvQqBUqZ/versions/Iq9qGzm9/lithium-fabric-mc1.21.4-0.14.3.jar", // Lithium
                    "https://cdn.modrinth.com/data/P7dR8mSH/versions/DXTJz4V3/fabric-api-0.110.5%2B1.21.4.jar", // Fabric API
                    "https://cdn.modrinth.com/data/AANobbMI/versions/JGfwEgJ6/sodium-fabric-0.6.5%2Bmc1.21.4.jar", // Sodium
                    
                    // Visual enhancement mods that don't require config
                    "https://cdn.modrinth.com/data/zV5r3pPn/versions/vYXe74OW/3dskinlayers-fabric-1.7.5-mc1.21.4.jar", // 3D Skin Layers
                    "https://cdn.modrinth.com/data/Wb5oqrBJ/versions/yGOJFWmG/chat_heads-0.13.7-fabric-1.21.4.jar", // Chat Heads
                    "https://cdn.modrinth.com/data/1IjD5062/versions/F6Vn3LV4/continuity-3.1.0%2B1.21.4.jar", // Continuity
                ],
                customMods: "/Users/simeonkummer/dev/mc-launcher/custom-mod/build/libs"
            }
        };

        // Add custom JVM arguments if provided
        if (defaultOptions.javaArgs && defaultOptions.javaArgs.length > 0) {
            launchOptions.javaArgs = defaultOptions.javaArgs;
        }

        // Add custom game arguments if provided
        if (defaultOptions.gameArgs && defaultOptions.gameArgs.length > 0) {
            launchOptions.gameArgs = defaultOptions.gameArgs;
        }

        // Add custom Java path if provided
        if (defaultOptions.javaPath) {
            launchOptions.javaPath = defaultOptions.javaPath;
        }

        // Add environment variables if provided
        if (defaultOptions.environmentVars && Object.keys(defaultOptions.environmentVars).length > 0) {
            launchOptions.env = defaultOptions.environmentVars;
        }

        await this.launcher.Launch(launchOptions);

        this.launcher
            .on("progress", (progress, size) => {
                if (!defaultOptions.downloadQuiet) {
                    console.log(`[DL] ${((progress / size) * 100).toFixed(2)}%`);
                }
            })
            .on("patch", (patch) => {
                if (!defaultOptions.quiet) {
                    process.stdout.write(patch);
                }
            })
            .on("data", (line) => {
                if (!defaultOptions.quiet) {
                    process.stdout.write(line);
                }
            })
            .on("error", (err) => {
                if (!defaultOptions.quiet) {
                    console.error("Launch error:", err);
                }
                this.isRunning = false;
            })
            .on("close", () => {
                this.isRunning = false;
                this.gameControls = null;
                // Emit close event to any listeners (like the main menu)
                this.emit('close');
            });

        this.isRunning = true;
        this.gameControls = {
            kill: (quiet = defaultOptions.quiet) => this.kill(quiet),
            inspectLogs: () => this.inspectLogs(),
            isRunning: () => this.isRunning,
            process: this.launcher.minecraftProcess,
        };
        
        if (!defaultOptions.quiet) {
            console.log("Minecraft launched successfully!");
        }
        return this.gameControls;
    }

    // Kill function
    async kill(quiet = false) {
        if (!this.isRunning || !this.launcher.minecraftProcess) {
            if (!quiet) console.log("No Minecraft process to kill");
            return false;
        }

        if (!quiet) console.log("Killing Minecraft...");
        const result = await this.launcher.killProcess(quiet);
        if (result) {
            this.isRunning = false;
            this.gameControls = null;
        }
        return result;
    }

    // Inspect logs - only Minecraft game logs, no Java crash dumps
    inspectLogs() {
        const instance = "Fabric1214"; // Default instance
        const instancePath = `./minecraft/instances/${instance}`;
        const logPath = `${instancePath}/logs/latest.log`;
        
        try {
            if (fs.existsSync(logPath)) {
                const logs = fs.readFileSync(logPath, "utf8");
                if (logs && logs.trim()) {
                    return logs;
                } else {
                    return "Minecraft log is empty (game may not have started yet)";
                }
            } else {
                return "No Minecraft logs found (game has not been started yet)";
            }
        } catch (error) {
            return `Error reading Minecraft log: ${error.message}`;
        }
    }

    // Get current state
    getState() {
        return {
            isRunning: this.isRunning,
            hasProcess: !!this.launcher.minecraftProcess,
            pid: this.launcher.getPID()
        };
    }
}

// Create singleton instance
const minecraftLauncher = new MinecraftLauncher();

// Export simple functions
module.exports = {
    login: (useGUI = true, quiet = false) => minecraftLauncher.login(useGUI, quiet),
    launch: (options = {}) => minecraftLauncher.launch(options),
    kill: (quiet = false) => minecraftLauncher.kill(quiet),
    inspectLogs: () => minecraftLauncher.inspectLogs(),
    getState: () => minecraftLauncher.getState(),
    launcher: minecraftLauncher // Direct access if needed
};

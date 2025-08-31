const fs = require("fs");
const path = require("path");
const os = require("os");
const enquirer = require("enquirer");

class SettingsManager {
    constructor() {
        // Get proper app data directory based on platform
        this.appDataDir = this.getAppDataDirectory();
        this.settingsPath = path.join(this.appDataDir, "settings.json");
        this.defaultSettings = {
            memory: {
                min: "4G",
                max: "8G"
            },
            java: {
                customPath: null,
                autoDetect: true
            },
            jvm: {
                customArgs: [],
                gc: "G1GC", // G1GC, ZGC, ParallelGC, SerialGC
                gcArgs: "-XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=32M",
                optimizations: true
            },
            game: {
                fullscreen: false,
                resolution: {
                    width: 1920,
                    height: 1080
                }
            },
            performance: {
                allocation: "high", // low, medium, high, ultra
                fpsLimit: 0, // 0 = unlimited
                vSync: false
            },
            advanced: {
                javaArgs: "",
                gameArgs: "",
                environmentVars: {},
                workingDirectory: "./minecraft"
            }
        };
        this.settings = this.loadSettings();
    }

    getAppDataDirectory() {
        const platform = os.platform();
        const homedir = os.homedir();
        let appDataPath;
        
        switch (platform) {
            case 'win32':
                appDataPath = path.join(process.env.APPDATA || path.join(homedir, 'AppData', 'Roaming'), 'MinecraftLauncher');
                break;
            case 'darwin':
                appDataPath = path.join(homedir, 'Library', 'Application Support', 'MinecraftLauncher');
                break;
            default: // Linux and other Unix-like systems
                appDataPath = path.join(process.env.XDG_CONFIG_HOME || path.join(homedir, '.config'), 'MinecraftLauncher');
                break;
        }
        
        // Create directory if it doesn't exist
        if (!fs.existsSync(appDataPath)) {
            fs.mkdirSync(appDataPath, { recursive: true });
        }
        
        return appDataPath;
    }

    loadSettings() {
        try {
            if (fs.existsSync(this.settingsPath)) {
                const savedSettings = JSON.parse(fs.readFileSync(this.settingsPath, "utf-8"));
                return this.mergeSettings(this.defaultSettings, savedSettings);
            }
        } catch (error) {
            console.warn("Failed to load settings, using defaults:", error.message);
        }
        return JSON.parse(JSON.stringify(this.defaultSettings));
    }

    mergeSettings(defaults, saved) {
        const merged = JSON.parse(JSON.stringify(defaults));
        
        for (const [key, value] of Object.entries(saved)) {
            if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
                if (merged[key] && typeof merged[key] === 'object') {
                    merged[key] = { ...merged[key], ...value };
                } else {
                    merged[key] = value;
                }
            } else {
                merged[key] = value;
            }
        }
        
        return merged;
    }

    saveSettings() {
        try {
            fs.writeFileSync(this.settingsPath, JSON.stringify(this.settings, null, 4));
            return true;
        } catch (error) {
            console.error("Failed to save settings:", error.message);
            return false;
        }
    }

    getSettings() {
        return this.settings;
    }

    updateSettings(newSettings) {
        this.settings = this.mergeSettings(this.settings, newSettings);
        return this.saveSettings();
    }

    resetSettings() {
        this.settings = JSON.parse(JSON.stringify(this.defaultSettings));
        return this.saveSettings();
    }

    // Convert settings to launcher options
    toLauncherOptions() {
        const settings = this.settings;
        
        // Build JVM arguments
        let jvmArgs = [];
        
        // Add GC settings
        if (settings.jvm.gc === "G1GC") {
            jvmArgs.push("-XX:+UseG1GC");
            if (settings.jvm.gcArgs) {
                jvmArgs.push(...settings.jvm.gcArgs.split(/\s+/).filter(arg => arg.trim()));
            }
        } else if (settings.jvm.gc === "ZGC") {
            jvmArgs.push("-XX:+UnlockExperimentalVMOptions", "-XX:+UseZGC");
        } else if (settings.jvm.gc === "ParallelGC") {
            jvmArgs.push("-XX:+UseParallelGC");
        } else if (settings.jvm.gc === "SerialGC") {
            jvmArgs.push("-XX:+UseSerialGC");
        }

        // Add performance optimizations
        if (settings.jvm.optimizations) {
            jvmArgs.push(
                "-XX:+DisableExplicitGC",
                "-XX:+AlwaysPreTouch",
                "-XX:+PerfDisableSharedMem",
                "-Dfml.ignoreInvalidMinecraftCertificates=true",
                "-Dfml.ignorePatchDiscrepancies=true"
            );
        }

        // Add custom JVM args
        if (settings.jvm.customArgs && settings.jvm.customArgs.length > 0) {
            jvmArgs.push(...settings.jvm.customArgs);
        }

        // Add custom Java args from advanced settings
        if (settings.advanced.javaArgs && settings.advanced.javaArgs.trim()) {
            jvmArgs.push(...settings.advanced.javaArgs.trim().split(/\s+/).filter(arg => arg.trim()));
        }

        // Build game arguments
        let gameArgs = [];
        
        if (settings.game.fullscreen) {
            gameArgs.push("--fullscreen");
        } else {
            gameArgs.push("--width", settings.game.resolution.width.toString());
            gameArgs.push("--height", settings.game.resolution.height.toString());
        }

        // Add custom game args
        if (settings.advanced.gameArgs && settings.advanced.gameArgs.trim()) {
            gameArgs.push(...settings.advanced.gameArgs.trim().split(/\s+/).filter(arg => arg.trim()));
        }

        return {
            memory: {
                min: settings.memory.min,
                max: settings.memory.max
            },
            javaArgs: jvmArgs,
            gameArgs: gameArgs,
            javaPath: settings.java.autoDetect ? null : settings.java.customPath,
            environmentVars: settings.advanced.environmentVars,
            workingDirectory: settings.advanced.workingDirectory
        };
    }

    // Main settings menu
    async showSettingsMenu() {
        while (true) {
                    console.log("\nLAUNCHER SETTINGS");
        console.log("=" .repeat(40));

        const choices = [
            "Memory Settings",
            "Java Settings", 
            "JVM Options",
            "Game Settings",
            "Performance Settings",
            "Advanced Options",
            "View Current Settings",
            "Reset to Defaults",
            "Save & Return to Main Menu"
        ];

            const prompt = new enquirer.Select({
                name: "action",
                message: "Choose a category:",
                choices: choices
            });

            try {
                const answer = await prompt.run();
                
                if (answer.includes("Memory Settings")) {
                    await this.memorySettingsMenu();
                } else if (answer.includes("Java Settings")) {
                    await this.javaSettingsMenu();
                } else if (answer.includes("JVM Options")) {
                    await this.jvmSettingsMenu();
                } else if (answer.includes("Game Settings")) {
                    await this.gameSettingsMenu();
                } else if (answer.includes("Performance Settings")) {
                    await this.performanceSettingsMenu();
                } else if (answer.includes("Advanced Options")) {
                    await this.advancedSettingsMenu();
                } else if (answer.includes("View Current Settings")) {
                    this.displayCurrentSettings();
                    console.log("\nPress any key to continue...");
                    await new Promise(resolve => {
                        const stdin = process.stdin;
                        stdin.setRawMode(true);
                        stdin.resume();
                        stdin.once('data', () => {
                            stdin.setRawMode(false);
                            stdin.pause();
                            resolve();
                        });
                    });
                } else if (answer.includes("Reset to Defaults")) {
                    const confirmReset = await new enquirer.Confirm({
                        name: "confirm",
                        message: "Are you sure you want to reset all settings to defaults?"
                    }).run();
                    
                    if (confirmReset) {
                        this.resetSettings();
                        console.log("Settings reset to defaults!");
                        await new Promise(resolve => setTimeout(resolve, 1500));
                    }
                } else if (answer.includes("Save & Return")) {
                    if (this.saveSettings()) {
                        console.log("Settings saved successfully!");
                    } else {
                        console.log("Failed to save settings!");
                    }
                    await new Promise(resolve => setTimeout(resolve, 1500));
                    break;
                }
            } catch (error) {
                if (error.message && error.message.includes('cancelled')) {
                    break;
                }
                console.error("Error in settings menu:", error.message);
                await new Promise(resolve => setTimeout(resolve, 2000));
            }
        }
    }

    async memorySettingsMenu() {
        console.log("\nMEMORY SETTINGS");
        console.log("=" .repeat(30));
        console.log(`Current: ${this.settings.memory.min} - ${this.settings.memory.max}`);
        
        const choices = [
            "2G - 4G (Low)",
            "4G - 6G (Medium)", 
            "4G - 8G (High - Current)",
            "6G - 12G (Very High)",
            "8G - 16G (Ultra)",
            "Custom values",
            "← Back"
        ];

        const answer = await new enquirer.Select({
            name: "memory",
            message: "Select memory allocation:",
            choices: choices
        }).run();

        if (answer.includes("2G - 4G")) {
            this.settings.memory = { min: "2G", max: "4G" };
        } else if (answer.includes("4G - 6G")) {
            this.settings.memory = { min: "4G", max: "6G" };
        } else if (answer.includes("4G - 8G")) {
            this.settings.memory = { min: "4G", max: "8G" };
        } else if (answer.includes("6G - 12G")) {
            this.settings.memory = { min: "6G", max: "12G" };
        } else if (answer.includes("8G - 16G")) {
            this.settings.memory = { min: "8G", max: "16G" };
        } else if (answer.includes("Custom")) {
            await this.customMemorySettings();
        }
    }

    async customMemorySettings() {
        try {
            const minMem = await new enquirer.Input({
                name: "min",
                message: "Minimum memory (e.g., 4G, 2048M):",
                initial: this.settings.memory.min
            }).run();

            const maxMem = await new enquirer.Input({
                name: "max", 
                message: "Maximum memory (e.g., 8G, 8192M):",
                initial: this.settings.memory.max
            }).run();

            this.settings.memory = { min: minMem, max: maxMem };
            console.log(`Memory set to ${minMem} - ${maxMem}`);
        } catch (error) {
            // User cancelled
        }
    }

    async javaSettingsMenu() {
        console.log("\nJAVA SETTINGS");
        console.log("=" .repeat(30));
        
        const autoDetectAnswer = await new enquirer.Confirm({
            name: "autoDetect",
            message: "Auto-detect Java installation?",
            initial: this.settings.java.autoDetect
        }).run();

        this.settings.java.autoDetect = autoDetectAnswer;

        if (!autoDetectAnswer) {
            const customPath = await new enquirer.Input({
                name: "path",
                message: "Enter custom Java executable path:",
                initial: this.settings.java.customPath || ""
            }).run();

            this.settings.java.customPath = customPath || null;
        }
    }

    async jvmSettingsMenu() {
        console.log("\nJVM OPTIONS");
        console.log("=" .repeat(30));

        const choices = [
            "Garbage Collector Settings",
            "Performance Optimizations",
            "Custom JVM Arguments", 
            "← Back"
        ];

        const answer = await new enquirer.Select({
            name: "jvm",
            message: "Choose JVM option category:",
            choices: choices
        }).run();

        if (answer.includes("Garbage Collector")) {
            await this.gcSettingsMenu();
        } else if (answer.includes("Performance Optimizations")) {
            const enabled = await new enquirer.Confirm({
                name: "optimizations",
                message: "Enable performance optimizations?",
                initial: this.settings.jvm.optimizations
            }).run();
            this.settings.jvm.optimizations = enabled;
        } else if (answer.includes("Custom JVM Arguments")) {
            await this.customJvmArgsMenu();
        }
    }

    async gcSettingsMenu() {
        const choices = [
            "G1GC (Recommended for most users)",
            "ZGC (Low latency, Java 17+)",
            "ParallelGC (High throughput)",
            "SerialGC (Single core)"
        ];

        const answer = await new enquirer.Select({
            name: "gc",
            message: "Select Garbage Collector:",
            choices: choices,
            initial: choices.findIndex(choice => choice.includes(this.settings.jvm.gc))
        }).run();

        if (answer.includes("G1GC")) {
            this.settings.jvm.gc = "G1GC";
        } else if (answer.includes("ZGC")) {
            this.settings.jvm.gc = "ZGC";
        } else if (answer.includes("ParallelGC")) {
            this.settings.jvm.gc = "ParallelGC";
        } else if (answer.includes("SerialGC")) {
            this.settings.jvm.gc = "SerialGC";
        }

        if (this.settings.jvm.gc === "G1GC") {
            const customGC = await new enquirer.Confirm({
                name: "customGC",
                message: "Customize G1GC arguments?",
                initial: false
            }).run();

            if (customGC) {
                const gcArgs = await new enquirer.Input({
                    name: "args",
                    message: "G1GC arguments:",
                    initial: this.settings.jvm.gcArgs
                }).run();
                this.settings.jvm.gcArgs = gcArgs;
            }
        }
    }

    async customJvmArgsMenu() {
        console.log("\nCurrent JVM arguments:");
        console.log(this.settings.jvm.customArgs.join(" ") || "None");

        const argsString = await new enquirer.Input({
            name: "args",
            message: "Enter custom JVM arguments (space-separated):",
            initial: this.settings.jvm.customArgs.join(" ")
        }).run();

        this.settings.jvm.customArgs = argsString ? argsString.split(/\s+/).filter(arg => arg.trim()) : [];
    }

    async gameSettingsMenu() {
        console.log("\nGAME SETTINGS");
        console.log("=" .repeat(30));

        const fullscreen = await new enquirer.Confirm({
            name: "fullscreen",
            message: "Launch in fullscreen?",
            initial: this.settings.game.fullscreen
        }).run();

        this.settings.game.fullscreen = fullscreen;

        if (!fullscreen) {
            const width = await new enquirer.Input({
                name: "width",
                message: "Window width:",
                initial: this.settings.game.resolution.width.toString()
            }).run();

            const height = await new enquirer.Input({
                name: "height", 
                message: "Window height:",
                initial: this.settings.game.resolution.height.toString()
            }).run();

            this.settings.game.resolution.width = parseInt(width) || 1920;
            this.settings.game.resolution.height = parseInt(height) || 1080;
        }

        // Launcher always stays open - removed confusing setting
    }

    async performanceSettingsMenu() {
        console.log("\nPERFORMANCE SETTINGS");
        console.log("=" .repeat(30));

        const allocationChoices = [
            "Low (Basic performance)",
            "Medium (Balanced)",
            "High (Recommended)",
            "Ultra (Maximum performance)"
        ];

        const allocation = await new enquirer.Select({
            name: "allocation",
            message: "Performance allocation:",
            choices: allocationChoices,
            initial: allocationChoices.findIndex(choice => choice.includes(this.settings.performance.allocation))
        }).run();

        if (allocation.includes("Low")) {
            this.settings.performance.allocation = "low";
        } else if (allocation.includes("Medium")) {
            this.settings.performance.allocation = "medium";
        } else if (allocation.includes("High")) {
            this.settings.performance.allocation = "high";
        } else if (allocation.includes("Ultra")) {
            this.settings.performance.allocation = "ultra";
        }

        const fpsLimit = await new enquirer.Input({
            name: "fps",
            message: "FPS limit (0 = unlimited):",
            initial: this.settings.performance.fpsLimit.toString()
        }).run();

        this.settings.performance.fpsLimit = parseInt(fpsLimit) || 0;

        const vsync = await new enquirer.Confirm({
            name: "vsync",
            message: "Enable VSync?",
            initial: this.settings.performance.vSync
        }).run();

        this.settings.performance.vSync = vsync;
    }

    async advancedSettingsMenu() {
        console.log("\nADVANCED OPTIONS");
        console.log("=" .repeat(30));

        const choices = [
            "Custom Java Arguments",
            "Custom Game Arguments", 
            "Environment Variables",
            "Working Directory",
            "← Back"
        ];

        const answer = await new enquirer.Select({
            name: "advanced",
            message: "Choose advanced option:",
            choices: choices
        }).run();

        if (answer.includes("Custom Java Arguments")) {
            const javaArgs = await new enquirer.Input({
                name: "javaArgs",
                message: "Custom Java arguments:",
                initial: this.settings.advanced.javaArgs
            }).run();
            this.settings.advanced.javaArgs = javaArgs;

        } else if (answer.includes("Custom Game Arguments")) {
            const gameArgs = await new enquirer.Input({
                name: "gameArgs",
                message: "Custom game arguments:",
                initial: this.settings.advanced.gameArgs
            }).run();
            this.settings.advanced.gameArgs = gameArgs;

        } else if (answer.includes("Environment Variables")) {
            console.log("Current environment variables:");
            console.log(JSON.stringify(this.settings.advanced.environmentVars, null, 2));
            
            const envVarsString = await new enquirer.Input({
                name: "envVars",
                message: "Environment variables (JSON format):",
                initial: JSON.stringify(this.settings.advanced.environmentVars)
            }).run();

            try {
                this.settings.advanced.environmentVars = JSON.parse(envVarsString || "{}");
            } catch (error) {
                console.log("Invalid JSON format, keeping current values");
            }

        } else if (answer.includes("Working Directory")) {
            const workingDir = await new enquirer.Input({
                name: "workingDir",
                message: "Working directory:",
                initial: this.settings.advanced.workingDirectory
            }).run();
            this.settings.advanced.workingDirectory = workingDir;
        }
    }

    displayCurrentSettings() {
        console.log("\nCURRENT SETTINGS");
        console.log("=" .repeat(50));
        
        console.log("\nMemory:");
        console.log(`  Min: ${this.settings.memory.min}`);
        console.log(`  Max: ${this.settings.memory.max}`);
        
        console.log("\nJava:");
        console.log(`  Auto-detect: ${this.settings.java.autoDetect}`);
        console.log(`  Custom path: ${this.settings.java.customPath || "None"}`);
        
        console.log("\nJVM:");
        console.log(`  Garbage Collector: ${this.settings.jvm.gc}`);
        console.log(`  Optimizations: ${this.settings.jvm.optimizations}`);
        console.log(`  Custom args: ${this.settings.jvm.customArgs.join(" ") || "None"}`);
        
        console.log("\nGame:");
        console.log(`  Fullscreen: ${this.settings.game.fullscreen}`);
        console.log(`  Resolution: ${this.settings.game.resolution.width}x${this.settings.game.resolution.height}`);
        
        console.log("\nPerformance:");
        console.log(`  Allocation: ${this.settings.performance.allocation}`);
        console.log(`  FPS limit: ${this.settings.performance.fpsLimit === 0 ? "Unlimited" : this.settings.performance.fpsLimit}`);
        console.log(`  VSync: ${this.settings.performance.vSync}`);
        
        console.log("\nAdvanced:");
        console.log(`  Java args: ${this.settings.advanced.javaArgs || "None"}`);
        console.log(`  Game args: ${this.settings.advanced.gameArgs || "None"}`);
        console.log(`  Environment vars: ${Object.keys(this.settings.advanced.environmentVars).length} defined`);
        console.log(`  Working directory: ${this.settings.advanced.workingDirectory}`);
        
        console.log("=" .repeat(50));
    }
}

module.exports = SettingsManager;

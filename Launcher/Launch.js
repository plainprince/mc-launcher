"use strict";
/**
 * @author Luuxis
 * Luuxis License v1.0 (voir fichier LICENSE pour les détails en FR/EN)
 */
const { EventEmitter } = require("events");
const path = require("path");
const fs = require("fs");
const { spawn } = require("child_process");
const MinecraftJson = require("./Minecraft/Minecraft-Json.js");
const MinecraftLibraries = require("./Minecraft/Minecraft-Libraries.js");
const MinecraftAssets = require("./Minecraft/Minecraft-Assets.js");
const MinecraftLoader = require("./Minecraft/Minecraft-Loader.js");
const MinecraftJava = require("./Minecraft/Minecraft-Java.js");
const MinecraftBundle = require("./Minecraft/Minecraft-Bundle.js");
const MinecraftArguments = require("./Minecraft/Minecraft-Arguments.js");
const { isold } = require("./utils/Index.js");
const Downloader = require("./utils/Downloader.js");

class Launch extends EventEmitter {
    constructor() {
        super();
        this.minecraftProcess = null;
        this.spawnPromise = null;
    }
    
    // Method to kill the Minecraft process directly
    async killProcess(quiet = null) {
        // Use the options.quiet if no explicit quiet parameter is provided
        if (quiet === null) {
            quiet = this.options?.quiet || false;
        }
        if (this.spawnPromise) {
            try {
                await this.spawnPromise;
            } catch (error) {
                this.emit('log', 'info', "Minecraft process failed to start, nothing to kill.");
                if (!quiet) console.log("Minecraft process failed to start, nothing to kill.");
                return false;
            }
        }

        if (this.minecraftProcess && !this.minecraftProcess.killed) {
            this.emit('log', 'info', `Killing Minecraft process (PID: ${this.minecraftProcess.pid})`);
            this.minecraftProcess.kill('SIGTERM');
            
            // Force kill after 5 seconds if it doesn't respond
            setTimeout(() => {
                if (this.minecraftProcess && !this.minecraftProcess.killed) {
                    this.emit('log', 'warn', 'Force killing Minecraft process...');
                    this.minecraftProcess.kill('SIGKILL');
                }
            }, 5000);
            
            // Process termination logged by wrapper
            return true;
        } else {
            this.emit('log', 'info', "No Minecraft process to kill");
            if (!quiet) console.log("No Minecraft process to kill");
            return false;
        }
    }

    getPID() {
        return this.minecraftProcess?.pid || null;
    }
    
    async Launch(opt) {
        const defaultOptions = {
            url: null,
            authenticator: null,
            timeout: 10000,
            path: '.Minecraft',
            version: 'latest_release',
            instance: null,
            detached: false,
            intelEnabledMac: false,
            downloadFileMultiple: 5,
            bypassOffline: false,
            quiet: false, // Add quiet option to default options
            loader: {
                path: './loader',
                type: null,
                build: 'latest',
                enable: false,
            },
            mcp: null,
            verify: false,
            ignored: [],
            JVM_ARGS: [],
            GAME_ARGS: [],
            java: {
                path: null,
                version: null,
                type: 'jre',
            },
            screen: {
                width: null,
                height: null,
                fullscreen: false,
            },
            memory: {
                min: '1G',
                max: '2G'
            },
            ...opt,
        };
        this.options = defaultOptions;
        this.options.path = path.resolve(this.options.path).replace(/\\/g, '/');
        if (this.options.mcp) {
            if (this.options.instance)
                this.options.mcp = `${this.options.path}/instances/${this.options.instance}/${this.options.mcp}`;
            else
                this.options.mcp = path.resolve(`${this.options.path}/${this.options.mcp}`).replace(/\\/g, '/');
        }
        if (this.options.loader.type) {
            this.options.loader.type = this.options.loader.type.toLowerCase();
            this.options.loader.build = this.options.loader.build.toLowerCase();
        }
        if (!this.options.authenticator)
            return this.emit("error", { error: "Authenticator not found" });
        if (this.options.downloadFileMultiple < 1)
            this.options.downloadFileMultiple = 1;
        if (this.options.downloadFileMultiple > 30)
            this.options.downloadFileMultiple = 30;
        if (typeof this.options.loader.path !== 'string')
            this.options.loader.path = `./loader/${this.options.loader.type}`;
        await this.start();
    }
    async start() {
        let data = await this.DownloadGame();
        if (data.error)
            return this.emit('error', data);
        let { minecraftJson, minecraftLoader, minecraftVersion, minecraftJava } = data;
        let minecraftArguments = await new MinecraftArguments(this.options).GetArguments(minecraftJson, minecraftLoader);
        if (minecraftArguments.error)
            return this.emit('error', minecraftArguments);
        let loaderArguments = await new MinecraftLoader(this.options).GetArguments(minecraftLoader, minecraftVersion);
        if (loaderArguments.error)
            return this.emit('error', loaderArguments);
        let Arguments = [
            ...minecraftArguments.jvm,
            ...minecraftArguments.classpath,
            ...loaderArguments.jvm,
            minecraftArguments.mainClass,
            ...minecraftArguments.game,
            ...loaderArguments.game
        ];
        let java = this.options.java.path ? this.options.java.path : minecraftJava.path;
        let logs = this.options.instance ? `${this.options.path}/instances/${this.options.instance}` : this.options.path;
        if (!fs.existsSync(logs))
            fs.mkdirSync(logs, { recursive: true });
        let argumentsLogs = Arguments.join(' ');
        argumentsLogs = argumentsLogs.replaceAll(this.options.authenticator?.access_token, '????????');
        argumentsLogs = argumentsLogs.replaceAll(this.options.authenticator?.client_token, '????????');
        argumentsLogs = argumentsLogs.replaceAll(this.options.authenticator?.uuid, '????????');
        argumentsLogs = argumentsLogs.replaceAll(this.options.authenticator?.xboxAccount?.xuid, '????????');
        argumentsLogs = argumentsLogs.replaceAll(`${this.options.path}/`, '');
        this.emit('data', `Launching with arguments ${argumentsLogs}`);

        this.spawnPromise = new Promise((resolve, reject) => {
            const minecraftDebug = spawn(java, Arguments, { cwd: logs, detached: this.options.detached });

            this.minecraftProcess = minecraftDebug;
    
            const instancePath = path.join(this.options.path, 'instances', this.options.instance);
            const logDir = path.join(instancePath, 'logs');
            const latestLogPath = path.join(logDir, 'latest.log');
            
            if (!fs.existsSync(logDir)) {
                fs.mkdirSync(logDir, { recursive: true });
            }
            
            fs.writeFileSync(latestLogPath, '');
            const logStream = fs.createWriteStream(latestLogPath, { flags: 'a' });
            
            minecraftDebug.stdout.on('data', (data) => {
                const text = data.toString('utf-8');
                logStream.write(text);
                this.emit('data', text);
            });
            
            minecraftDebug.stderr.on('data', (data) => {
                const text = data.toString('utf-8');
                logStream.write(text);
                this.emit('data', text);
            });
            
            minecraftDebug.on('close', (code) => {
                logStream.end();
                
                if (fs.existsSync(latestLogPath)) {
                    const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '_');
                    const archiveLogPath = path.join(logDir, `${timestamp}.log`);
                    fs.copyFileSync(latestLogPath, archiveLogPath);
                    this.emit('data', `📋 Game logs archived as: ${timestamp}.log\n`);
                }
                
                this.minecraftProcess = null;
                this.emit('close', 'Minecraft closed');
            });
            
            minecraftDebug.on('spawn', () => {
                this.emit('data', `🔍 Process spawned with PID: ${minecraftDebug.pid}\n`);
                resolve();
            });

            minecraftDebug.on('error', (error) => {
                this.emit('error', { error: 'Minecraft process failed to start.', details: error });
                reject(error);
            });
        });

        try {
            await this.spawnPromise;
        } catch (err) {
            // The error is already emitted by the promise reject handler.
            // We catch it here to prevent an unhandled promise rejection,
            // as the library primarily uses events for error reporting.
        }
    }
    async DownloadGame() {
        let InfoVersion = await new MinecraftJson(this.options).GetInfoVersion();
        let loaderJson = null;
        if ('error' in InfoVersion) {
            return this.emit('error', InfoVersion);
        }
        let { json, version } = InfoVersion;
        let libraries = new MinecraftLibraries(this.options);
        let bundle = new MinecraftBundle(this.options);
        let java = new MinecraftJava(this.options);
        java.on('progress', (progress, size, element) => {
            this.emit('progress', progress, size, element);
        });
        java.on('extract', (progress) => {
            this.emit('extract', progress);
        });
        let gameLibraries = await libraries.Getlibraries(json);
        let gameAssetsOther = await libraries.GetAssetsOthers(this.options.url);
        let gameAssets = await new MinecraftAssets(this.options).getAssets(json);
        let gameJava = this.options.java.path ? { files: [] } : await java.getJavaFiles(json);
        if (gameJava.error)
            return gameJava;
        let filesList = await bundle.checkBundle([...gameLibraries, ...gameAssetsOther, ...gameAssets, ...gameJava.files]);
        if (filesList.length > 0) {
            let downloader = new Downloader();
            let totsize = await bundle.getTotalSize(filesList);
            downloader.on("progress", (DL, totDL, element) => {
                this.emit("progress", DL, totDL, element);
            });
            downloader.on("speed", (speed) => {
                this.emit("speed", speed);
            });
            downloader.on("estimated", (time) => {
                this.emit("estimated", time);
            });
            downloader.on("error", (e) => {
                this.emit("error", e);
            });
            await downloader.downloadFileMultiple(filesList, totsize, this.options.downloadFileMultiple, this.options.timeout);
        }
        if (this.options.loader.enable === true) {
            let loaderInstall = new MinecraftLoader(this.options);
            loaderInstall.on('extract', (extract) => {
                this.emit('extract', extract);
            });
            loaderInstall.on('progress', (progress, size, element) => {
                this.emit('progress', progress, size, element);
            });
            loaderInstall.on('check', (progress, size, element) => {
                this.emit('check', progress, size, element);
            });
            loaderInstall.on('patch', (patch) => {
                this.emit('patch', patch);
            });
            let jsonLoader = await loaderInstall.GetLoader(version, this.options.java.path ? this.options.java.path : gameJava.path)
                .then((data) => data)
                .catch((err) => err);
            if (jsonLoader.error)
                return jsonLoader;
            loaderJson = jsonLoader;
        }
        if (this.options.verify)
            await bundle.checkFiles([...gameLibraries, ...gameAssetsOther, ...gameAssets, ...gameJava.files]);
        let natives = await libraries.natives(gameLibraries);
        if (natives.length === 0)
            json.nativesList = false;
        else
            json.nativesList = true;
        if (isold(json))
            new MinecraftAssets(this.options).copyAssets(json);
            
        // Handle mod loading if mods configuration is provided
        if (this.options.mods) {
            await this.setupMods();
        }
            
        return {
            minecraftJson: json,
            minecraftLoader: loaderJson,
            minecraftVersion: version,
            minecraftJava: gameJava
        };
    }

    async setupMods() {
        if (!this.options.mods) return;

        const { downloadUrls = [], customMods } = this.options.mods;
        const instancePath = this.options.instance ? 
            path.join(this.options.path, 'instances', this.options.instance) : 
            this.options.path;
        const modsDir = path.join(instancePath, 'mods');

        // Create mods directory if it doesn't exist
        if (!fs.existsSync(modsDir)) {
            fs.mkdirSync(modsDir, { recursive: true });
        }

        this.emit('data', '🔧 Setting up mods...\n');

        // Download mods from URLs
        if (downloadUrls.length > 0) {
            this.emit('data', '⬇️  Downloading mods from URLs...\n');
            const Downloader = require('./utils/Downloader');
            const downloader = new Downloader();
            
            for (const url of downloadUrls) {
                try {
                    const fileName = url.split('/').pop().split('?')[0]; // Extract filename from URL
                    const destPath = path.join(modsDir, fileName);
                    
                    // Skip if mod already exists
                    if (fs.existsSync(destPath)) {
                        this.emit('data', `⏭️  Mod already exists: ${fileName}\n`);
                        continue;
                    }
                    
                    this.emit('data', `⬇️  Downloading: ${fileName}\n`);
                    await downloader.downloadFile(url, modsDir, fileName);
                    this.emit('data', `✅ Downloaded: ${fileName}\n`);
                } catch (error) {
                    this.emit('data', `❌ Failed to download mod from ${url}: ${error.message}\n`);
                }
            }
        }



        // Copy custom mods
        if (customMods && fs.existsSync(customMods)) {
            const customModFiles = fs.readdirSync(customMods).filter(file => 
                file.endsWith('.jar') && !file.includes('sources')
            );
            for (const modFile of customModFiles) {
                const sourcePath = path.join(customMods, modFile);
                const destPath = path.join(modsDir, modFile);
                
                try {
                    fs.copyFileSync(sourcePath, destPath);
                    this.emit('data', `✅ Copied custom mod: ${modFile}\n`);
                } catch (error) {
                    this.emit('data', `❌ Failed to copy custom mod ${modFile}: ${error.message}\n`);
                }
            }
        }

        this.emit('data', '🎮 Mods setup complete!\n');
    }
}
module.exports = Launch;
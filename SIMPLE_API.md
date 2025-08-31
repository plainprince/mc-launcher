# 🎮 Simple Minecraft Launcher API

## ✨ Super Clean API - Just Two Functions!

```javascript
const { login, launch } = require("./launcher-wrapper");

// That's literally it! 
await login();   // Optional - launch() does this automatically
await launch();  // Handles everything: auth, platform detection, process management
```

## 🚀 Core Functions

### `login(useGUI = true)`
- 🔐 Handles Microsoft authentication
- 💾 Automatic token refresh and storage
- 🖥️ GUI popup by default (set to false for terminal)
- ✅ Returns account info

### `launch(options = {})`
- 🎮 Launches Minecraft with all the smart defaults
- 🤖 **Auto-handles everything**: Intel emulation, platform detection, auth
- 📦 Returns game controls object
- 🔧 Configurable options

**Options:**
```javascript
{
    version: "1.8.9",     // Minecraft version
    instance: "Hypixel",  // Instance name  
    quiet: false,         // Show/hide output
    downloadQuiet: false  // Show/hide download progress
}
```

## 🎯 Additional Utilities

### `kill()`
- 🔄 Kills the running Minecraft process
- ✅ Uses direct child process reference (super reliable!)
- 📱 Returns `true` if killed, `false` if no process

### `getState()`
- 📊 Returns current launcher state
- 🔍 `{ isRunning, hasProcess, pid }`

### `inspectLogs()`
- 📋 Gets the latest Minecraft logs
- 📄 Handles "no logs found" gracefully

## 🎪 Example Usage

### Minimal Example
```javascript
const { launch } = require("./launcher-wrapper");

// Just launch it!
await launch();
```

### Full Control Example
```javascript
const { login, launch, kill, getState } = require("./launcher-wrapper");

// Login first (optional)
await login(true); // GUI auth

// Launch with specific settings
const game = await launch({
    version: "1.17.1",   // Different version
    instance: "Vanilla", // Different instance
    quiet: true          // Silent mode
});

// Check if running
const state = getState();
console.log(`Running: ${state.isRunning}, PID: ${state.pid}`);

// Kill when done
kill();
```

## 🧠 Smart Features (All Automatic!)

### ✅ **Platform Detection**
- 🍎 Automatically enables Intel emulation on macOS for MC ≤ 1.16
- 💻 Cross-platform compatibility

### ✅ **Authentication Management**  
- 🔐 GUI authentication by default
- 💾 Token storage and automatic refresh
- 🔄 Re-authentication when needed

### ✅ **Process Management**
- 🎯 Direct child process killing (super reliable!)
- 📊 Real-time state tracking
- 🛡️ Prevents multiple instances

### ✅ **Error Handling**
- 🚨 Graceful error messages
- 🔄 Automatic cleanup on failures
- 📝 User-friendly feedback

## 🎭 Interactive Launcher (`index.js`)

Run `bun index.js` for a beautiful interactive menu:

```
🎮 Minecraft Launcher
==============================
? What would you like to do? 
  ❯ Launch Minecraft
    View Game Logs  
    Exit

? Minecraft is running (PID: 12345). What would you like to do?
  ❯ Kill Minecraft
    View Game Logs
    Exit
```

### ✨ Dynamic Menu Features:
- 🔄 **Changes based on state** - Shows "Launch" or "Kill" based on whether MC is running
- 📊 **Shows PID** when Minecraft is running
- 📋 **Smart log handling** - Shows "no logs found" when appropriate
- 🎨 **Clean UX** with emojis and clear feedback

## 🏗️ Architecture

### Before (Complex)
```javascript
// Lots of manual setup required
const launcher = new Launch();
const auth = new MicrosoftAuth();
const account = await auth.getAuth({useGUI: true});
const options = {
    // 20+ lines of configuration
    intelEnabledMac: process.platform === 'darwin' && version <= '1.16',
    // ... tons more setup
};
await launcher.Launch(options);
// Manual process finding and killing
```

### After (Simple!)
```javascript
// Everything handled automatically
const { launch } = require("./launcher-wrapper");
await launch(); // That's it!
```

## 🎯 Benefits

1. **🚀 Super Simple** - Just `login()` and `launch()`
2. **🤖 Intelligent** - Auto-detects platform, version requirements
3. **🔒 Reliable** - Direct process management, no searching
4. **🎨 Beautiful** - Clean UI with emojis and clear feedback  
5. **🛡️ Robust** - Proper error handling and state management
6. **📦 Self-Contained** - All complexity hidden in `Launcher/` folder

Perfect for both simple scripts and interactive launchers!

# Minecraft Launcher (NW.js)

A modern Minecraft launcher built with NW.js, featuring a beautiful UI and support for Fabric 1.21.4.

## Features

- 🎮 **Modern UI**: Beautiful, responsive interface with dark theme
- 🧩 **Mod Management**: Integrated mod loading and management
- ⚡ **Performance**: Optimized launcher with memory and performance settings
- 🔐 **Microsoft Authentication**: Secure login with Microsoft accounts
- 📋 **Real-time Logs**: Live game output and console logs
- ⚙️ **Settings**: Comprehensive configuration options

## Setup

1. Install dependencies:
   ```bash
   bun install
   ```

2. Run the launcher:
   ```bash
   bun run start
   ```

3. For development with logging:
   ```bash
   bun run dev
   ```

## Configuration

The launcher is pre-configured for:
- **Minecraft Version**: 1.21.4
- **Mod Loader**: Fabric 0.16.10
- **Instance**: Fabric1214
- **Mods**: Automatically loads mods from the configured directories

## Mods Included

The launcher automatically manages mods from:
- PrismLauncher instance mods (excluding specified mods)
- Custom built mods from `custom-mod/build/libs`

See the current mod list in the Mods tab of the launcher.

## Building

To build a distributable:
```bash
bun run build
```

## Requirements

- NW.js 0.92.0+
- Node.js 18+ (for dependencies)
- macOS 10.15+ / Windows 10+ / Linux

## Structure

```
nw-launcher/
├── index.html          # Main UI
├── styles.css          # Styling
├── renderer.js         # Frontend logic
├── launcher-wrapper.js # Launcher integration
├── settings.js         # Settings management
├── Launcher/           # Core launcher library
└── assets/             # Icons and images
```

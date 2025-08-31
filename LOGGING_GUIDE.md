# Event-Based Logging System

This launcher now supports an event-based logging system that allows package users to capture and handle all log messages through events instead of relying on console output.

## Overview

Previously, the launcher used `console.log()` statements throughout the codebase, making it difficult for package users to control logging behavior. Now, all logging operations emit events while maintaining backward compatibility with console output.

## Key Features

- **Event-driven logging**: All log messages are emitted as events
- **Backward compatibility**: Console output still works when `quiet: false`
- **Quiet mode support**: Set `quiet: true` to suppress console output while still receiving events
- **Log levels**: Support for `info`, `warn`, `error`, and `debug` levels
- **Multiple sources**: Events from launcher core, utilities, and GUI components

## Usage

### Basic Event Listening

```javascript
const { launcher } = require('./launcher-wrapper');

// Listen to all log events
launcher.on('log', (level, message) => {
    console.log(`[${level.toUpperCase()}] ${message}`);
});
```

### Filtered Logging

```javascript
// Only show warnings and errors
launcher.on('log', (level, message) => {
    if (level === 'warn' || level === 'error') {
        console.log(`🚨 ${message}`);
    }
});
```

### Custom Logger Integration

```javascript
const customLogger = {
    log: (level, message) => {
        // Send to your logging service
        myLoggingService.send({
            timestamp: Date.now(),
            level,
            message,
            component: 'minecraft-launcher'
        });
    }
};

launcher.on('log', (level, message) => {
    customLogger.log(level, message);
});
```

### Quiet Mode with Events

```javascript
// Launch without console output, but still receive events
await launch({ 
    version: "1.8.9", 
    instance: "Hypixel",
    quiet: true  // No console.log output
});

// All logging information is still available via events
launcher.on('log', (level, message) => {
    // Handle logs however you want
});
```

## Log Levels

| Level | Description | Example Use Cases |
|-------|-------------|-------------------|
| `info` | General information | Process status, successful operations |
| `warn` | Warning messages | Fallback operations, deprecated features |
| `error` | Error messages | Failed operations, exceptions |
| `debug` | Debug information | Detailed process information, troubleshooting |

## Event Sources

The logging events come from several components:

1. **Main Launcher** (`Launch.js`): Core launching operations
2. **Utils** (`utils.js`): Utility functions like process killing
3. **Terminal GUI** (`Terminal.js`): Terminal-based authentication
4. **Electron GUI** (`Electron.js`): Electron-based authentication

## Migration Guide

### Before (console-only):
```javascript
// You had to live with whatever the launcher logged
await launch({ version: "1.8.9" });
// Console output was uncontrollable
```

### After (event-based):
```javascript
// Set up your own logging
launcher.on('log', (level, message) => {
    if (level === 'error') {
        myErrorTracker.track(message);
    }
    myLogger.log(level, message);
});

// Launch in quiet mode
await launch({ 
    version: "1.8.9",
    quiet: true  // No console spam
});
```

## Available Options

### Launch Options
- `quiet: boolean` - Suppress console output from launcher core
- `downloadQuiet: boolean` - Suppress download progress output

### Kill Options
- `quiet: boolean` - Suppress console output during process termination

## Examples

See `examples/event-logging-example.js` for comprehensive examples including:
- Basic event logging
- Filtered logging by level
- Custom logger integration
- Quiet mode usage

## Backward Compatibility

All existing code continues to work. If you don't set `quiet: true`, console output behaves exactly as before. The event system is an addition, not a replacement.

```javascript
// This still works exactly as before
await launch({ version: "1.8.9" });
```

## Best Practices

1. **Use quiet mode** when integrating the launcher as a library
2. **Filter by log level** to avoid spam in production
3. **Integrate with your logging system** using events
4. **Keep error handling** for critical issues
5. **Test both modes** to ensure your application works correctly

## Troubleshooting

### No Events Received
- Ensure you're listening to events before calling launch functions
- Check that you're using the correct event name (`'log'`)

### Console Output Still Showing
- Make sure you set `quiet: true` in your launch options
- Some critical errors may still show in console for debugging

### Missing Logs
- Check if you're filtering out important log levels
- Verify your event listener is set up correctly

const { launch, launcher } = require("../launcher-wrapper");

/**
 * Example demonstrating how to use the new event-based logging system
 * 
 * The launcher now emits 'log' events with three parameters:
 * - level: 'info', 'warn', 'error', 'debug'
 * - message: The log message string
 * - source: The component that generated the log (optional)
 * 
 * This allows package users to:
 * 1. Control what gets logged and how
 * 2. Filter by log level
 * 3. Integrate with their own logging systems
 * 4. Choose between quiet mode and event handling
 */

async function exampleWithEventLogging() {
    console.log("🎮 Event-based Logging Example");
    console.log("=" .repeat(40));

    // Set up event listeners for different log levels
    launcher.on('log', (level, message) => {
        const timestamp = new Date().toISOString();
        const prefix = {
            'info': '📝',
            'warn': '⚠️ ',
            'error': '❌',
            'debug': '🔍'
        }[level] || '📝';
        
        console.log(`[${timestamp}] ${prefix} ${level.toUpperCase()}: ${message}`);
    });

    // Example 1: Launch with quiet mode but still get structured logs
    console.log("\n--- Example 1: Quiet mode with event logging ---");
    try {
        await launch({ 
            version: "1.8.9", 
            instance: "Hypixel",
            quiet: true, // No console.log output from launcher
            downloadQuiet: true // No download progress output
        });
        
        console.log("✅ Game launched! You can see all logs above via events.");
        
        // Kill after 5 seconds for demo
        setTimeout(async () => {
            console.log("\n🔄 Killing game for demo...");
            const killed = await launcher.kill(true); // quiet kill
            if (killed) {
                console.log("✅ Demo completed!");
                process.exit(0);
            }
        }, 5000);
        
    } catch (error) {
        console.error("❌ Launch failed:", error.message);
    }
}

async function exampleWithFilteredLogging() {
    console.log("🎮 Filtered Logging Example");
    console.log("=" .repeat(40));

    // Only show warnings and errors
    launcher.on('log', (level, message) => {
        if (level === 'warn' || level === 'error') {
            console.log(`🚨 [${level.toUpperCase()}] ${message}`);
        }
    });

    // Example 2: Show only important messages
    console.log("\n--- Example 2: Error/Warning only logging ---");
    try {
        await launch({ 
            version: "1.8.9", 
            instance: "Hypixel",
            quiet: true
        });
        
        console.log("✅ Only warnings and errors are shown via events.");
        
    } catch (error) {
        console.error("❌ Launch failed:", error.message);
    }
}

async function exampleWithCustomLogging() {
    console.log("🎮 Custom Logging Integration Example");
    console.log("=" .repeat(40));

    // Custom logger that could integrate with your logging system
    const customLogger = {
        logs: [],
        log: function(level, message) {
            const entry = {
                timestamp: Date.now(),
                level,
                message,
                component: 'minecraft-launcher'
            };
            this.logs.push(entry);
            
            // You could send this to your own logging service, file, etc.
            console.log(`[CUSTOM] ${level}: ${message}`);
        },
        
        getLogs: function() {
            return this.logs;
        },
        
        getLogsByLevel: function(level) {
            return this.logs.filter(log => log.level === level);
        }
    };

    launcher.on('log', (level, message) => {
        customLogger.log(level, message);
    });

    // Example 3: Custom logging integration
    console.log("\n--- Example 3: Custom logging integration ---");
    try {
        await launch({ 
            version: "1.8.9", 
            instance: "Hypixel",
            quiet: true
        });
        
        console.log("✅ All logs captured in custom logger!");
        console.log(`📊 Total logs: ${customLogger.getLogs().length}`);
        console.log(`⚠️  Warnings: ${customLogger.getLogsByLevel('warn').length}`);
        console.log(`❌ Errors: ${customLogger.getLogsByLevel('error').length}`);
        
    } catch (error) {
        console.error("❌ Launch failed:", error.message);
    }
}

// Run the example you want to test
// Uncomment one of these:

exampleWithEventLogging();
// exampleWithFilteredLogging();
// exampleWithCustomLogging();

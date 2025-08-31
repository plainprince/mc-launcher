const enquirer = require("enquirer");
const { launch, kill, inspectLogs, getState, launcher } = require("./launcher-wrapper");
const SettingsManager = require("./settings");

async function main() {
    console.log("Minecraft Launcher");
    console.log("=" .repeat(30));

    // Initialize settings manager
    const settingsManager = new SettingsManager();

    // Set up event listener for launcher logs (optional - for demonstration)
    launcher.on('log', (level, message) => {
        if (level === 'error' || level === 'warn') {
            console.log(`[${level.toUpperCase()}] ${message}`);
        }
    });

    // Set up event listener for Minecraft process close to refresh menu
    let currentPromptController = null;
    let minecraftJustClosed = false; // Flag to track if Minecraft just closed
    launcher.on('close', () => {
        minecraftJustClosed = true;
        // If there's an active prompt, cancel it to refresh the menu
        if (currentPromptController) {
            currentPromptController.cancel();
        }
    });

    let firstLaunch = true;
    
    while(true) {
        let state = getState();
        
        // Dynamic menu based on current state
        const choices = [];
        
        if (!state.isRunning) {
            choices.push("Launch Minecraft");
        } else {
            choices.push("Kill Minecraft");
        }
        
        choices.push("View Game Logs");
        choices.push("Settings");
        choices.push("Exit");

        // console.log(state)

        const prompt = new enquirer.Select({
            name: "action",
            message: state.isRunning ? 
                `Minecraft is running (PID: ${state.pid}). What would you like to do?` : 
                "What would you like to do?",
            choices: choices
        });

        try {
            // Store the prompt controller so it can be cancelled if Minecraft closes
            currentPromptController = prompt;
            const answer = await prompt.run();
            currentPromptController = null; // Clear the controller after successful completion
            
            if(answer === "Launch Minecraft") {
                try {
                    console.log("\nLaunching Minecraft...");
                    
                    // Get launcher options from settings
                    const launcherOptions = settingsManager.toLauncherOptions();
                    
                    await launch({ 
                        version: "1.21.4", 
                        instance: "Fabric1214",
                        quiet: true, // Keep output clean in menu
                        ...launcherOptions
                    });
                } catch (error) {
                    console.error("Failed to launch Minecraft:", error.message);
                    console.log("Returning to menu in 3 seconds...");
                    await new Promise(resolve => setTimeout(resolve, 3000));
                }
                
            } else if(answer === "Kill Minecraft") {
                const killed = await kill();
                if (killed) {
                    console.log("Minecraft has been terminated.");
                } else {
                    console.log("No Minecraft process found to kill.");
                }
                
            } else if(answer === "View Game Logs") {
                console.log("\nFetching Minecraft logs...");
                try {
                    const logs = inspectLogs();
                    console.log("\n" + "=".repeat(60));
                    console.log("MINECRAFT GAME LOGS:");
                    console.log("=".repeat(60));
                    
                    if (logs.includes("No Minecraft logs") || logs.includes("empty") || logs.includes("Error")) {
                        console.log(logs);
                    } else {
                        // Show last 50 lines for better readability
                        const lines = logs.split('\n');
                        const lastLines = lines.slice(-50);
                        console.log(lastLines.join('\n'));
                        if (lines.length > 50) {
                            console.log(`\n... (showing last 50 lines of ${lines.length} total)`);
                        }
                    }
                    console.log("=".repeat(60));
                } catch (error) {
                    console.log("Error reading logs:", error.message);
                }
                
                // Auto-continue after 3 seconds or any key press
                console.log("\nReturning to menu in 3 seconds... (press any key to continue immediately)");
                await Promise.race([
                    new Promise(resolve => setTimeout(resolve, 3000)),
                    new Promise(resolve => process.stdin.once('data', resolve))
                ]);
                
            } else if(answer === "Settings") {
                try {
                    await settingsManager.showSettingsMenu();
                } catch (error) {
                    if (!error.message || !error.message.includes('cancelled')) {
                        console.error("Error in settings:", error.message);
                        await new Promise(resolve => setTimeout(resolve, 2000));
                    }
                }
                
            } else if(answer === "Exit") {
                console.log("\nShutting down...");
                if (state.isRunning) {
                    console.log("Stopping Minecraft first...");
                    await kill();
                }
                console.log("Goodbye!");
                process.exit(0);
            }
            
        } catch (error) {
            currentPromptController = null; // Clear the controller on any error
            
            // Check if this error is due to Minecraft closing
            if (minecraftJustClosed) {
                minecraftJustClosed = false; // Reset the flag
                console.log("Minecraft was closed. Refreshing menu...");
                continue;
            }
            
            // Handle prompt cancellation (when Minecraft closes) differently
            if (error.message && error.message.includes('cancelled')) {
                // Prompt was cancelled due to Minecraft closing, just continue to refresh menu
                continue;
            }
            
            // Handle other enquirer cancellation scenarios (Ctrl+C, etc.)
            if (!error.message || error.message === 'undefined' || error.name === 'CancelPromptError') {
                // Likely a prompt cancellation, continue silently to refresh menu
                continue;
            }
            
            console.error("Error:", error.message || 'Unknown error occurred');
            console.log("Returning to menu in 3 seconds...");
            await new Promise(resolve => setTimeout(resolve, 3000));
        }
        
        console.log(); // Add spacing between menu iterations
    }
}

main().catch(error => {
    console.error("Fatal error:", error);
    process.exit(1);
});
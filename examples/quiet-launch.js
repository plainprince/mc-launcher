// Example showing the quiet mode - no console spam!

const { launch, kill, getState } = require("../launcher-wrapper");

async function quietExample() {
    console.log("🤫 Quiet Launch Example - No Spam!");
    console.log("=" .repeat(40));
    
    try {
        console.log("1. Launching in quiet mode...");
        
        await launch({
            version: "1.8.9",
            instance: "Hypixel",
            quiet: true,        // No console output from launcher
            downloadQuiet: true // No download progress
        });
        
        console.log("2. ✅ Launched silently!");
        
        const state = getState();
        console.log(`3. 📊 State: Running=${state.isRunning}, PID=${state.pid}`);
        
        console.log("4. Waiting 5 seconds...");
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        console.log("5. Killing quietly...");
        await kill(true); // Quiet kill
        
        console.log("6. ✅ All done - no spam!");
        
    } catch (error) {
        console.error("❌ Error:", error.message);
    }
}

quietExample();

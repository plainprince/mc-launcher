// Simple API example - just login and launch!

const { login, launch, kill, getState } = require("../launcher-wrapper");

async function simpleExample() {
    console.log("🎮 Simple Minecraft Launcher API Example");
    console.log("=" .repeat(50));
    
    try {
        // Step 1: Login (optional - launch() will do this automatically)
        console.log("\n1. 🔐 Logging in...");
        await login(true); // Use GUI
        
        // Step 2: Launch Minecraft
        console.log("\n2. 🚀 Launching Minecraft...");
        const gameControls = await launch({
            version: "1.8.9",
            instance: "Hypixel",
            quiet: false // Show output
        });
        
        // Step 3: Check state
        console.log("\n3. 📊 Checking state...");
        const state = getState();
        console.log("State:", state);
        
        // Step 4: Wait a bit then kill
        console.log("\n4. ⏳ Waiting 10 seconds...");
        await new Promise(resolve => setTimeout(resolve, 10000));
        
        console.log("\n5. 🔄 Killing Minecraft...");
        const killed = await kill();
        console.log("Killed:", killed);
        
        console.log("\n✅ Example completed!");
        
    } catch (error) {
        console.error("❌ Error:", error.message);
    }
}

simpleExample();

const { launch } = require("../launcher-wrapper");

async function demo() {
    console.log("🎮 Direct Child Process Killing Demo");
    console.log("=" .repeat(40));
    
    console.log("\n1. Launching Minecraft...");
    const gameControls = await launch(false, false, "1.8.9");
    
    console.log("\n2. Waiting 5 seconds...");
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    console.log("\n3. Killing via direct child process reference...");
    const killed = await gameControls.kill();
    
    if (killed) {
        console.log("✅ Successfully killed via direct child process!");
    } else {
        console.log("❌ Failed to kill via direct child process");
    }
    
    console.log("\n4. Process info:");
    if (gameControls.process) {
        console.log(`   PID: ${gameControls.process.pid}`);
        console.log(`   Killed: ${gameControls.process.killed}`);
    } else {
        console.log("   No process reference available");
    }
    
    console.log("\n🎉 Demo completed!");
}

demo().catch(console.error);

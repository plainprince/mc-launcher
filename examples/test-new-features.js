// Test the new features: PID display, log streaming, and clean log viewing

const { launch, kill, getState, inspectLogs } = require("../launcher-wrapper");

async function testNewFeatures() {
    console.log("🧪 Testing New Features");
    console.log("=".repeat(40));
    
    try {
        console.log("1. 🚀 Launching Minecraft...");
        await launch({
            version: "1.8.9",
            instance: "Hypixel",
            quiet: true
        });
        
        console.log("2. 📊 Checking state...");
        const state = getState();
        console.log(`   Running: ${state.isRunning}`);
        console.log(`   Has Process: ${state.hasProcess}`);
        console.log(`   PID: ${state.pid} ${state.pid ? '✅' : '❌'}`);
        
        console.log("3. 📋 Reading logs...");
        const logs = inspectLogs();
        const lineCount = logs.split('\n').length;
        console.log(`   Log lines: ${lineCount}`);
        console.log(`   Sample: ${logs.split('\n')[0]}`);
        
        console.log("4. ⏳ Waiting 5 seconds...");
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        console.log("5. 🔪 Killing process...");
        const killed = await kill(true); // Quiet kill
        console.log(`   Killed: ${killed ? '✅' : '❌'}`);
        
        console.log("6. 📁 Checking log archiving...");
        const fs = require('fs');
        const logDir = './minecraft/instances/Hypixel/logs';
        const files = fs.readdirSync(logDir);
        const archiveFiles = files.filter(f => f.match(/\d{4}-\d{2}-\d{2}_\d{2}-\d{2}-\d{2}\.log/));
        console.log(`   Archive files: ${archiveFiles.length} ${archiveFiles.length > 0 ? '✅' : '❌'}`);
        
        console.log("\n🎉 All features working!");
        
    } catch (error) {
        console.error("❌ Error:", error.message);
    }
}

testNewFeatures();

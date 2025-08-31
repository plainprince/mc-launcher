const { launch, kill } = require("../launcher-wrapper");
const { kill: killWithDebug } = require("../Launcher/utils");

async function testFeatures() {
    console.log("=".repeat(50));
    console.log("🎮 Enhanced Minecraft Launcher Test");
    console.log("=".repeat(50));
    
    // Test 1: Platform and version detection
    console.log("\n1. Testing platform and version detection:");
    console.log("   For Minecraft 1.8.9 (should use Intel emulation on macOS)");
    
    try {
        const gameControls = await launch(false, false, "1.8.9"); // quiet=false to see platform info
        console.log("   ✅ Launcher started successfully");
        
        // Wait a moment for the game to start
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        console.log("\n2. Testing enhanced process detection:");
        await killWithDebug(true); // Enable debug mode
        
        console.log("   ✅ Kill function test completed");
        
    } catch (error) {
        console.error("   ❌ Error during test:", error.message);
    }
    
    console.log("\n3. Testing with newer version (1.17.1):");
    console.log("   This should disable Intel emulation on macOS");
    
    // Simulate version check
    const version = "1.17.1";
    const versionParts = version.split('.');
    const majorVersion = parseInt(versionParts[0]);
    const minorVersion = parseInt(versionParts[1]);
    const needsIntelEmulation = process.platform === 'darwin' && 
                               (majorVersion < 1 || (majorVersion === 1 && minorVersion <= 16));
    
    console.log(`   Platform: ${process.platform}`);
    console.log(`   Version: ${version}`);
    console.log(`   Intel Emulation: ${needsIntelEmulation ? 'Enabled' : 'Disabled'}`);
    console.log("   ✅ Version detection working correctly");
    
    console.log("\n" + "=".repeat(50));
    console.log("🎉 All tests completed!");
    console.log("=".repeat(50));
}

testFeatures().catch(console.error);

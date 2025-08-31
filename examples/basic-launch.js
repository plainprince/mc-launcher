const { launch, kill, inspectLogs, getMicrosoftAuth } = require("../launcher-wrapper");

async function main() {
    console.log("Starting Minecraft...");
    const controls = await launch();
    console.log("Minecraft launched.");

    // Example: Kill Minecraft after 20 seconds
    setTimeout(async () => {
        console.log("Killing Minecraft...");
        await controls.kill();
        console.log("Minecraft killed.");
    }, 20000);

    // Example: Inspect logs every 5 seconds
    const logInterval = setInterval(async () => {
        console.log("Inspecting logs...");
        const logs = await controls.inspectLogs();
        console.log("Latest logs:\n", logs.slice(-500)); // print last 500 chars
    }, 5000);

    // Stop inspecting logs after 30 seconds
    setTimeout(() => {
        clearInterval(logInterval);
    }, 30000);
}

main().catch(console.error);


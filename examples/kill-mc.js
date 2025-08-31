const { kill } = require("../launcher-wrapper");

async function main() {
    console.log("Attempting to kill Minecraft process...");
    await kill();
}

main().catch(console.error);


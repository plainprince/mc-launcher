const { inspectLogs } = require("../launcher-wrapper");
const { getMicrosoftAuth } = require("../launcher-wrapper");


async function main() {
    console.log("Getting latest logs...");
    // You might need to authenticate first to ensure instance folder exists
    await getMicrosoftAuth(); // This will create account.json if it doesn't exist
    const logs = await inspectLogs("./minecraft", "Hypixel");
    console.log(logs);
}

main().catch(console.error);


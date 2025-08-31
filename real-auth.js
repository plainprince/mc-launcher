import fs from "fs";
import { Microsoft } from "minecraft-java-core";

let mc = await new Microsoft().getAuth();
fs.writeFileSync("./account.json", JSON.stringify(mc, null, 4));

// wichtig: Output an Parent schicken
console.log(JSON.stringify(mc, null, 2));
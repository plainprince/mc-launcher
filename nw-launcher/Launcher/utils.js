const fkill = require("fkill").default;
const findProcess = require("find-process").default;
const fs = require("fs");
const { EventEmitter } = require("events");

// Create a global event emitter for utility logging
const utilsEmitter = new EventEmitter();

// Legacy kill function - only used as fallback when direct child process killing fails
async function killBySearch(debug = false, quiet = false) {
	const message = "⚠️  Using fallback process search method...";
	utilsEmitter.emit('log', 'warn', message);
	if (!quiet) console.log(message);
	
	try {
		// Find Java processes that contain minecraft in their command line
		const list = await findProcess("name", "java");
		const foundMessage = `Found ${list.length} Java processes`;
		utilsEmitter.emit('log', 'info', foundMessage);
		if (!quiet) console.log(foundMessage);
		
		if (debug) {
			const debugMessage = "Debug: All Java processes:";
			utilsEmitter.emit('log', 'debug', debugMessage);
			if (!quiet) console.log(debugMessage);
			
			list.forEach((p, i) => {
				const pidMessage = `  ${i + 1}. PID: ${p.pid}, Name: ${p.name}`;
				const cmdMessage = `     Command: ${p.cmd?.substring(0, 100)}...`;
				utilsEmitter.emit('log', 'debug', pidMessage);
				utilsEmitter.emit('log', 'debug', cmdMessage);
				if (!quiet) {
					console.log(pidMessage);
					console.log(cmdMessage);
				}
			});
		}
		
		// Look for processes with minecraft-related arguments
		const mc = list.find((p) => {
			if (!p.cmd) return false;
			const cmd = p.cmd.toLowerCase();
			return cmd.includes("minecraft") || 
				   cmd.includes("launchwrapper") || 
				   cmd.includes("fml") ||
				   cmd.includes("forge") ||
				   cmd.includes("net.minecraft");
		});

		if (!mc) {
			const notFoundMessage = "Minecraft process not found via command line arguments";
			utilsEmitter.emit('log', 'info', notFoundMessage);
			if (!quiet) console.log(notFoundMessage);
			
			// Fallback: try to find any Java process with our specific runtime path
			const runtimeMc = list.find((p) => {
				if (!p.cmd) return false;
				return p.cmd.includes("/minecraft/runtime/") || p.cmd.includes("jre-8u74");
			});
			
			if (runtimeMc) {
				const foundRuntimeMessage = "Found Minecraft Java process via runtime path";
				utilsEmitter.emit('log', 'info', foundRuntimeMessage);
				if (!quiet) console.log(foundRuntimeMessage);
				
				if (debug) {
					const debugRuntimeMessage = `Debug: Runtime process - PID: ${runtimeMc.pid}, Command: ${runtimeMc.cmd}`;
					utilsEmitter.emit('log', 'debug', debugRuntimeMessage);
					if (!quiet) console.log(debugRuntimeMessage);
				}
				await fkill(runtimeMc.pid, { force: true });
				
				const terminatedMessage = "Minecraft has been terminated!";
				utilsEmitter.emit('log', 'info', terminatedMessage);
				if (!quiet) console.log(terminatedMessage);
				return true;
			}
			
			const noProcessMessage = "No Minecraft processes found to kill";
			utilsEmitter.emit('log', 'info', noProcessMessage);
			if (!quiet) console.log(noProcessMessage);
			return false;
		}

		const killingMessage = `Killing Minecraft process (PID: ${mc.pid})`;
		utilsEmitter.emit('log', 'info', killingMessage);
		if (!quiet) console.log(killingMessage);
		
		if (debug) {
			const debugMainMessage = `Debug: Main process - Command: ${mc.cmd}`;
			utilsEmitter.emit('log', 'debug', debugMainMessage);
			if (!quiet) console.log(debugMainMessage);
		}
		await fkill(mc.pid, { force: true });
		
		const terminatedMessage = "Minecraft has been terminated!";
		utilsEmitter.emit('log', 'info', terminatedMessage);
		if (!quiet) console.log(terminatedMessage);
		return true;
	} catch (err) {
		const errorMessage = `Error killing Minecraft: ${err}`;
		utilsEmitter.emit('log', 'error', errorMessage);
		if (!quiet) console.error("Error killing Minecraft:", err);
		return false;
	}
}

// Simple kill function that tries fallback if needed
async function kill(debug = false, quiet = false) {
	const noteMessage = "🚨 Note: Use launcher.killProcess() for direct child process killing instead";
	utilsEmitter.emit('log', 'warn', noteMessage);
	if (!quiet) console.log(noteMessage);
	return await killBySearch(debug, quiet);
}

async function inspectLogs(minecraftPath, instanceName) {
	const logPath = `${minecraftPath}/instances/${instanceName}/logs/latest.log`;
	try {
		const logs = fs.readFileSync(
			logPath,
			"utf8"
		);
		return logs;
	} catch (error) {
		if (error.code === 'ENOENT') {
			return `Log file not found at ${logPath}`;
		}
		return `Error reading log file: ${error.message}`;
	}
}

module.exports = {
	kill,
	killBySearch,
	inspectLogs,
	utilsEmitter // Export the event emitter for external access
};


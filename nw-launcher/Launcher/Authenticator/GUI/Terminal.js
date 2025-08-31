"use strict";
/**
 * @author Luuxis
 * Luuxis License v1.0 (voir fichier LICENSE pour les détails en FR/EN)
 */
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const prompt_1 = __importDefault(require("prompt"));
const { EventEmitter } = require("events");

// Create event emitter for terminal GUI logging
const terminalEmitter = new EventEmitter();

module.exports = async function (url, quiet = false) {
    const message = `Open browser ${url}`;
    terminalEmitter.emit('log', 'info', message);
    if (!quiet) console.log(message);
    
    prompt_1.default.start();
    let result = await prompt_1.default.get(['copy-URL']);
    return result['copy-URL'].split("code=")[1].split("&")[0];
};

module.exports.terminalEmitter = terminalEmitter;
//# sourceMappingURL=Terminal.js.map
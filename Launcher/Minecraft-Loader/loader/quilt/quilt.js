"use strict";
/**
 * @author Luuxis
 * Luuxis License v1.0 (voir fichier LICENSE pour les détails en FR/EN)
 */
const path = require("path");
const { EventEmitter } = require("events");
const Index_js_1 = require("../../../utils/Index.js");
const Downloader = require("../../../utils/Downloader.js");
/**
 * This class handles fetching the Quilt loader metadata,
 * identifying the appropriate build for a given Minecraft version,
 * and downloading required libraries.
 */
class Quilt extends EventEmitter {
    constructor(options = { path: '', loader: { version: '', build: '' } }) {
        super();
        this.options = options;
    }
    /**
     * Fetches the Quilt loader metadata to identify the correct build for the specified
     * Minecraft version. If "latest" or "recommended" is requested, picks the most
     * recent or stable build accordingly.
     *
     * @param Loader An object describing where to fetch Quilt metadata and JSON.
     * @returns      A QuiltJSON object on success, or an error object if something fails.
     */
    async downloadJson(Loader) {
        let selectedBuild;
        // Fetch the metadata
        const metaResponse = await fetch(Loader.metaData);
        const metaData = await metaResponse.json();
        // Check if the requested Minecraft version is supported
        const mcVersionExists = metaData.game.find((ver) => ver.version === this.options.loader.version);
        if (!mcVersionExists) {
            return { error: `QuiltMC doesn't support Minecraft ${this.options.loader.version}` };
        }
        // Gather all available builds for this version
        const availableBuilds = metaData.loader.map((b) => b.version);
        // Determine which build to use
        if (this.options.loader.build === 'latest') {
            selectedBuild = metaData.loader[0];
        }
        else if (this.options.loader.build === 'recommended') {
            // Attempt to find a build that isn't labeled "beta"
            selectedBuild = metaData.loader.find((b) => !b.version.includes('beta'));
        }
        else {
            // Otherwise, match a specific build
            selectedBuild = metaData.loader.find((loaderItem) => loaderItem.version === this.options.loader.build);
        }
        if (!selectedBuild) {
            return {
                error: `QuiltMC Loader ${this.options.loader.build} not found, Available builds: ${availableBuilds.join(', ')}`
            };
        }
        // Build the URL for the Quilt loader profile JSON
        const url = Loader.json
            .replace('${build}', selectedBuild.version)
            .replace('${version}', this.options.loader.version);
        // Fetch the JSON profile
        try {
            const response = await fetch(url);
            const quiltJson = await response.json();
            return quiltJson;
        }
        catch (err) {
            return { error: err.message || 'Failed to fetch or parse Quilt loader JSON' };
        }
    }
    /**
     * Parses the Quilt JSON to determine which libraries need downloading, skipping
     * any that already exist or that are disqualified by "rules". Downloads them
     * in bulk using the Downloader utility.
     *
     * @param quiltJson A QuiltJSON object containing a list of libraries.
     * @returns         The final list of libraries, or an error if something fails.
     */
    async downloadLibraries(quiltJson) {
        const { libraries } = quiltJson;
        const downloader = new Downloader_js_1.default();
        let filesToDownload = [];
        let checkedLibraries = 0;
        let totalSize = 0;
        for (const lib of libraries) {
            // If rules exist, skip it (likely platform-specific logic)
            if (lib.rules) {
                this.emit('check', checkedLibraries++, libraries.length, 'libraries');
                continue;
            }
            // Construct the local path where this library should reside
            const libInfo = (0, require("../../../utils/Index.js").getPathLibraries)(lib.name);
            const libFolder = path_1.default.resolve(this.options.path, 'libraries', libInfo.path);
            const libFilePath = path_1.default.resolve(libFolder, libInfo.name);
            // If the library doesn't exist locally, prepare to download
            if (!fs_1.default.existsSync(libFilePath)) {
                const libUrl = `${lib.url}${libInfo.path}/${libInfo.name}`;
                let fileSize = 0;
                const checkResult = await downloader.checkURL(libUrl);
                if (checkResult && checkResult.status === 200) {
                    fileSize = checkResult.size;
                    totalSize += fileSize;
                }
                filesToDownload.push({
                    url: libUrl,
                    folder: libFolder,
                    path: libFilePath,
                    name: libInfo.name,
                    size: fileSize
                });
            }
            // Emit a "check" event for each library
            this.emit('check', checkedLibraries++, libraries.length, 'libraries');
        }
        // If there are libraries to download, proceed with the bulk download
        if (filesToDownload.length > 0) {
            downloader.on('progress', (downloaded, total) => {
                this.emit('progress', downloaded, total, 'libraries');
            });
            await downloader.downloadFileMultiple(filesToDownload, totalSize, this.options.downloadFileMultiple);
        }
        return libraries;
    }
}
exports.default = Quilt;
//# sourceMappingURL=quilt.js.map
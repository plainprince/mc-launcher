#!/bin/bash

echo "Starting Minecraft Launcher (NW.js)..."
echo "======================================="

cd nw-launcher

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    bun install
fi

echo "Launching application..."
bun run start

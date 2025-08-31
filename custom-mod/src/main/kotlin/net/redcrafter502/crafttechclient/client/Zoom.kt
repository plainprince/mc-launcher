package net.redcrafter502.crafttechclient.client

import net.minecraft.client.MinecraftClient
import net.minecraft.util.math.MathHelper

class Zoom {
    companion object {
        private const val defaultLevel: Double = 3.0
        private var currentLevel: Double = defaultLevel
        private var originalFov: Double? = null
        private var isInitialized: Boolean = false

        fun initialize() {
            isInitialized = true
        }

        fun isZooming(): Boolean {
            return isInitialized && KeyBindings.zoomKeyBinding.isPressed
        }

        fun onClientTick(client: MinecraftClient) {
            if (!isInitialized) return
            
            val options = client.options
            
            // Handle zoom level adjustments
            if (isZooming()) {
                // Handle zoom in/out keys
                while (KeyBindings.zoomInKeyBinding.wasPressed()) {
                    currentLevel = MathHelper.clamp(currentLevel * 1.2, 1.0, 50.0)
                }
                while (KeyBindings.zoomOutKeyBinding.wasPressed()) {
                    currentLevel = MathHelper.clamp(currentLevel * 0.8, 1.0, 50.0)
                }
            }
            
            if (isZooming()) {
                // Store original FOV if we haven't already
                if (originalFov == null) {
                    originalFov = options.fov.value.toDouble()
                }
                
                // Apply zoom by modifying FOV
                val zoomedFov = originalFov!! / currentLevel
                val clampedFov = MathHelper.clamp(zoomedFov, 1.0, 110.0)
                options.fov.setValue(clampedFov.toInt())
            } else {
                // Restore original FOV when not zooming
                if (originalFov != null) {
                    options.fov.setValue(originalFov!!.toInt())
                    originalFov = null
                }
                // Reset zoom level
                currentLevel = defaultLevel
            }
        }

        fun onMouseScroll(amount: Double) {
            if (!isZooming()) {
                return
            }

            if (amount > 0) {
                currentLevel = currentLevel * 1.1
            } else if (amount < 0) {
                currentLevel = currentLevel * 0.9
            }
            currentLevel = MathHelper.clamp(currentLevel, 1.0, 50.0)
        }
    }
}
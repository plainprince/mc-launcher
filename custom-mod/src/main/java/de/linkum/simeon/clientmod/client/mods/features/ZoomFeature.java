package de.linkum.simeon.clientmod.client.mods.features;

import de.linkum.simeon.clientmod.client.mods.ModFeature;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;
import net.minecraft.util.math.MathHelper;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;

public class ZoomFeature extends ModFeature {
    // Zoom configuration
    private static final double MIN_ZOOM = 1.0;
    private static final double MAX_ZOOM = 30.0;
    private static final double DEFAULT_ZOOM = 4.0;
    private static final double SCROLL_SENSITIVITY = 1.1;
    private static final double SMOOTH_FACTOR = 0.2; // For smooth transitions
    
    // Zoom state
    private double targetZoomLevel = DEFAULT_ZOOM;
    private double currentZoomLevel = 1.0; // Smooth interpolated value
    private double originalFov = 70.0;
    private boolean isZooming = false;
    private boolean wasZooming = false;
    
    // Key binding
    private KeyBinding zoomKeyBinding;
    
    // Smooth transition
    private boolean smoothTransitions = true;
    
    public ZoomFeature() {
        super("Zoom");
    }
    
    @Override
    protected void onEnable() {
        // Initialize zoom key binding
        zoomKeyBinding = KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.clientmod.zoom",
            InputUtil.Type.KEYSYM,
            InputUtil.GLFW_KEY_C,
            "category.clientmod.general"
        ));
        
        // Store original FOV
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.options != null) {
            originalFov = client.options.getFov().getValue();
        }
    }
    
    @Override
    protected void onDisable() {
        // Restore original FOV immediately
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.options != null) {
            client.options.getFov().setValue((int) originalFov);
        }
        
        // Reset zoom state
        isZooming = false;
        wasZooming = false;
        currentZoomLevel = 1.0;
        targetZoomLevel = DEFAULT_ZOOM;
    }
    
    @Override
    public void tick() {
        if (!enabled) return;
        
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.options == null) return;
        
        // Update zoom state
        boolean shouldZoom = zoomKeyBinding != null && zoomKeyBinding.isPressed();
        
        if (shouldZoom != isZooming) {
            isZooming = shouldZoom;
            if (!isZooming) {
                // Reset target zoom when stopping zoom
                targetZoomLevel = DEFAULT_ZOOM;
            }
        }
        
        // Calculate target zoom level
        double targetLevel = isZooming ? targetZoomLevel : 1.0;
        
        // Smooth interpolation
        if (smoothTransitions) {
            double delta = targetLevel - currentZoomLevel;
            if (Math.abs(delta) > 0.01) {
                currentZoomLevel += delta * SMOOTH_FACTOR;
            } else {
                currentZoomLevel = targetLevel;
            }
        } else {
            currentZoomLevel = targetLevel;
        }
        
        // Apply zoom through FOV modification
        if (currentZoomLevel > 1.01 || wasZooming) {
            double zoomedFov = originalFov / currentZoomLevel;
            // Clamp to prevent extreme values
            zoomedFov = MathHelper.clamp(zoomedFov, 1.0, originalFov);
            client.options.getFov().setValue((int) Math.round(zoomedFov));
            wasZooming = true;
        } else if (wasZooming && currentZoomLevel <= 1.01) {
            // Restore original FOV when zoom is fully disabled
            client.options.getFov().setValue((int) originalFov);
            wasZooming = false;
        }
    }
    
    public boolean onMouseScroll(double amount) {
        if (!enabled || !isZooming) {
            return false;
        }
        
        // Calculate zoom adjustment
        double scrollFactor = amount > 0 ? SCROLL_SENSITIVITY : (1.0 / SCROLL_SENSITIVITY);
        targetZoomLevel = MathHelper.clamp(targetZoomLevel * scrollFactor, MIN_ZOOM, MAX_ZOOM);
        
        return true; // Consume the scroll event
    }
    
    // Getters for external access
    public boolean isZooming() {
        return enabled && isZooming;
    }
    
    public double getCurrentZoomLevel() {
        return currentZoomLevel;
    }
    
    public double getTargetZoomLevel() {
        return targetZoomLevel;
    }
    
    // Configuration methods
    public void setSmoothTransitions(boolean smooth) {
        this.smoothTransitions = smooth;
    }
    
    public boolean isSmoothTransitions() {
        return smoothTransitions;
    }
    
    public void setZoomLevel(double level) {
        this.targetZoomLevel = MathHelper.clamp(level, MIN_ZOOM, MAX_ZOOM);
    }
}

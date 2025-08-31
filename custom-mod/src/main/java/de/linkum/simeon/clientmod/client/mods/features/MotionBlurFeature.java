package de.linkum.simeon.clientmod.client.mods.features;

import de.linkum.simeon.clientmod.client.mods.ModFeature;

public class MotionBlurFeature extends ModFeature {
    // Motion blur configuration
    private static final float MIN_BLUR_STRENGTH = 0.1f;
    private static final float MAX_BLUR_STRENGTH = 1.0f;
    private static final float DEFAULT_BLUR_STRENGTH = 0.6f;
    
    // Motion blur state
    private float blurStrength = DEFAULT_BLUR_STRENGTH;
    private float blurFactor = 0.7f; // Multiplier for frame blending
    private boolean adaptiveBlur = true; // Adjust blur based on movement speed
    
    // Performance settings
    private int maxBlurSamples = 8; // Maximum samples for blur effect
    private boolean reduceInCombat = true; // Reduce blur during PvP for better visibility
    
    public MotionBlurFeature() {
        super("Motion Blur");
    }
    
    @Override
    protected void onEnable() {
        // Motion blur will be implemented in rendering mixins
        // Initialize any necessary rendering state here
    }
    
    @Override
    protected void onDisable() {
        // Disable motion blur rendering
        // Clean up any rendering state here
    }
    
    @Override
    public void tick() {
        if (!enabled) return;
        
        // Update motion blur based on current conditions
        if (reduceInCombat) {
            // TODO: Detect if player is in combat and reduce blur accordingly
            // This could check for recent damage, nearby players, etc.
        }
    }
    
    // Configuration getters and setters
    public float getBlurStrength() {
        return blurStrength;
    }
    
    public void setBlurStrength(float strength) {
        this.blurStrength = Math.max(MIN_BLUR_STRENGTH, Math.min(MAX_BLUR_STRENGTH, strength));
    }
    
    public float getBlurFactor() {
        return blurFactor;
    }
    
    public void setBlurFactor(float factor) {
        this.blurFactor = Math.max(0.1f, Math.min(0.9f, factor));
    }
    
    public boolean isAdaptiveBlur() {
        return adaptiveBlur;
    }
    
    public void setAdaptiveBlur(boolean adaptive) {
        this.adaptiveBlur = adaptive;
    }
    
    public int getMaxBlurSamples() {
        return maxBlurSamples;
    }
    
    public void setMaxBlurSamples(int samples) {
        this.maxBlurSamples = Math.max(2, Math.min(16, samples));
    }
    
    public boolean isReduceInCombat() {
        return reduceInCombat;
    }
    
    public void setReduceInCombat(boolean reduce) {
        this.reduceInCombat = reduce;
    }
    
    // Utility methods for rendering
    public float getEffectiveBlurStrength() {
        // Return the actual blur strength to use, considering all factors
        float effective = blurStrength;
        
        if (adaptiveBlur) {
            // TODO: Adjust based on movement speed
            // effective *= calculateMovementFactor();
        }
        
        if (reduceInCombat) {
            // TODO: Reduce if in combat
            // if (isInCombat()) effective *= 0.5f;
        }
        
        return effective;
    }
    
    // Method to calculate current blur intensity based on camera movement
    public float calculateBlurIntensity(float deltaYaw, float deltaPitch) {
        if (!enabled) return 0.0f;
        
        // Calculate blur based on camera rotation speed
        float rotationSpeed = (float) Math.sqrt(deltaYaw * deltaYaw + deltaPitch * deltaPitch);
        float normalizedSpeed = Math.min(rotationSpeed / 10.0f, 1.0f); // Normalize to 0-1
        
        return getEffectiveBlurStrength() * normalizedSpeed * blurFactor;
    }
}

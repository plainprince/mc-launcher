package de.linkum.simeon.clientmod.client.rendering;

public class CosmeticsManager {
    private static CosmeticsManager instance;
    
    private boolean capeEnabled = false;
    private boolean wingsEnabled = false;
    private boolean particlesEnabled = false;
    
    private CosmeticsManager() {}
    
    public static CosmeticsManager getInstance() {
        if (instance == null) {
            instance = new CosmeticsManager();
        }
        return instance;
    }
    
    public boolean isCapeEnabled() {
        return capeEnabled;
    }
    
    public void toggleCape() {
        capeEnabled = !capeEnabled;
    }
    
    public void setCapeEnabled(boolean enabled) {
        capeEnabled = enabled;
    }
    
    public boolean areWingsEnabled() {
        return wingsEnabled;
    }
    
    public void toggleWings() {
        wingsEnabled = !wingsEnabled;
    }
    
    public void setWingsEnabled(boolean enabled) {
        wingsEnabled = enabled;
    }
    
    public boolean areParticlesEnabled() {
        return particlesEnabled;
    }
    
    public void toggleParticles() {
        particlesEnabled = !particlesEnabled;
    }
    
    public void setParticlesEnabled(boolean enabled) {
        particlesEnabled = enabled;
    }
}

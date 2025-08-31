package de.linkum.simeon.clientmod.client.mods;

public abstract class ModFeature {
    protected boolean enabled = false;
    protected final String name;
    
    public ModFeature(String name) {
        this.name = name;
    }
    
    public String getName() {
        return name;
    }
    
    public boolean isEnabled() {
        return enabled;
    }
    
    public void setEnabled(boolean enabled) {
        if (this.enabled != enabled) {
            this.enabled = enabled;
            if (enabled) {
                onEnable();
            } else {
                onDisable();
            }
        }
    }
    
    public void toggle() {
        setEnabled(!enabled);
    }
    
    protected abstract void onEnable();
    protected abstract void onDisable();
    
    public void tick() {
        // Override in subclasses if needed
    }
}

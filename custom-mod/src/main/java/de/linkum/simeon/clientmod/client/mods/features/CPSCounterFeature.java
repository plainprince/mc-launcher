package de.linkum.simeon.clientmod.client.mods.features;

import de.linkum.simeon.clientmod.client.mods.ModFeature;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;

import java.util.ArrayList;
import java.util.List;

public class CPSCounterFeature extends ModFeature {
    // Click tracking
    private final List<Long> leftClicks = new ArrayList<>();
    private final List<Long> rightClicks = new ArrayList<>();
    private static final long CLICK_TIMEOUT = 1000; // 1 second
    
    // Display configuration
    private String displayFormat = "CPS: $c"; // Default format
    private boolean showSeparate = false; // Show left/right separately
    private String separateFormat = "CPS: [$l|$r]"; // Format for separate display
    
    // Position and styling
    private int x = 10;
    private int y = 70;
    private int color = 0xFFFFFF; // White
    private boolean shadow = true;
    
    // Current CPS values
    private int leftCPS = 0;
    private int rightCPS = 0;
    private int combinedCPS = 0;
    
    public CPSCounterFeature() {
        super("CPS Counter");
    }
    
    @Override
    protected void onEnable() {
        // Clear existing clicks when enabling
        leftClicks.clear();
        rightClicks.clear();
    }
    
    @Override
    protected void onDisable() {
        // Clear clicks when disabling
        leftClicks.clear();
        rightClicks.clear();
        leftCPS = 0;
        rightCPS = 0;
        combinedCPS = 0;
    }
    
    @Override
    public void tick() {
        if (!enabled) return;
        
        long currentTime = System.currentTimeMillis();
        
        // Remove old clicks (older than 1 second)
        leftClicks.removeIf(clickTime -> currentTime - clickTime > CLICK_TIMEOUT);
        rightClicks.removeIf(clickTime -> currentTime - clickTime > CLICK_TIMEOUT);
        
        // Update CPS values
        leftCPS = leftClicks.size();
        rightCPS = rightClicks.size();
        combinedCPS = leftCPS + rightCPS;
    }
    
    // Method to register clicks (called from mouse mixin)
    public void registerLeftClick() {
        if (enabled) {
            leftClicks.add(System.currentTimeMillis());
        }
    }
    
    public void registerRightClick() {
        if (enabled) {
            rightClicks.add(System.currentTimeMillis());
        }
    }
    
    // Display methods
    public String getDisplayText() {
        if (!enabled) return "";
        
        String format = showSeparate ? separateFormat : displayFormat;
        
        return format
                .replace("$l", String.valueOf(leftCPS))
                .replace("$r", String.valueOf(rightCPS))
                .replace("$c", String.valueOf(combinedCPS));
    }
    
    // Configuration getters and setters
    public String getDisplayFormat() {
        return displayFormat;
    }
    
    public void setDisplayFormat(String format) {
        this.displayFormat = format != null ? format : "CPS: $c";
    }
    
    public boolean isShowSeparate() {
        return showSeparate;
    }
    
    public void setShowSeparate(boolean separate) {
        this.showSeparate = separate;
    }
    
    public String getSeparateFormat() {
        return separateFormat;
    }
    
    public void setSeparateFormat(String format) {
        this.separateFormat = format != null ? format : "CPS: [$l|$r]";
    }
    
    public int getX() {
        return x;
    }
    
    public void setX(int x) {
        this.x = x;
    }
    
    public int getY() {
        return y;
    }
    
    public void setY(int y) {
        this.y = y;
    }
    
    public int getColor() {
        return color;
    }
    
    public void setColor(int color) {
        this.color = color;
    }
    
    public boolean hasShadow() {
        return shadow;
    }
    
    public void setShadow(boolean shadow) {
        this.shadow = shadow;
    }
    
    // Current values getters
    public int getLeftCPS() {
        return leftCPS;
    }
    
    public int getRightCPS() {
        return rightCPS;
    }
    
    public int getCombinedCPS() {
        return combinedCPS;
    }
}

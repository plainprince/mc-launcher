package de.linkum.simeon.clientmod.client.mods.features;

import de.linkum.simeon.clientmod.client.mods.ModFeature;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.KeyBinding;

import java.util.HashMap;
import java.util.Map;

public class KeystrokesFeature extends ModFeature {
    // Display modes
    public enum DisplayMode {
        WASD("WASD", new String[]{"W", "A", "S", "D"}),
        WASD_SPACE("WASD + Space", new String[]{"W", "A", "S", "D", "Space"}),
        WASD_SHIFT("WASD + Shift", new String[]{"W", "A", "S", "D", "Shift"}),
        WASD_SPACE_SHIFT("WASD + Space + Shift", new String[]{"W", "A", "S", "D", "Space", "Shift"});
        
        public final String displayName;
        public final String[] keys;
        
        DisplayMode(String displayName, String[] keys) {
            this.displayName = displayName;
            this.keys = keys;
        }
    }
    
    // Configuration
    private DisplayMode displayMode = DisplayMode.WASD_SPACE;
    private int x = 10;
    private int y = 90;
    
    // Colors
    private int normalColor = 0x80000000; // Semi-transparent black
    private int pressedColor = 0x80FFFFFF; // Semi-transparent white
    private int textColor = 0xFFFFFF; // White text
    private int pressedTextColor = 0x000000; // Black text when pressed
    
    // Layout
    private int keySize = 20;
    private int keySpacing = 2;
    private boolean showKeyLabels = true;
    
    // Key state tracking
    private final Map<String, Boolean> keyStates = new HashMap<>();
    private final Map<String, KeyBinding> keyBindings = new HashMap<>();
    
    public KeystrokesFeature() {
        super("Keystrokes");
    }
    
    @Override
    protected void onEnable() {
        updateKeyBindings();
        initializeKeyStates();
    }
    
    @Override
    protected void onDisable() {
        keyStates.clear();
        keyBindings.clear();
    }
    
    @Override
    public void tick() {
        if (!enabled) return;
        
        // Update key states
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.options == null) return;
        
        // Update movement keys
        keyStates.put("W", client.options.forwardKey.isPressed());
        keyStates.put("A", client.options.leftKey.isPressed());
        keyStates.put("S", client.options.backKey.isPressed());
        keyStates.put("D", client.options.rightKey.isPressed());
        
        // Update special keys if they're in the display mode
        for (String key : displayMode.keys) {
            switch (key) {
                case "Space":
                    keyStates.put("Space", client.options.jumpKey.isPressed());
                    break;
                case "Shift":
                    keyStates.put("Shift", client.options.sneakKey.isPressed());
                    break;
            }
        }
    }
    
    private void updateKeyBindings() {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.options == null) return;
        
        keyBindings.put("W", client.options.forwardKey);
        keyBindings.put("A", client.options.leftKey);
        keyBindings.put("S", client.options.backKey);
        keyBindings.put("D", client.options.rightKey);
        keyBindings.put("Space", client.options.jumpKey);
        keyBindings.put("Shift", client.options.sneakKey);
    }
    
    private void initializeKeyStates() {
        for (String key : displayMode.keys) {
            keyStates.put(key, false);
        }
    }
    
    // Rendering helper methods
    public KeyRenderInfo[] getKeysToRender() {
        if (!enabled) return new KeyRenderInfo[0];
        
        String[] keys = displayMode.keys;
        KeyRenderInfo[] renderInfo = new KeyRenderInfo[keys.length];
        
        for (int i = 0; i < keys.length; i++) {
            String key = keys[i];
            boolean pressed = keyStates.getOrDefault(key, false);
            
            // Calculate position based on key layout
            int keyX = x;
            int keyY = y;
            
            switch (key) {
                case "W":
                    keyX = x + keySize + keySpacing;
                    keyY = y;
                    break;
                case "A":
                    keyX = x;
                    keyY = y + keySize + keySpacing;
                    break;
                case "S":
                    keyX = x + keySize + keySpacing;
                    keyY = y + keySize + keySpacing;
                    break;
                case "D":
                    keyX = x + (keySize + keySpacing) * 2;
                    keyY = y + keySize + keySpacing;
                    break;
                case "Space":
                    keyX = x;
                    keyY = y + (keySize + keySpacing) * 2;
                    break;
                case "Shift":
                    keyX = x + (keySize + keySpacing) * 2;
                    keyY = y + (keySize + keySpacing) * 2;
                    break;
            }
            
            String displayText = getKeyDisplayText(key);
            
            renderInfo[i] = new KeyRenderInfo(
                key,
                keyX, keyY,
                keySize, keySize,
                pressed ? pressedColor : normalColor,
                pressed ? pressedTextColor : textColor,
                displayText,
                pressed
            );
        }
        
        return renderInfo;
    }
    
    private String getKeyDisplayText(String key) {
        if (!showKeyLabels) return "";
        
        MinecraftClient client = MinecraftClient.getInstance();
        KeyBinding binding = keyBindings.get(key);
        
        if (binding != null) {
            // Get the actual key name from the binding
            String boundKey = binding.getBoundKeyLocalizedText().getString();
            
            // Simplify common key names
            switch (boundKey.toUpperCase()) {
                case "SPACE":
                    return "___";
                case "LEFT SHIFT":
                case "RIGHT SHIFT":
                    return "SHIFT";
                default:
                    return boundKey.length() > 4 ? boundKey.substring(0, 4) : boundKey;
            }
        }
        
        return key;
    }
    
    // Configuration getters and setters
    public DisplayMode getDisplayMode() {
        return displayMode;
    }
    
    public void setDisplayMode(DisplayMode mode) {
        this.displayMode = mode;
        if (enabled) {
            initializeKeyStates();
        }
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
    
    public int getNormalColor() {
        return normalColor;
    }
    
    public void setNormalColor(int color) {
        this.normalColor = color;
    }
    
    public int getPressedColor() {
        return pressedColor;
    }
    
    public void setPressedColor(int color) {
        this.pressedColor = color;
    }
    
    public int getTextColor() {
        return textColor;
    }
    
    public void setTextColor(int color) {
        this.textColor = color;
    }
    
    public int getPressedTextColor() {
        return pressedTextColor;
    }
    
    public void setPressedTextColor(int color) {
        this.pressedTextColor = color;
    }
    
    public int getKeySize() {
        return keySize;
    }
    
    public void setKeySize(int size) {
        this.keySize = Math.max(10, Math.min(50, size));
    }
    
    public int getKeySpacing() {
        return keySpacing;
    }
    
    public void setKeySpacing(int spacing) {
        this.keySpacing = Math.max(0, Math.min(10, spacing));
    }
    
    public boolean isShowKeyLabels() {
        return showKeyLabels;
    }
    
    public void setShowKeyLabels(boolean show) {
        this.showKeyLabels = show;
    }
    
    // Helper class for rendering information
    public static class KeyRenderInfo {
        public final String key;
        public final int x, y, width, height;
        public final int backgroundColor, textColor;
        public final String displayText;
        public final boolean pressed;
        
        public KeyRenderInfo(String key, int x, int y, int width, int height, 
                           int backgroundColor, int textColor, String displayText, boolean pressed) {
            this.key = key;
            this.x = x;
            this.y = y;
            this.width = width;
            this.height = height;
            this.backgroundColor = backgroundColor;
            this.textColor = textColor;
            this.displayText = displayText;
            this.pressed = pressed;
        }
    }
}

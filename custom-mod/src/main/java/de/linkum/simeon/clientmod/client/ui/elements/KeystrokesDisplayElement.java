package de.linkum.simeon.clientmod.client.ui.elements;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.features.KeystrokesFeature;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

public class KeystrokesDisplayElement extends DraggableUIElement {
    
    public KeystrokesDisplayElement(int x, int y) {
        super("Keystrokes", x, y, 70, 70); // Default size for WASD layout
    }
    
    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        if (!visible) return;
        
        KeystrokesFeature keystrokesFeature = (KeystrokesFeature) ModManager.getInstance().getFeature("Keystrokes");
        if (keystrokesFeature == null || !keystrokesFeature.isEnabled()) return;
        
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.textRenderer == null) return;
        
        KeystrokesFeature.KeyRenderInfo[] keys = keystrokesFeature.getKeysToRender();
        if (keys.length == 0) return;
        
        // Calculate total bounds
        int minX = Integer.MAX_VALUE, minY = Integer.MAX_VALUE;
        int maxX = Integer.MIN_VALUE, maxY = Integer.MIN_VALUE;
        
        for (KeystrokesFeature.KeyRenderInfo key : keys) {
            minX = Math.min(minX, key.x);
            minY = Math.min(minY, key.y);
            maxX = Math.max(maxX, key.x + key.width);
            maxY = Math.max(maxY, key.y + key.height);
        }
        
        // Update element size
        this.width = maxX - minX;
        this.height = maxY - minY;
        
        // Calculate offset to render relative to this element's position
        int offsetX = x - minX;
        int offsetY = y - minY;
        
        // Render each key
        for (KeystrokesFeature.KeyRenderInfo key : keys) {
            int keyX = key.x + offsetX;
            int keyY = key.y + offsetY;
            
            // Draw key background
            context.fill(keyX, keyY, keyX + key.width, keyY + key.height, key.backgroundColor);
            
            // Draw key border
            context.drawBorder(keyX, keyY, key.width, key.height, 0xFF444444);
            
            // Draw key text
            if (!key.displayText.isEmpty()) {
                int textX = keyX + (key.width - client.textRenderer.getWidth(key.displayText)) / 2;
                int textY = keyY + (key.height - client.textRenderer.fontHeight) / 2;
                
                context.drawText(client.textRenderer, key.displayText, textX, textY, key.textColor, false);
            }
        }
    }
    
    @Override
    public void setPosition(int x, int y) {
        super.setPosition(x, y);
        
        // Update the keystrokes feature position
        KeystrokesFeature keystrokesFeature = (KeystrokesFeature) ModManager.getInstance().getFeature("Keystrokes");
        if (keystrokesFeature != null) {
            keystrokesFeature.setX(x);
            keystrokesFeature.setY(y);
        }
    }
}

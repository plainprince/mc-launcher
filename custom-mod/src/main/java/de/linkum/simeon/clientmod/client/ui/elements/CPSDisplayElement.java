package de.linkum.simeon.clientmod.client.ui.elements;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.features.CPSCounterFeature;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

public class CPSDisplayElement extends DraggableUIElement {
    
    public CPSDisplayElement(int x, int y) {
        super("CPS Counter", x, y, 100, 10);
    }
    
    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        if (!visible) return;
        
        CPSCounterFeature cpsFeature = (CPSCounterFeature) ModManager.getInstance().getFeature("CPS Counter");
        if (cpsFeature == null || !cpsFeature.isEnabled()) return;
        
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.textRenderer == null) return;
        
        String displayText = cpsFeature.getDisplayText();
        if (displayText.isEmpty()) return;
        
        if (cpsFeature.hasShadow()) {
            context.drawTextWithShadow(client.textRenderer, displayText, x, y, cpsFeature.getColor());
        } else {
            context.drawText(client.textRenderer, displayText, x, y, cpsFeature.getColor(), false);
        }
        
        // Update width based on text
        this.width = client.textRenderer.getWidth(displayText);
        this.height = client.textRenderer.fontHeight;
    }
    
    @Override
    public void setPosition(int x, int y) {
        super.setPosition(x, y);
        
        // Update the CPS feature position
        CPSCounterFeature cpsFeature = (CPSCounterFeature) ModManager.getInstance().getFeature("CPS Counter");
        if (cpsFeature != null) {
            cpsFeature.setX(x);
            cpsFeature.setY(y);
        }
    }
}

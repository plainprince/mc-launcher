package de.linkum.simeon.clientmod.client.ui.elements;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;

public class TextUIElement extends DraggableUIElement {
    private String text;
    private int color;
    
    public TextUIElement(String name, int x, int y, String text) {
        this(name, x, y, text, 0xFFFFFF);
    }
    
    public TextUIElement(String name, int x, int y, String text, int color) {
        super(name, x, y, 100, 10); // Default size, will be adjusted based on text
        this.text = text;
        this.color = color;
        updateSize();
    }
    
    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        if (!visible) return;
        
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.textRenderer != null) {
            context.drawTextWithShadow(client.textRenderer, text, x, y, color);
        }
    }
    
    public void setText(String text) {
        this.text = text;
        updateSize();
    }
    
    public void setColor(int color) {
        this.color = color;
    }
    
    private void updateSize() {
        MinecraftClient client = MinecraftClient.getInstance();
        if (client.textRenderer != null) {
            this.width = client.textRenderer.getWidth(text);
            this.height = client.textRenderer.fontHeight;
        }
    }
    
    public String getText() {
        return text;
    }
}

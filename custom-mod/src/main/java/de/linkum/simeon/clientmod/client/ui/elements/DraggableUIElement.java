package de.linkum.simeon.clientmod.client.ui.elements;

import net.minecraft.client.gui.DrawContext;

public abstract class DraggableUIElement {
    protected int x, y;
    protected int width, height;
    protected String name;
    protected boolean visible = true;
    
    public DraggableUIElement(String name, int x, int y, int width, int height) {
        this.name = name;
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }
    
    public abstract void render(DrawContext context, int mouseX, int mouseY, float delta);
    
    public boolean isMouseOver(int mouseX, int mouseY) {
        return mouseX >= x && mouseX <= x + width && mouseY >= y && mouseY <= y + height;
    }
    
    public void setPosition(int x, int y) {
        this.x = x;
        this.y = y;
    }
    
    public int getX() { return x; }
    public int getY() { return y; }
    public int getWidth() { return width; }
    public int getHeight() { return height; }
    public String getName() { return name; }
    public boolean isVisible() { return visible; }
    public void setVisible(boolean visible) { this.visible = visible; }
}

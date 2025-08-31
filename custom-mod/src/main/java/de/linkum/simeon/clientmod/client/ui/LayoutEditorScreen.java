package de.linkum.simeon.clientmod.client.ui;

import de.linkum.simeon.clientmod.client.ui.elements.DraggableUIElement;
import de.linkum.simeon.clientmod.client.ui.elements.UIElementManager;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

import java.util.List;

public class LayoutEditorScreen extends Screen {
    private final Screen parent;
    private final UIElementManager uiManager;
    private boolean dragMode = false;
    private DraggableUIElement draggedElement = null;
    private int dragOffsetX = 0;
    private int dragOffsetY = 0;
    
    public LayoutEditorScreen(Screen parent) {
        super(Text.literal("Layout Editor"));
        this.parent = parent;
        this.uiManager = UIElementManager.getInstance();
    }

    @Override
    protected void init() {
        super.init();
        
        // Toggle drag mode button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Drag Mode: " + (dragMode ? "ON" : "OFF")), 
            button -> {
                dragMode = !dragMode;
                button.setMessage(Text.literal("Drag Mode: " + (dragMode ? "ON" : "OFF")));
            }
        ).dimensions(10, 10, 120, 20).build());
        
        // Reset positions button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Reset All"), 
            button -> uiManager.resetAllPositions()
        ).dimensions(140, 10, 100, 20).build());
        
        // Back button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Back"), 
            button -> this.close()
        ).dimensions(this.width - 110, this.height - 30, 100, 20).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, this.width / 2, 20, 0xFFFFFF);
        
        // Render UI elements
        List<DraggableUIElement> elements = uiManager.getAllElements();
        for (DraggableUIElement element : elements) {
            element.render(context, mouseX, mouseY, delta);
            
            // Highlight draggable elements in drag mode
            if (dragMode) {
                context.fill(element.getX() - 2, element.getY() - 2, 
                           element.getX() + element.getWidth() + 2, 
                           element.getY() + element.getHeight() + 2, 
                           0x44FFFF00);
            }
        }
        
        super.render(context, mouseX, mouseY, delta);
        
        // Instructions
        if (dragMode) {
            context.drawTextWithShadow(this.textRenderer, "Click and drag UI elements to reposition", 10, this.height - 20, 0xFFFFFF);
        }
    }

    @Override
    public boolean mouseClicked(double mouseX, double mouseY, int button) {
        if (dragMode && button == 0) {
            List<DraggableUIElement> elements = uiManager.getAllElements();
            for (DraggableUIElement element : elements) {
                if (element.isMouseOver((int)mouseX, (int)mouseY)) {
                    draggedElement = element;
                    dragOffsetX = (int)mouseX - element.getX();
                    dragOffsetY = (int)mouseY - element.getY();
                    return true;
                }
            }
        }
        
        return super.mouseClicked(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseReleased(double mouseX, double mouseY, int button) {
        if (draggedElement != null && button == 0) {
            draggedElement = null;
            return true;
        }
        
        return super.mouseReleased(mouseX, mouseY, button);
    }

    @Override
    public boolean mouseDragged(double mouseX, double mouseY, int button, double deltaX, double deltaY) {
        if (draggedElement != null && button == 0) {
            draggedElement.setPosition((int)mouseX - dragOffsetX, (int)mouseY - dragOffsetY);
            return true;
        }
        
        return super.mouseDragged(mouseX, mouseY, button, deltaX, deltaY);
    }

    @Override
    public void close() {
        this.client.setScreen(parent);
    }

    @Override
    public boolean shouldPause() {
        return false;
    }
}

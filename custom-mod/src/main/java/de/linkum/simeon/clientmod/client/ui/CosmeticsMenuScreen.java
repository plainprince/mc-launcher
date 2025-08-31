package de.linkum.simeon.clientmod.client.ui;

import de.linkum.simeon.clientmod.client.rendering.CosmeticsManager;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public class CosmeticsMenuScreen extends Screen {
    private final Screen parent;
    private final CosmeticsManager cosmeticsManager;
    
    public CosmeticsMenuScreen(Screen parent) {
        super(Text.literal("Cosmetics"));
        this.parent = parent;
        this.cosmeticsManager = CosmeticsManager.getInstance();
    }

    @Override
    protected void init() {
        super.init();
        
        int centerX = this.width / 2;
        int startY = 50;
        int buttonHeight = 20;
        int buttonSpacing = 25;
        
        // Cosmetic toggles
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Cape: " + (cosmeticsManager.isCapeEnabled() ? "ON" : "OFF")), 
            button -> {
                cosmeticsManager.toggleCape();
                button.setMessage(Text.literal("Cape: " + (cosmeticsManager.isCapeEnabled() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY, 200, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Wings: " + (cosmeticsManager.areWingsEnabled() ? "ON" : "OFF")), 
            button -> {
                cosmeticsManager.toggleWings();
                button.setMessage(Text.literal("Wings: " + (cosmeticsManager.areWingsEnabled() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY + buttonSpacing, 200, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Particles: " + (cosmeticsManager.areParticlesEnabled() ? "ON" : "OFF")), 
            button -> {
                cosmeticsManager.toggleParticles();
                button.setMessage(Text.literal("Particles: " + (cosmeticsManager.areParticlesEnabled() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY + buttonSpacing * 2, 200, buttonHeight).build());
        
        // Back button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Back"), 
            button -> this.close()
        ).dimensions(centerX - 50, this.height - 40, 100, buttonHeight).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, this.width / 2, 20, 0xFFFFFF);
        super.render(context, mouseX, mouseY, delta);
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

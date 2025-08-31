package de.linkum.simeon.clientmod.client.ui;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;
import net.minecraft.util.Identifier;

public class MainMenuScreen extends Screen {
    private static final Identifier DEFAULT_LOGO = Identifier.of("clientmod", "textures/gui/logo.png");
    private final Screen parent;
    
    public MainMenuScreen(Screen parent) {
        super(Text.literal("ClientMod Menu"));
        this.parent = parent;
    }

    @Override
    protected void init() {
        super.init();
        
        int centerX = this.width / 2;
        int centerY = this.height / 2;
        
        // Logo area (customizable SVG would be rendered here)
        // For now, we'll just have a placeholder
        
        // Three main buttons
        int buttonWidth = 120;
        int buttonHeight = 20;
        int buttonSpacing = 130;
        
        // Cosmetics Menu (Left)
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Cosmetics"), 
            button -> this.client.setScreen(new CosmeticsMenuScreen(this))
        ).dimensions(centerX - buttonSpacing, centerY + 50, buttonWidth, buttonHeight).build());
        
        // Mods Menu (Center)
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Mods"), 
            button -> this.client.setScreen(new ModsMenuScreen(this))
        ).dimensions(centerX - buttonWidth/2, centerY + 50, buttonWidth, buttonHeight).build());
        
        // Layout Editor (Right)
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Layout"), 
            button -> this.client.setScreen(new LayoutEditorScreen(this))
        ).dimensions(centerX + buttonSpacing - buttonWidth, centerY + 50, buttonWidth, buttonHeight).build());
        
        // Close button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Close"), 
            button -> this.close()
        ).dimensions(centerX - 50, centerY + 90, 100, buttonHeight).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        
        int centerX = this.width / 2;
        int centerY = this.height / 2;
        
        // Draw title
        context.drawCenteredTextWithShadow(this.textRenderer, this.title, centerX, 20, 0xFFFFFF);
        
        // Draw logo placeholder (this would be replaced with SVG rendering)
        context.fill(centerX - 50, centerY - 50, centerX + 50, centerY - 10, 0x88000000);
        context.drawCenteredTextWithShadow(this.textRenderer, Text.literal("LOGO"), centerX, centerY - 35, 0xFFFFFF);
        
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

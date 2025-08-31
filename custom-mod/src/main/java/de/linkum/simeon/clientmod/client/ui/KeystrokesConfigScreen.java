package de.linkum.simeon.clientmod.client.ui;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.features.KeystrokesFeature;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public class KeystrokesConfigScreen extends Screen {
    private final Screen parent;
    private final KeystrokesFeature keystrokesFeature;
    
    public KeystrokesConfigScreen(Screen parent) {
        super(Text.literal("Keystrokes Config"));
        this.parent = parent;
        this.keystrokesFeature = (KeystrokesFeature) ModManager.getInstance().getFeature("Keystrokes");
    }

    @Override
    protected void init() {
        super.init();
        
        if (keystrokesFeature == null) return;
        
        int centerX = this.width / 2;
        int startY = 50;
        int buttonHeight = 20;
        int spacing = 25;
        
        // Display mode selection
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Mode: " + keystrokesFeature.getDisplayMode().displayName), 
            button -> {
                KeystrokesFeature.DisplayMode[] modes = KeystrokesFeature.DisplayMode.values();
                KeystrokesFeature.DisplayMode current = keystrokesFeature.getDisplayMode();
                int currentIndex = 0;
                for (int i = 0; i < modes.length; i++) {
                    if (modes[i] == current) {
                        currentIndex = i;
                        break;
                    }
                }
                int nextIndex = (currentIndex + 1) % modes.length;
                keystrokesFeature.setDisplayMode(modes[nextIndex]);
                button.setMessage(Text.literal("Mode: " + keystrokesFeature.getDisplayMode().displayName));
            }
        ).dimensions(centerX - 120, startY, 240, buttonHeight).build());
        
        // Key size
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Key Size: " + keystrokesFeature.getKeySize()), 
            button -> {
                int newSize = keystrokesFeature.getKeySize() + 5;
                if (newSize > 50) newSize = 10;
                keystrokesFeature.setKeySize(newSize);
                button.setMessage(Text.literal("Key Size: " + keystrokesFeature.getKeySize()));
            }
        ).dimensions(centerX - 100, startY + spacing, 200, buttonHeight).build());
        
        // Key spacing
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Spacing: " + keystrokesFeature.getKeySpacing()), 
            button -> {
                int newSpacing = keystrokesFeature.getKeySpacing() + 1;
                if (newSpacing > 10) newSpacing = 0;
                keystrokesFeature.setKeySpacing(newSpacing);
                button.setMessage(Text.literal("Spacing: " + keystrokesFeature.getKeySpacing()));
            }
        ).dimensions(centerX - 100, startY + spacing * 2, 200, buttonHeight).build());
        
        // Show key labels
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Show Labels: " + (keystrokesFeature.isShowKeyLabels() ? "ON" : "OFF")), 
            button -> {
                keystrokesFeature.setShowKeyLabels(!keystrokesFeature.isShowKeyLabels());
                button.setMessage(Text.literal("Show Labels: " + (keystrokesFeature.isShowKeyLabels() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY + spacing * 3, 200, buttonHeight).build());
        
        // Color presets for normal state        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Dark"), 
            button -> keystrokesFeature.setNormalColor(0x80000000)
        ).dimensions(centerX - 120, startY + spacing * 4, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Gray"), 
            button -> keystrokesFeature.setNormalColor(0x80555555)
        ).dimensions(centerX - 50, startY + spacing * 4, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Blue"), 
            button -> keystrokesFeature.setNormalColor(0x80000080)
        ).dimensions(centerX + 20, startY + spacing * 4, 60, buttonHeight).build());
        
        // Color presets for pressed state        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("White"), 
            button -> keystrokesFeature.setPressedColor(0x80FFFFFF)
        ).dimensions(centerX - 120, startY + spacing * 5, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Red"), 
            button -> keystrokesFeature.setPressedColor(0x80FF5555)
        ).dimensions(centerX - 50, startY + spacing * 5, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Green"), 
            button -> keystrokesFeature.setPressedColor(0x8055FF55)
        ).dimensions(centerX + 20, startY + spacing * 5, 60, buttonHeight).build());
        
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
        
        // Draw color labels that I couldn't put in init
        context.drawTextWithShadow(this.textRenderer, "Normal Colors:", this.width / 2 - 60, 165, 0xFFFFFF);
        context.drawTextWithShadow(this.textRenderer, "Pressed Colors:", this.width / 2 - 60, 190, 0xFFFFFF);
        
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

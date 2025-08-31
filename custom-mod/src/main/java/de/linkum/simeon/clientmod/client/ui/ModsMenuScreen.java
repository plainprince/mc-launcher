package de.linkum.simeon.clientmod.client.ui;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.ModFeature;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

import java.util.List;

public class ModsMenuScreen extends Screen {
    private final Screen parent;
    private final ModManager modManager;
    
    public ModsMenuScreen(Screen parent) {
        super(Text.literal("Mods"));
        this.parent = parent;
        this.modManager = ModManager.getInstance();
    }

    @Override
    protected void init() {
        super.init();
        
        int centerX = this.width / 2;
        int startY = 50;
        int buttonHeight = 20;
        int buttonSpacing = 25;
        
        List<ModFeature> features = modManager.getAllFeatures();
        
        for (int i = 0; i < features.size(); i++) {
            ModFeature feature = features.get(i);
            
            // Toggle button
            this.addDrawableChild(ButtonWidget.builder(
                Text.literal(feature.getName() + ": " + (feature.isEnabled() ? "ON" : "OFF")), 
                button -> {
                    feature.toggle();
                    button.setMessage(Text.literal(feature.getName() + ": " + (feature.isEnabled() ? "ON" : "OFF")));
                }
            ).dimensions(centerX - 120, startY + i * buttonSpacing, 160, buttonHeight).build());
            
            // Config button (for features that have configuration)
            if (feature.getName().equals("CPS Counter")) {
                this.addDrawableChild(ButtonWidget.builder(
                    Text.literal("Config"), 
                    button -> this.client.setScreen(new CPSConfigScreen(this))
                ).dimensions(centerX + 50, startY + i * buttonSpacing, 60, buttonHeight).build());
            } else if (feature.getName().equals("Keystrokes")) {
                this.addDrawableChild(ButtonWidget.builder(
                    Text.literal("Config"), 
                    button -> this.client.setScreen(new KeystrokesConfigScreen(this))
                ).dimensions(centerX + 50, startY + i * buttonSpacing, 60, buttonHeight).build());
            }
        }
        
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

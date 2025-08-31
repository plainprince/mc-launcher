package de.linkum.simeon.clientmod.client.ui;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.features.CPSCounterFeature;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.gui.widget.TextFieldWidget;
import net.minecraft.text.Text;

public class CPSConfigScreen extends Screen {
    private final Screen parent;
    private final CPSCounterFeature cpsFeature;
    private TextFieldWidget displayFormatField;
    private TextFieldWidget separateFormatField;
    
    public CPSConfigScreen(Screen parent) {
        super(Text.literal("CPS Counter Config"));
        this.parent = parent;
        this.cpsFeature = (CPSCounterFeature) ModManager.getInstance().getFeature("CPS Counter");
    }

    @Override
    protected void init() {
        super.init();
        
        if (cpsFeature == null) return;
        
        int centerX = this.width / 2;
        int startY = 50;
        int fieldWidth = 200;
        int buttonHeight = 20;
        int spacing = 25;
        
        // Display format field
        displayFormatField = new TextFieldWidget(this.textRenderer, centerX - fieldWidth/2, startY, fieldWidth, buttonHeight, Text.literal("Display Format"));
        displayFormatField.setMaxLength(50);
        displayFormatField.setText(cpsFeature.getDisplayFormat());
        this.addSelectableChild(displayFormatField);
        
        // Separate format field
        separateFormatField = new TextFieldWidget(this.textRenderer, centerX - fieldWidth/2, startY + spacing * 2, fieldWidth, buttonHeight, Text.literal("Separate Format"));
        separateFormatField.setMaxLength(50);
        separateFormatField.setText(cpsFeature.getSeparateFormat());
        this.addSelectableChild(separateFormatField);
        
        // Show separate toggle
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Show Separate: " + (cpsFeature.isShowSeparate() ? "ON" : "OFF")), 
            button -> {
                cpsFeature.setShowSeparate(!cpsFeature.isShowSeparate());
                button.setMessage(Text.literal("Show Separate: " + (cpsFeature.isShowSeparate() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY + spacing * 3, 200, buttonHeight).build());
        
        // Shadow toggle
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Shadow: " + (cpsFeature.hasShadow() ? "ON" : "OFF")), 
            button -> {
                cpsFeature.setShadow(!cpsFeature.hasShadow());
                button.setMessage(Text.literal("Shadow: " + (cpsFeature.hasShadow() ? "ON" : "OFF")));
            }
        ).dimensions(centerX - 100, startY + spacing * 4, 200, buttonHeight).build());
        
        // Color preset buttons
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("White"), 
            button -> cpsFeature.setColor(0xFFFFFF)
        ).dimensions(centerX - 120, startY + spacing * 5, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Red"), 
            button -> cpsFeature.setColor(0xFF5555)
        ).dimensions(centerX - 50, startY + spacing * 5, 60, buttonHeight).build());
        
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Green"), 
            button -> cpsFeature.setColor(0x55FF55)
        ).dimensions(centerX + 20, startY + spacing * 5, 60, buttonHeight).build());
        
        // Save button
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("Save"), 
            button -> {
                cpsFeature.setDisplayFormat(displayFormatField.getText());
                cpsFeature.setSeparateFormat(separateFormatField.getText());
                this.close();
            }
        ).dimensions(centerX - 50, startY + spacing * 7, 100, buttonHeight).build());
        
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
        
        // Draw labels
        context.drawTextWithShadow(this.textRenderer, "Combined Format:", this.width / 2 - 100, 40, 0xFFFFFF);
        context.drawTextWithShadow(this.textRenderer, "Separate Format:", this.width / 2 - 100, 90, 0xFFFFFF);
        
        // Draw format help
        context.drawTextWithShadow(this.textRenderer, "Use: $c (combined), $l (left), $r (right)", this.width / 2 - 120, 140, 0xAAAAAA);
        
        // Draw current preview
        if (cpsFeature != null) {
            String preview = "Preview: " + cpsFeature.getDisplayText();
            context.drawTextWithShadow(this.textRenderer, preview, this.width / 2 - 80, 160, cpsFeature.getColor());
        }
        
        displayFormatField.render(context, mouseX, mouseY, delta);
        separateFormatField.render(context, mouseX, mouseY, delta);
        
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

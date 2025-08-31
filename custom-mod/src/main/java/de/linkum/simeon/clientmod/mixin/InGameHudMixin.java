package de.linkum.simeon.clientmod.mixin;

import de.linkum.simeon.clientmod.client.ui.elements.UIElementManager;
import de.linkum.simeon.clientmod.client.ui.elements.DraggableUIElement;
import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.hud.InGameHud;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(InGameHud.class)
public class InGameHudMixin {
    
    @Inject(method = "render", at = @At("TAIL"))
    private void onRender(DrawContext context, float tickDelta, CallbackInfo ci) {
        // Render all UI elements
        UIElementManager uiManager = UIElementManager.getInstance();
        for (DraggableUIElement element : uiManager.getAllElements()) {
            if (element.isVisible()) {
                element.render(context, 0, 0, tickDelta);
            }
        }
    }
}

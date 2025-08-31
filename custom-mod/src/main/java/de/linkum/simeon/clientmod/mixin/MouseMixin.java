package de.linkum.simeon.clientmod.mixin;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.mods.features.ZoomFeature;
import de.linkum.simeon.clientmod.client.mods.features.CPSCounterFeature;
import net.minecraft.client.Mouse;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Mouse.class)
public class MouseMixin {
    
    @Inject(method = "onMouseScroll", at = @At("HEAD"), cancellable = true)
    private void onMouseScroll(long window, double xOffset, double yOffset, CallbackInfo ci) {
        ZoomFeature zoomFeature = (ZoomFeature) ModManager.getInstance().getFeature("Zoom");
        if (zoomFeature != null && zoomFeature.onMouseScroll(yOffset)) {
            ci.cancel();
        }
    }
    
    @Inject(method = "onMouseButton", at = @At("HEAD"))
    private void onMouseButton(long window, int button, int action, int mods, CallbackInfo ci) {
        // Track clicks for CPS counter (action 1 = press, 0 = release)
        if (action == 1) { // Button press
            CPSCounterFeature cpsFeature = (CPSCounterFeature) ModManager.getInstance().getFeature("CPS Counter");
            if (cpsFeature != null) {
                if (button == 0) { // Left click
                    cpsFeature.registerLeftClick();
                } else if (button == 1) { // Right click
                    cpsFeature.registerRightClick();
                }
            }
        }
    }
}

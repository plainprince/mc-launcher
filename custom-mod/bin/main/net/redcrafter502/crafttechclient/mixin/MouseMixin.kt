package net.redcrafter502.crafttechclient.mixin

import net.fabricmc.api.EnvType
import net.fabricmc.api.Environment
import net.minecraft.client.Mouse
import org.spongepowered.asm.mixin.Mixin

@Environment(EnvType.CLIENT)
@Mixin(Mouse::class)
class MouseMixin {
    // Mouse scroll handling disabled for now due to method signature changes in 1.21.4
    // Zoom functionality will work without scroll wheel control for now
}
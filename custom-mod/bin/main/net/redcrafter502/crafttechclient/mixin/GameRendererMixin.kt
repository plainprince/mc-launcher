package net.redcrafter502.crafttechclient.mixin

import net.fabricmc.api.EnvType
import net.fabricmc.api.Environment
import net.minecraft.client.render.GameRenderer
import org.spongepowered.asm.mixin.Mixin

@Environment(EnvType.CLIENT)
@Mixin(GameRenderer::class)
class GameRendererMixin {
    // GameRenderer mixin disabled for now due to method signature changes in 1.21.4
    // Zoom functionality temporarily disabled until correct method signatures are found
}
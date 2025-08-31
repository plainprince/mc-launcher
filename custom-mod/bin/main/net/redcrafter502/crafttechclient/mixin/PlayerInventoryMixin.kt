package net.redcrafter502.crafttechclient.mixin

import net.fabricmc.api.EnvType
import net.fabricmc.api.Environment
import net.minecraft.entity.player.PlayerInventory
import org.spongepowered.asm.mixin.Mixin

@Environment(EnvType.CLIENT)
@Mixin(PlayerInventory::class)
class PlayerInventoryMixin {
    // This mixin is currently empty - all scroll handling is done in MouseMixin
    // Future hotbar scroll prevention can be added here when the correct method name is found
}
package de.linkum.simeon.clientmod.client;

import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.util.InputUtil;

public class KeyBindings {
    public static KeyBinding modMenuKeyBinding;
    
    public static void initialize() {
        // Right Shift key for mod menu
        modMenuKeyBinding = KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.clientmod.mod_menu",
            InputUtil.Type.KEYSYM,
            InputUtil.GLFW_KEY_RIGHT_SHIFT,
            "category.clientmod.general"
        ));
    }
}

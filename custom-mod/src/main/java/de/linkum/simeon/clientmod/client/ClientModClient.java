package de.linkum.simeon.clientmod.client;

import de.linkum.simeon.clientmod.client.mods.ModManager;
import de.linkum.simeon.clientmod.client.ui.MainMenuScreen;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.Environment;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Environment(EnvType.CLIENT)
public class ClientModClient implements ClientModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger("clientmod");

    @Override
    public void onInitializeClient() {
        LOGGER.info("ClientMod client initialized");
        
        // Initialize key bindings
        KeyBindings.initialize();
        
        // Initialize mod manager
        ModManager modManager = ModManager.getInstance();
        
        // Register client tick event
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            if (client.player != null && client.world != null) {
                // Handle right shift menu
                if (KeyBindings.modMenuKeyBinding.wasPressed()) {
                    client.setScreen(new MainMenuScreen(client.currentScreen));
                }
                
                // Tick mod features
                modManager.tick();
            }
        });
    }
}

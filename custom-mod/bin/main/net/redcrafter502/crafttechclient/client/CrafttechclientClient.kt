package net.redcrafter502.crafttechclient.client

import net.fabricmc.api.EnvType
import net.fabricmc.api.ClientModInitializer
import net.fabricmc.api.Environment
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents

@Environment(EnvType.CLIENT)
class CrafttechclientClient : ClientModInitializer {

    override fun onInitializeClient() {
        // Initialize Key Bindings
        KeyBindings.initialize()
        
        // Initialize Zoom functionality
        Zoom.initialize()
        
        // Register client tick event for zoom handling
        ClientTickEvents.END_CLIENT_TICK.register { client ->
            if (client.player != null && client.world != null) {
                Zoom.onClientTick(client)
            }
        }
    }
}
package de.linkum.simeon.clientmod;

import net.fabricmc.api.ModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ClientMod implements ModInitializer {
    public static final String MOD_ID = "clientmod";
    private static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);

    @Override
    public void onInitialize() {
        LOGGER.info("ClientMod initialized");
    }
}

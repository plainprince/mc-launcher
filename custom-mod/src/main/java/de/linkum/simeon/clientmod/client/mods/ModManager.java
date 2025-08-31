package de.linkum.simeon.clientmod.client.mods;

import de.linkum.simeon.clientmod.client.mods.features.MotionBlurFeature;
import de.linkum.simeon.clientmod.client.mods.features.ZoomFeature;
import de.linkum.simeon.clientmod.client.mods.features.CPSCounterFeature;
import de.linkum.simeon.clientmod.client.mods.features.KeystrokesFeature;

import java.util.ArrayList;
import java.util.List;

public class ModManager {
    private static ModManager instance;
    private final List<ModFeature> features;
    
    private ModManager() {
        features = new ArrayList<>();
        initializeFeatures();
    }
    
    public static ModManager getInstance() {
        if (instance == null) {
            instance = new ModManager();
        }
        return instance;
    }
    
    private void initializeFeatures() {
        features.add(new MotionBlurFeature());
        features.add(new ZoomFeature());
        features.add(new CPSCounterFeature());
        features.add(new KeystrokesFeature());
    }
    
    public List<ModFeature> getAllFeatures() {
        return new ArrayList<>(features);
    }
    
    public ModFeature getFeature(String name) {
        return features.stream()
                .filter(feature -> feature.getName().equals(name))
                .findFirst()
                .orElse(null);
    }
    
    public void tick() {
        for (ModFeature feature : features) {
            if (feature.isEnabled()) {
                feature.tick();
            }
        }
    }
}

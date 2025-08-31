package de.linkum.simeon.clientmod.client.ui.elements;

import java.util.ArrayList;
import java.util.List;

public class UIElementManager {
    private static UIElementManager instance;
    private final List<DraggableUIElement> elements;
    
    private UIElementManager() {
        elements = new ArrayList<>();
        initializeDefaultElements();
    }
    
    public static UIElementManager getInstance() {
        if (instance == null) {
            instance = new UIElementManager();
        }
        return instance;
    }
    
    private void initializeDefaultElements() {
        // Add some default UI elements
        elements.add(new TextUIElement("FPS Counter", 10, 10, "FPS: 60"));
        elements.add(new TextUIElement("Coordinates", 10, 30, "XYZ: 0, 64, 0"));
        elements.add(new TextUIElement("Direction", 10, 50, "Facing: North"));
        
        // Add overlay elements
        elements.add(new CPSDisplayElement(10, 70));
        elements.add(new KeystrokesDisplayElement(10, 90));
    }
    
    public List<DraggableUIElement> getAllElements() {
        return new ArrayList<>(elements);
    }
    
    public void addElement(DraggableUIElement element) {
        elements.add(element);
    }
    
    public void removeElement(DraggableUIElement element) {
        elements.remove(element);
    }
    
    public void resetAllPositions() {
        // Reset to default positions
        for (int i = 0; i < elements.size(); i++) {
            elements.get(i).setPosition(10, 10 + i * 20);
        }
    }
}

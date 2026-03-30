# Phase 3: Panel-System Vervollständigen (Weeks 5-6)

**Goal:** Production-quality panel management

**Prerequisites:** Phase 2 complete (indicators rendering correctly)

## Week 5: Panel Interactions

### Task 5.1: Drag-to-Resize Improvements

**Objective:** Polish existing drag-resize with smooth animations and constraints.

**File: `apps/frontend/src/components/PanelContainer.astro`**

Current implementation already has basic drag-resize. Improvements:

```typescript
class PanelResizer {
    private readonly MIN_PANEL_HEIGHT = 100; // pixels
    private readonly MAX_PANEL_HEIGHT = 800; // pixels
    private resizeDebounceTimer: number | null = null;
    
    private handleSeparatorDrag(e: MouseEvent, separatorIndex: number): void {
        e.preventDefault();
        
        const startY = e.clientY;
        const panels = this.getPanelElements();
        const topPanel = panels[separatorIndex];
        const bottomPanel = panels[separatorIndex + 1];
        
        const startTopHeight = topPanel.offsetHeight;
        const startBottomHeight = bottomPanel.offsetHeight;
        
        const onMouseMove = (moveEvent: MouseEvent) => {
            const deltaY = moveEvent.clientY - startY;
            
            let newTopHeight = startTopHeight + deltaY;
            let newBottomHeight = startBottomHeight - deltaY;
            
            // Apply constraints
            newTopHeight = Math.max(this.MIN_PANEL_HEIGHT, 
                                   Math.min(this.MAX_PANEL_HEIGHT, newTopHeight));
            newBottomHeight = Math.max(this.MIN_PANEL_HEIGHT,
                                      Math.min(this.MAX_PANEL_HEIGHT, newBottomHeight));
            
            // Update heights with smooth transition
            topPanel.style.height = `${newTopHeight}px`;
            bottomPanel.style.height = `${newBottomHeight}px`;
            
            // Debounced save to IndexedDB
            this.debouncedSaveLayout();
        };
        
        const onMouseUp = () => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
            
            // Final save
            this.saveLayoutToIndexedDB();
            
            // Visual feedback
            this.showSaveIndicator();
        };
        
        document.addEventListener('mousemove', onMouseMove);
        document.addEventListener('mouseup', onMouseUp);
    }
    
    private debouncedSaveLayout(): void {
        if (this.resizeDebounceTimer) {
            clearTimeout(this.resizeDebounceTimer);
        }
        
        this.resizeDebounceTimer = setTimeout(() => {
            this.saveLayoutToIndexedDB();
        }, 500);
    }
    
    private showSaveIndicator(): void {
        // Show "Layout saved" toast
        window.dispatchEvent(new CustomEvent('showToast', {
            detail: {
                type: 'success',
                message: 'Layout saved',
                duration: 2000
            }
        }));
    }
}
```

**Add visual feedback during resize:**

```css
.panel-separator {
    height: 4px;
    background: var(--border);
    cursor: ns-resize;
    transition: background 0.2s;
}

.panel-separator:hover {
    background: var(--primary);
}

.panel-separator.dragging {
    background: var(--primary);
    box-shadow: 0 0 8px var(--primary);
}
```

**Deliverable:** Smooth panel resize with constraints and persistence

---

### Task 5.2: Panel Reordering

**Objective:** Allow drag-and-drop to reorder panels.

**File: `apps/frontend/src/components/PanelContainer.astro`**

```typescript
class PanelReorder {
    private draggedPanel: HTMLElement | null = null;
    private draggedIndex: number = -1;
    
    initReordering(): void {
        const panels = this.getPanelElements();
        
        panels.forEach((panel, index) => {
            const header = panel.querySelector('.panel-header');
            if (!header) return;
            
            header.setAttribute('draggable', 'true');
            
            header.addEventListener('dragstart', (e) => {
                this.draggedPanel = panel;
                this.draggedIndex = index;
                panel.classList.add('dragging');
                
                if (e.dataTransfer) {
                    e.dataTransfer.effectAllowed = 'move';
                }
            });
            
            header.addEventListener('dragend', () => {
                panel.classList.remove('dragging');
                this.clearPlaceholders();
            });
            
            panel.addEventListener('dragover', (e) => {
                e.preventDefault();
                if (e.dataTransfer) {
                    e.dataTransfer.dropEffect = 'move';
                }
                
                this.showDropPlaceholder(index);
            });
            
            panel.addEventListener('drop', (e) => {
                e.preventDefault();
                this.handleDrop(index);
            });
        });
    }
    
    private showDropPlaceholder(targetIndex: number): void {
        this.clearPlaceholders();
        
        const panels = this.getPanelElements();
        const targetPanel = panels[targetIndex];
        
        if (targetIndex < this.draggedIndex) {
            targetPanel.classList.add('drop-before');
        } else if (targetIndex > this.draggedIndex) {
            targetPanel.classList.add('drop-after');
        }
    }
    
    private clearPlaceholders(): void {
        const panels = this.getPanelElements();
        panels.forEach(p => {
            p.classList.remove('drop-before', 'drop-after');
        });
    }
    
    private handleDrop(targetIndex: number): void {
        if (this.draggedIndex === targetIndex) return;
        
        // Call WASM to reorder panels
        const wasm = window.getWasm?.();
        if (wasm) {
            wasm.reorder_panels(this.draggedIndex, targetIndex);
        }
        
        // Update UI
        this.reorderPanelElements(this.draggedIndex, targetIndex);
        
        // Save to IndexedDB
        this.saveLayoutToIndexedDB();
        
        this.clearPlaceholders();
    }
    
    private reorderPanelElements(fromIndex: number, toIndex: number): void {
        const container = document.getElementById('panel-container');
        if (!container) return;
        
        const panels = Array.from(container.children);
        const [movedPanel] = panels.splice(fromIndex, 1);
        panels.splice(toIndex, 0, movedPanel);
        
        // Re-append in new order
        panels.forEach(panel => container.appendChild(panel));
    }
}
```

**Add CSS for drag placeholders:**

```css
.panel.dragging {
    opacity: 0.5;
}

.panel.drop-before {
    border-top: 3px solid var(--primary);
}

.panel.drop-after {
    border-bottom: 3px solid var(--primary);
}

.panel-header[draggable="true"] {
    cursor: grab;
}

.panel-header[draggable="true"]:active {
    cursor: grabbing;
}
```

**WASM binding:**

```rust
#[wasm_bindgen]
impl WasmChart {
    pub fn reorder_panels(&mut self, from_index: usize, to_index: usize) {
        self.state.panel_manager.reorder(from_index, to_index);
    }
}
```

**Deliverable:** Drag-and-drop panel reordering

---

### Task 5.3: Panel Context Menu

**Objective:** Right-click menu for panel operations.

**File: `apps/frontend/src/components/PanelContextMenu.astro`**

```html
<div 
  x-data="{
    visible: false,
    x: 0,
    y: 0,
    panelId: null
  }"
  @panel-context-menu.window="
    visible = true;
    x = $event.detail.x;
    y = $event.detail.y;
    panelId = $event.detail.panelId;
  "
  @click.away="visible = false"
  x-show="visible"
  x-cloak
  :style="`left: ${x}px; top: ${y}px`"
  class="fixed bg-card border border-border rounded-lg shadow-xl z-50 py-1 min-w-[200px]"
>
  <button 
    @click="maximizePanel(panelId); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent flex items-center gap-2">
    <span class="text-lg">⬜</span>
    Maximize Panel
  </button>
  
  <button 
    @click="minimizePanel(panelId); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent flex items-center gap-2">
    <span class="text-lg">➖</span>
    Minimize Panel
  </button>
  
  <div class="border-t border-border my-1"></div>
  
  <button 
    @click="splitHorizontally(panelId); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent flex items-center gap-2">
    <span class="text-lg">⬌</span>
    Split Horizontally
  </button>
  
  <button 
    @click="splitVertically(panelId); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent flex items-center gap-2">
    <span class="text-lg">⬍</span>
    Split Vertically
  </button>
  
  <div class="border-t border-border my-1"></div>
  
  <button 
    @click="resetAllPanels(); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent flex items-center gap-2">
    <span class="text-lg">↻</span>
    Reset All Panels
  </button>
  
  <button 
    @click="closePanel(panelId); visible = false"
    class="w-full px-4 py-2 text-left hover:bg-accent text-destructive flex items-center gap-2">
    <span class="text-lg">✕</span>
    Close Panel
  </button>
</div>

<script>
  function maximizePanel(panelId: string) {
    const wasm = window.getWasm?.();
    if (wasm) {
      wasm.maximize_panel(panelId);
    }
  }
  
  function minimizePanel(panelId: string) {
    const wasm = window.getWasm?.();
    if (wasm) {
      wasm.minimize_panel(panelId);
    }
  }
  
  function closePanel(panelId: string) {
    const wasm = window.getWasm?.();
    if (wasm) {
      wasm.remove_panel(panelId);
    }
  }
  
  function resetAllPanels() {
    const wasm = window.getWasm?.();
    if (wasm) {
      wasm.reset_panel_layout();
    }
  }
</script>
```

**Trigger context menu from panel header:**

```html
<!-- In PanelContainer.astro, add to each panel header -->
<div 
  class="panel-header"
  @contextmenu.prevent="
    $dispatch('panel-context-menu', {
      x: $event.clientX,
      y: $event.clientY,
      panelId: panel.id
    })
  "
>
  <!-- Header content -->
</div>
```

**Deliverable:** Right-click context menu for panels

---

## Week 6: Panel Persistence

### Task 6.1: Layout Serialization

**Objective:** Complete save/load system for panel layouts.

**File: `apps/frontend/src/lib/panel-persistence.ts`**

```typescript
interface PanelLayout {
    version: number;
    timestamp: number;
    name: string;
    panels: Array<{
        id: string;
        type: 'chart' | 'indicator';
        stretchFactor: number;
        height: number;
        indicators: Array<{
            id: string;
            type: string;
            params: Record<string, any>;
            color: string;
            visible: boolean;
        }>;
        settings: {
            showGrid: boolean;
            showCrosshair: boolean;
            autoScale: boolean;
        };
    }>;
}

const DB_NAME = 'loom-layouts';
const DB_VERSION = 1;
const STORE_NAME = 'layouts';

async function openDB(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(DB_NAME, DB_VERSION);
        
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
        
        request.onupgradeneeded = (event) => {
            const db = (event.target as IDBOpenDBRequest).result;
            
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME, { keyPath: 'name' });
            }
        };
    });
}

export async function saveLayout(layout: PanelLayout): Promise<void> {
    const db = await openDB();
    
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);
        const request = store.put(layout);
        
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
    });
}

export async function loadLayout(name: string): Promise<PanelLayout | null> {
    const db = await openDB();
    
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);
        const request = store.get(name);
        
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result || null);
    });
}

export async function getAllLayouts(): Promise<PanelLayout[]> {
    const db = await openDB();
    
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readonly');
        const store = tx.objectStore(STORE_NAME);
        const request = store.getAll();
        
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
    });
}

export async function deleteLayout(name: string): Promise<void> {
    const db = await openDB();
    
    return new Promise((resolve, reject) => {
        const tx = db.transaction(STORE_NAME, 'readwrite');
        const store = tx.objectStore(STORE_NAME);
        const request = store.delete(name);
        
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve();
    });
}

export async function getCurrentLayout(): Promise<PanelLayout> {
    const wasm = window.getWasm?.();
    if (!wasm) {
        throw new Error('WASM not initialized');
    }
    
    const layoutJson = wasm.export_layout();
    return JSON.parse(layoutJson);
}

export async function applyLayout(layout: PanelLayout): Promise<void> {
    const wasm = window.getWasm?.();
    if (!wasm) {
        throw new Error('WASM not initialized');
    }
    
    wasm.import_layout(JSON.stringify(layout));
}
```

**WASM bindings:**

```rust
#[wasm_bindgen]
impl WasmChart {
    pub fn export_layout(&self) -> String {
        let layout = self.state.panel_manager.serialize();
        serde_json::to_string(&layout).unwrap()
    }
    
    pub fn import_layout(&mut self, layout_json: &str) -> Result<(), JsValue> {
        let layout: PanelLayout = serde_json::from_str(layout_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.state.panel_manager.deserialize(layout)
            .map_err(|e| JsValue::from_str(&e))
    }
}
```

**Deliverable:** Complete layout persistence system

---

### Task 6.2: Layout Presets

**Objective:** Provide built-in layout presets.

**File: `apps/frontend/src/lib/layout-presets.ts`**

```typescript
export const LAYOUT_PRESETS: Record<string, PanelLayout> = {
    default: {
        version: 1,
        timestamp: Date.now(),
        name: 'Default',
        panels: [
            {
                id: 'main-chart',
                type: 'chart',
                stretchFactor: 3.0,
                height: 600,
                indicators: [],
                settings: {
                    showGrid: true,
                    showCrosshair: true,
                    autoScale: true,
                }
            }
        ]
    },
    
    multiTimeframe: {
        version: 1,
        timestamp: Date.now(),
        name: 'Multi-Timeframe',
        panels: [
            {
                id: 'chart-1h',
                type: 'chart',
                stretchFactor: 1.0,
                height: 250,
                indicators: [],
                settings: { showGrid: true, showCrosshair: true, autoScale: true }
            },
            {
                id: 'chart-15m',
                type: 'chart',
                stretchFactor: 1.0,
                height: 250,
                indicators: [],
                settings: { showGrid: true, showCrosshair: true, autoScale: true }
            },
            {
                id: 'chart-5m',
                type: 'chart',
                stretchFactor: 1.0,
                height: 250,
                indicators: [],
                settings: { showGrid: true, showCrosshair: true, autoScale: true }
            }
        ]
    },
    
    indicatorDashboard: {
        version: 1,
        timestamp: Date.now(),
        name: 'Indicator Dashboard',
        panels: [
            {
                id: 'main-chart',
                type: 'chart',
                stretchFactor: 3.0,
                height: 450,
                indicators: [
                    { id: 'sma-20', type: 'sma', params: { period: 20 }, color: '#2196F3', visible: true },
                    { id: 'ema-50', type: 'ema', params: { period: 50 }, color: '#FF5722', visible: true }
                ],
                settings: { showGrid: true, showCrosshair: true, autoScale: true }
            },
            {
                id: 'rsi-panel',
                type: 'indicator',
                stretchFactor: 1.0,
                height: 150,
                indicators: [
                    { id: 'rsi-14', type: 'rsi', params: { period: 14 }, color: '#FF9800', visible: true }
                ],
                settings: { showGrid: true, showCrosshair: false, autoScale: true }
            },
            {
                id: 'macd-panel',
                type: 'indicator',
                stretchFactor: 1.0,
                height: 150,
                indicators: [
                    { id: 'macd', type: 'macd', params: {}, color: '#2196F3', visible: true }
                ],
                settings: { showGrid: true, showCrosshair: false, autoScale: true }
            },
            {
                id: 'volume-panel',
                type: 'indicator',
                stretchFactor: 0.8,
                height: 100,
                indicators: [
                    { id: 'volume', type: 'volume', params: {}, color: '#9C27B0', visible: true }
                ],
                settings: { showGrid: false, showCrosshair: false, autoScale: true }
            }
        ]
    },
    
    trading: {
        version: 1,
        timestamp: Date.now(),
        name: 'Trading Layout',
        panels: [
            {
                id: 'main-chart',
                type: 'chart',
                stretchFactor: 2.5,
                height: 500,
                indicators: [
                    { id: 'bb', type: 'bollinger-bands', params: {}, color: '#9C27B0', visible: true },
                    { id: 'vwap', type: 'vwap', params: {}, color: '#00BCD4', visible: true }
                ],
                settings: { showGrid: true, showCrosshair: true, autoScale: true }
            },
            {
                id: 'volume',
                type: 'indicator',
                stretchFactor: 0.8,
                height: 120,
                indicators: [
                    { id: 'volume', type: 'volume', params: {}, color: '#607D8B', visible: true }
                ],
                settings: { showGrid: false, showCrosshair: false, autoScale: true }
            }
        ]
    }
};

export async function applyPreset(presetName: string): Promise<void> {
    const preset = LAYOUT_PRESETS[presetName];
    if (!preset) {
        throw new Error(`Unknown preset: ${presetName}`);
    }
    
    await applyLayout(preset);
    
    // Also save as current layout
    await saveLayout({ ...preset, name: 'current' });
}
```

**Add preset selector to UI:**

**File: `apps/frontend/src/components/LayoutPresets.astro`**

```html
<div class="p-4">
  <h3 class="font-semibold mb-3">Layout Presets</h3>
  
  <div class="grid grid-cols-2 gap-2">
    <button 
      @click="applyPreset('default')"
      class="p-3 bg-background border border-border rounded hover:bg-accent text-left">
      <div class="font-medium">Default</div>
      <div class="text-xs text-muted-foreground">Single chart view</div>
    </button>
    
    <button 
      @click="applyPreset('multiTimeframe')"
      class="p-3 bg-background border border-border rounded hover:bg-accent text-left">
      <div class="font-medium">Multi-Timeframe</div>
      <div class="text-xs text-muted-foreground">3 charts stacked</div>
    </button>
    
    <button 
      @click="applyPreset('indicatorDashboard')"
      class="p-3 bg-background border border-border rounded hover:bg-accent text-left">
      <div class="font-medium">Indicator Dashboard</div>
      <div class="text-xs text-muted-foreground">Chart + RSI + MACD + Volume</div>
    </button>
    
    <button 
      @click="applyPreset('trading')"
      class="p-3 bg-background border border-border rounded hover:bg-accent text-left">
      <div class="font-medium">Trading</div>
      <div class="text-xs text-muted-foreground">Chart + Volume</div>
    </button>
  </div>
  
  <div class="mt-4">
    <h3 class="font-semibold mb-3">Saved Layouts</h3>
    <div id="saved-layouts-list">
      <!-- Populated dynamically -->
    </div>
    
    <button 
      @click="saveCurrentLayout()"
      class="mt-2 w-full px-4 py-2 bg-primary text-primary-foreground rounded">
      Save Current Layout
    </button>
  </div>
</div>
```

**Deliverable:** Layout presets and custom layout saving

---

### Task 6.3: Layout Migration

**Objective:** Handle layout version changes gracefully.

**File: `apps/frontend/src/lib/layout-migration.ts`**

```typescript
export function migrateLayout(layout: any): PanelLayout {
    const version = layout.version || 0;
    
    if (version === 0) {
        // Migrate from v0 to v1
        layout = migrateV0ToV1(layout);
    }
    
    // Future migrations would go here
    // if (version === 1) { layout = migrateV1ToV2(layout); }
    
    return layout as PanelLayout;
}

function migrateV0ToV1(oldLayout: any): any {
    return {
        version: 1,
        timestamp: Date.now(),
        name: oldLayout.name || 'Migrated Layout',
        panels: (oldLayout.panels || []).map((panel: any) => ({
            id: panel.id || generateId(),
            type: panel.type || 'chart',
            stretchFactor: panel.stretchFactor || 1.0,
            height: panel.height || 400,
            indicators: panel.indicators || [],
            settings: {
                showGrid: panel.showGrid !== false,
                showCrosshair: panel.showCrosshair !== false,
                autoScale: panel.autoScale !== false,
            }
        }))
    };
}

export async function loadLayoutSafe(name: string): Promise<PanelLayout | null> {
    try {
        const layout = await loadLayout(name);
        if (!layout) return null;
        
        return migrateLayout(layout);
    } catch (err) {
        console.error('Failed to load layout, using default:', err);
        return LAYOUT_PRESETS.default;
    }
}
```

**Deliverable:** Layout version migration system

---

## Phase 3 Completion Checklist

- [ ] Smooth panel resize with min/max constraints
- [ ] Debounced save during resize
- [ ] Visual feedback (separators highlight on hover/drag)
- [ ] Drag-and-drop panel reordering
- [ ] Drop placeholder visuals
- [ ] Right-click context menu on panels
- [ ] Maximize/minimize panel operations
- [ ] Split panel operations (H/V)
- [ ] Layout serialization to IndexedDB
- [ ] Layout presets (4+ presets)
- [ ] Custom layout save/load
- [ ] Layout migration system
- [ ] "Layout saved" toast notifications
- [ ] All tests passing

## Success Criteria

At the end of Phase 3:
1. Smooth, professional panel interactions
2. Layouts persist across sessions
3. Users can save custom layouts
4. 4+ built-in layout presets available
5. No data loss on layout version changes

**Time Budget:** 2 weeks  
**Risk Level:** Low (mostly UI polish)  
**Dependencies:** Phase 2 (indicators working)

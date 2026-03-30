/**
 * Layout Storage using IndexedDB
 * Task 6.1: Layout Serialization
 * Task 6.3: Layout Migration
 */

import {
  autoMigrateLayout,
  addVersionInfo,
  needsMigration,
} from "./layout-migration";

interface PanelLayout {
  panels: Array<{
    id: string;
    title: string;
    y_offset: number;
    computed_height: number;
    panel_type: any;
    config: any;
  }>;
  total_height: number;
}

interface SavedLayout {
  id: string;
  name: string;
  layout: string; // JSON serialized PanelLayout
  timestamp: number;
  isDefault?: boolean;
}

const DB_NAME = "loom-layouts";
const DB_VERSION = 1;
const STORE_NAME = "layouts";
const DEFAULT_LAYOUT_KEY = "default-layout";
const AUTOSAVE_KEY = "autosave-layout";

class LayoutStorage {
  private db: IDBDatabase | null = null;

  /**
   * Initialize IndexedDB
   */
  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        console.error("Failed to open IndexedDB:", request.error);
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log("[LayoutStorage] IndexedDB initialized");
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object store for layouts
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: "id" });
          store.createIndex("timestamp", "timestamp", { unique: false });
          store.createIndex("isDefault", "isDefault", { unique: false });
          console.log("[LayoutStorage] Object store created");
        }
      };
    });
  }

  /**
   * Save current layout to IndexedDB
   */
  async saveLayout(
    name: string,
    layoutJson: string,
    isDefault = false,
  ): Promise<string> {
    if (!this.db) {
      throw new Error("IndexedDB not initialized");
    }

    // Task 6.3: Add version info to new layouts
    const versionedLayout = addVersionInfo(layoutJson);

    const id = isDefault ? DEFAULT_LAYOUT_KEY : `layout-${Date.now()}`;
    const savedLayout: SavedLayout = {
      id,
      name,
      layout: versionedLayout,
      timestamp: Date.now(),
      isDefault,
    };

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.put(savedLayout);

      request.onsuccess = () => {
        console.log(`[LayoutStorage] Saved layout: ${name} (${id})`);
        resolve(id);
      };

      request.onerror = () => {
        console.error("Failed to save layout:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Auto-save current layout (debounced)
   */
  async autoSave(layoutJson: string): Promise<void> {
    return this.saveLayout("Auto-saved", layoutJson, false).then(() => {
      // Store the auto-save ID for quick restoration
      localStorage.setItem(AUTOSAVE_KEY, layoutJson);
    });
  }

  /**
   * Load layout by ID
   */
  async loadLayout(id: string): Promise<SavedLayout | null> {
    if (!this.db) {
      throw new Error("IndexedDB not initialized");
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(id);

      request.onsuccess = () => {
        const saved = request.result;
        if (!saved) {
          resolve(null);
          return;
        }

        // Task 6.3: Auto-migrate layout if needed
        if (needsMigration(saved.layout)) {
          console.log(`[LayoutStorage] Migrating layout: ${saved.name}`);
          const migrated = autoMigrateLayout(saved.layout);

          if (migrated) {
            // Save migrated version
            saved.layout = migrated;
            this.saveLayout(saved.name, migrated, saved.isDefault).catch(
              console.error,
            );
          } else {
            console.error(
              `[LayoutStorage] Migration failed for layout: ${saved.name}`,
            );
          }
        }

        resolve(saved);
      };

      request.onerror = () => {
        console.error("Failed to load layout:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Load default layout
   */
  async loadDefaultLayout(): Promise<SavedLayout | null> {
    return this.loadLayout(DEFAULT_LAYOUT_KEY);
  }

  /**
   * Load auto-saved layout (from localStorage for speed)
   */
  loadAutoSave(): string | null {
    const saved = localStorage.getItem(AUTOSAVE_KEY);
    if (!saved) {
      return null;
    }

    // Task 6.3: Auto-migrate if needed
    if (needsMigration(saved)) {
      console.log("[LayoutStorage] Migrating auto-saved layout");
      const migrated = autoMigrateLayout(saved);

      if (migrated) {
        localStorage.setItem(AUTOSAVE_KEY, migrated);
        return migrated;
      } else {
        console.error(
          "[LayoutStorage] Auto-save migration failed - clearing corrupted data",
        );
        // Clear corrupted auto-save so it doesn't keep failing
        localStorage.removeItem(AUTOSAVE_KEY);
        return null;
      }
    }

    return saved;
  }

  /**
   * Get all saved layouts
   */
  async getAllLayouts(): Promise<SavedLayout[]> {
    if (!this.db) {
      throw new Error("IndexedDB not initialized");
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        const layouts = request.result || [];
        // Sort by timestamp (newest first)
        layouts.sort((a, b) => b.timestamp - a.timestamp);
        resolve(layouts);
      };

      request.onerror = () => {
        console.error("Failed to get all layouts:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Delete layout by ID
   */
  async deleteLayout(id: string): Promise<void> {
    if (!this.db) {
      throw new Error("IndexedDB not initialized");
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.delete(id);

      request.onsuccess = () => {
        console.log(`[LayoutStorage] Deleted layout: ${id}`);
        resolve();
      };

      request.onerror = () => {
        console.error("Failed to delete layout:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Clear all layouts
   */
  async clearAll(): Promise<void> {
    if (!this.db) {
      throw new Error("IndexedDB not initialized");
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.clear();

      request.onsuccess = () => {
        console.log("[LayoutStorage] Cleared all layouts");
        localStorage.removeItem(AUTOSAVE_KEY);
        resolve();
      };

      request.onerror = () => {
        console.error("Failed to clear layouts:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Export layout as JSON file
   */
  exportLayout(layout: SavedLayout): void {
    const dataStr = JSON.stringify(layout, null, 2);
    const blob = new Blob([dataStr], { type: "application/json" });
    const url = URL.createObjectURL(blob);

    const link = document.createElement("a");
    link.href = url;
    link.download = `${layout.name.replace(/\s+/g, "-")}-${Date.now()}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  /**
   * Import layout from JSON file
   */
  async importLayout(file: File): Promise<SavedLayout> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = async (e) => {
        try {
          const layout = JSON.parse(e.target?.result as string) as SavedLayout;

          // Validate layout structure
          if (!layout.id || !layout.name || !layout.layout) {
            throw new Error("Invalid layout format");
          }

          // Generate new ID and timestamp
          layout.id = `layout-${Date.now()}`;
          layout.timestamp = Date.now();

          // Save imported layout
          await this.saveLayout(layout.name, layout.layout, false);

          resolve(layout);
        } catch (err) {
          reject(new Error(`Failed to parse layout file: ${err}`));
        }
      };

      reader.onerror = () => {
        reject(new Error("Failed to read layout file"));
      };

      reader.readAsText(file);
    });
  }

  /**
   * Close database connection
   */
  close(): void {
    if (this.db) {
      this.db.close();
      this.db = null;
      console.log("[LayoutStorage] IndexedDB closed");
    }
  }
}

// Singleton instance
export const layoutStorage = new LayoutStorage();

// Export types
export type { SavedLayout, PanelLayout };

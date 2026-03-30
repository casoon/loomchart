/**
 * Chart State Manager
 *
 * Handles chart state export, import, and persistence using IndexedDB.
 */

import type { RustChart } from "./rust-chart";

export interface ChartStateData {
  id: string;
  name: string;
  timestamp: number;
  symbol: string;
  timeframe: string;
  state: string; // JSON string from WASM
}

const DB_NAME = "loom-chart-states";
const DB_VERSION = 1;
const STORE_NAME = "states";
const AUTOSAVE_KEY = "__autosave__";
const AUTOSAVE_INTERVAL = 30000; // 30 seconds

export class ChartStateManager {
  private db: IDBDatabase | null = null;
  private chart: RustChart | null = null;
  private autosaveInterval: number | null = null;
  private currentSymbol: string = "";
  private currentTimeframe: string = "";

  constructor() {
    this.initDB();
  }

  /**
   * Initialize IndexedDB
   */
  private async initDB(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        console.error("[StateManager] IndexedDB error:", request.error);
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log("[StateManager] IndexedDB initialized");
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: "id" });
          store.createIndex("timestamp", "timestamp", { unique: false });
          store.createIndex("name", "name", { unique: false });
          console.log("[StateManager] Object store created");
        }
      };
    });
  }

  /**
   * Set chart reference
   */
  setChart(chart: RustChart, symbol: string, timeframe: string): void {
    this.chart = chart;
    this.currentSymbol = symbol;
    this.currentTimeframe = timeframe;
  }

  /**
   * Export current chart state
   */
  async export(): Promise<string> {
    if (!this.chart) {
      throw new Error("Chart not initialized");
    }

    try {
      // Get WASM state
      const wasmState = (this.chart as any).wasmChart?.exportState();
      if (!wasmState) {
        throw new Error("Failed to export WASM state");
      }

      // Wrap with metadata
      const stateData = {
        version: "1.0.0",
        timestamp: Date.now(),
        symbol: this.currentSymbol,
        timeframe: this.currentTimeframe,
        wasmState: JSON.parse(wasmState),
      };

      return JSON.stringify(stateData, null, 2);
    } catch (error) {
      console.error("[StateManager] Export error:", error);
      throw error;
    }
  }

  /**
   * Import chart state
   */
  async import(json: string): Promise<void> {
    if (!this.chart) {
      throw new Error("Chart not initialized");
    }

    try {
      const stateData = JSON.parse(json);

      // Validate structure
      if (!stateData.wasmState) {
        throw new Error("Invalid state data: missing wasmState");
      }

      // Import to WASM
      const wasmStateJson = JSON.stringify(stateData.wasmState);
      await (this.chart as any).wasmChart?.importState(wasmStateJson);

      console.log("[StateManager] State imported successfully");
    } catch (error) {
      console.error("[StateManager] Import error:", error);
      throw error;
    }
  }

  /**
   * Save state to IndexedDB
   */
  async save(name: string): Promise<string> {
    if (!this.db) {
      await this.initDB();
    }

    const stateJson = await this.export();
    const id = `state-${Date.now()}`;

    const stateData: ChartStateData = {
      id,
      name,
      timestamp: Date.now(),
      symbol: this.currentSymbol,
      timeframe: this.currentTimeframe,
      state: stateJson,
    };

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.put(stateData);

      request.onsuccess = () => {
        console.log("[StateManager] State saved:", name);
        resolve(id);
      };

      request.onerror = () => {
        console.error("[StateManager] Save error:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Load state from IndexedDB
   */
  async load(id: string): Promise<void> {
    if (!this.db) {
      await this.initDB();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(id);

      request.onsuccess = async () => {
        const stateData = request.result as ChartStateData;
        if (!stateData) {
          reject(new Error("State not found"));
          return;
        }

        try {
          await this.import(stateData.state);
          console.log("[StateManager] State loaded:", stateData.name);
          resolve();
        } catch (error) {
          reject(error);
        }
      };

      request.onerror = () => {
        console.error("[StateManager] Load error:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Get all saved states
   */
  async getAllStates(): Promise<ChartStateData[]> {
    if (!this.db) {
      await this.initDB();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readonly");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        const states = (request.result as ChartStateData[]).filter(
          (s) => s.id !== AUTOSAVE_KEY
        );
        states.sort((a, b) => b.timestamp - a.timestamp); // Most recent first
        resolve(states);
      };

      request.onerror = () => {
        console.error("[StateManager] GetAll error:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Delete state
   */
  async deleteState(id: string): Promise<void> {
    if (!this.db) {
      await this.initDB();
    }

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction([STORE_NAME], "readwrite");
      const store = transaction.objectStore(STORE_NAME);
      const request = store.delete(id);

      request.onsuccess = () => {
        console.log("[StateManager] State deleted:", id);
        resolve();
      };

      request.onerror = () => {
        console.error("[StateManager] Delete error:", request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Auto-save current state
   */
  private async autoSave(): Promise<void> {
    try {
      const stateJson = await this.export();

      const stateData: ChartStateData = {
        id: AUTOSAVE_KEY,
        name: "Auto-save",
        timestamp: Date.now(),
        symbol: this.currentSymbol,
        timeframe: this.currentTimeframe,
        state: stateJson,
      };

      if (this.db) {
        const transaction = this.db.transaction([STORE_NAME], "readwrite");
        const store = transaction.objectStore(STORE_NAME);
        store.put(stateData);
      }
    } catch (error) {
      console.error("[StateManager] Auto-save error:", error);
    }
  }

  /**
   * Start auto-save
   */
  startAutoSave(): void {
    if (this.autosaveInterval) {
      clearInterval(this.autosaveInterval);
    }

    this.autosaveInterval = window.setInterval(() => {
      this.autoSave();
    }, AUTOSAVE_INTERVAL);

    console.log("[StateManager] Auto-save started");
  }

  /**
   * Stop auto-save
   */
  stopAutoSave(): void {
    if (this.autosaveInterval) {
      clearInterval(this.autosaveInterval);
      this.autosaveInterval = null;
      console.log("[StateManager] Auto-save stopped");
    }
  }

  /**
   * Restore auto-saved state
   */
  async restoreAutoSave(): Promise<boolean> {
    try {
      await this.load(AUTOSAVE_KEY);
      console.log("[StateManager] Auto-save restored");
      return true;
    } catch (error) {
      console.log("[StateManager] No auto-save available");
      return false;
    }
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.stopAutoSave();
    if (this.db) {
      this.db.close();
      this.db = null;
    }
    console.log("[StateManager] Destroyed");
  }
}

// Global instance
export const stateManager = new ChartStateManager();

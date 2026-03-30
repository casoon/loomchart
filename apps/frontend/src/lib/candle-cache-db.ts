// IndexedDB wrapper for persistent candle cache
// Survives page reloads and browser restarts

interface CandleCacheEntry {
  symbol: string;
  timeframe: string;
  timestamp: number; // Last update
  candles: any[]; // Serialized candles
}

interface PanelLayoutEntry {
  id: string;
  layout: any; // Panel layout JSON
  timestamp: number;
}

interface CacheStats {
  totalEntries: number;
  totalSize: number; // Approximate bytes
  oldestEntry: number;
  newestEntry: number;
}

class CandleCacheDB {
  private dbName = "loom-trading-cache";
  private version = 2;
  private db: IDBDatabase | null = null;

  async init(): Promise<void> {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;

        // Create object store: (symbol, timeframe) -> candles
        if (!db.objectStoreNames.contains("candles")) {
          const store = db.createObjectStore("candles", {
            keyPath: ["symbol", "timeframe"],
          });
          store.createIndex("timestamp", "timestamp");
          store.createIndex("symbol", "symbol");
        }

        // Create panel layouts store
        if (!db.objectStoreNames.contains("layouts")) {
          const layoutStore = db.createObjectStore("layouts", {
            keyPath: "id",
          });
          layoutStore.createIndex("timestamp", "timestamp");
        }
      };
    });
  }

  async saveCandles(
    symbol: string,
    timeframe: string,
    candles: any[],
  ): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles"], "readwrite");
      const store = tx.objectStore("candles");

      const entry: CandleCacheEntry = {
        symbol,
        timeframe,
        timestamp: Date.now(),
        candles,
      };

      const request = store.put(entry);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async loadCandles(symbol: string, timeframe: string): Promise<any[] | null> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles"], "readonly");
      const store = tx.objectStore("candles");

      const request = store.get([symbol, timeframe]);
      request.onsuccess = () => {
        const entry = request.result as CandleCacheEntry | undefined;

        // Check if cache is stale (older than 1 hour)
        if (entry && Date.now() - entry.timestamp < 3600000) {
          resolve(entry.candles);
        } else {
          resolve(null);
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async clearOldEntries(maxAge: number = 86400000): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles"], "readwrite");
      const store = tx.objectStore("candles");
      const index = store.index("timestamp");

      const cutoff = Date.now() - maxAge;
      const range = IDBKeyRange.upperBound(cutoff);
      const request = index.openCursor(range);

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          cursor.delete();
          cursor.continue();
        } else {
          resolve();
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async clearSymbol(symbol: string): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles"], "readwrite");
      const store = tx.objectStore("candles");
      const index = store.index("symbol");

      const request = index.openCursor(IDBKeyRange.only(symbol));

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          cursor.delete();
          cursor.continue();
        } else {
          resolve();
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  async clearAll(): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles", "layouts"], "readwrite");

      const candlesStore = tx.objectStore("candles");
      const layoutsStore = tx.objectStore("layouts");

      candlesStore.clear();
      layoutsStore.clear();

      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  }

  async getCacheStats(): Promise<CacheStats> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["candles"], "readonly");
      const store = tx.objectStore("candles");

      let totalEntries = 0;
      let totalSize = 0;
      let oldestEntry = Date.now();
      let newestEntry = 0;

      const request = store.openCursor();

      request.onsuccess = (event) => {
        const cursor = (event.target as IDBRequest).result;
        if (cursor) {
          const entry = cursor.value as CandleCacheEntry;
          totalEntries++;
          totalSize += JSON.stringify(entry.candles).length;
          oldestEntry = Math.min(oldestEntry, entry.timestamp);
          newestEntry = Math.max(newestEntry, entry.timestamp);
          cursor.continue();
        } else {
          resolve({ totalEntries, totalSize, oldestEntry, newestEntry });
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  // === Panel Layout Storage ===

  async saveLayout(id: string, layout: any): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["layouts"], "readwrite");
      const store = tx.objectStore("layouts");

      const entry: PanelLayoutEntry = {
        id,
        layout,
        timestamp: Date.now(),
      };

      const request = store.put(entry);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  async loadLayout(id: string = "default"): Promise<any | null> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["layouts"], "readonly");
      const store = tx.objectStore("layouts");

      const request = store.get(id);
      request.onsuccess = () => {
        const entry = request.result as PanelLayoutEntry | undefined;
        resolve(entry ? entry.layout : null);
      };
      request.onerror = () => reject(request.error);
    });
  }

  async listLayouts(): Promise<string[]> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["layouts"], "readonly");
      const store = tx.objectStore("layouts");

      const request = store.getAllKeys();
      request.onsuccess = () => {
        resolve(request.result as string[]);
      };
      request.onerror = () => reject(request.error);
    });
  }

  async deleteLayout(id: string): Promise<void> {
    if (!this.db) throw new Error("DB not initialized");

    return new Promise((resolve, reject) => {
      const tx = this.db!.transaction(["layouts"], "readwrite");
      const store = tx.objectStore("layouts");

      const request = store.delete(id);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }
}

export const candleCacheDB = new CandleCacheDB();

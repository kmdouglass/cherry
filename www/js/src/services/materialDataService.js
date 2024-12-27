import { useState, useEffect } from 'react';

const INITIAL_DATA_URL = "/data/initial-data.json";
const SERVICE_WORKER_URL = new URL("/materialServiceWorker.js", import.meta.url);

export class MaterialDataService {
  constructor() {
    this.memoryCache = new Map();
    this.updateListeners = new Set();
    this.hasIndexedDB = this.checkIndexedDBAvailable();
  }

  checkIndexedDBAvailable() {
    // TODO Implement this
    return false;
  }

  async initStorage() {
    if (this.hasIndexedDB) {
      try {
        this.db = await openDB('MaterialsDB', 1, {
          upgrade(db) {
            if (!db.objectStoreNames.contains('materials')) {
              db.createObjectStore('materials', { keyPath: 'id' });
            }
          },
        });
      } catch (e) {
        console.warn('IndexedDB failed to initialize, falling back to memory cache');
        this.hasIndexedDB = false;
      }
    }

    // Try to register service worker, but don't depend on it
    if ('serviceWorker' in navigator) {
      try {
        const registration = await navigator.serviceWorker.register(SERVICE_WORKER_URL);
        navigator.serviceWorker.addEventListener('message', (event) => {
          if (event.data.type === 'MATERIALS_UPDATE_COMPLETE') {
            this.notifyUpdateListeners();
          }
        });
      } catch (e) {
        console.warn('Service Worker registration failed, will fetch data directly');
      }
    }
  }

  async loadMaterialData() {
    // First try to load from IndexedDB
    if (this.hasIndexedDB) {
      try {
        const cached = await this.db.getAll('materials');
        if (cached.length > 0) {
          cached.forEach(material => this.memoryCache.set(material.id, material));
          return Array.from(this.memoryCache.values());
        }
      } catch (e) {
        console.warn('Failed to load from IndexedDB, falling back to direct fetch');
      }
    }

    // If no cached data, fetch initial data
    try {
      const response = await fetch(INITIAL_DATA_URL);
      if (!response.ok) throw new Error("Could not fetch initial data from network");
      
      const initialData = await response.json();
      
      // Store in memory cache
      for (const [key, value] of Object.entries(initialData.inner)) {
        this.memoryCache.set(key, value);
      }
      
      // Try to store in IndexedDB if available
      if (this.hasIndexedDB) {
        try {
          await this.storeMaterialData(initialData);
        } catch (e) {
          console.warn('Failed to store in IndexedDB');
        }
      }

      // Start loading full dataset if possible
      this.loadFullDataset();

      return initialData;
    } catch (error) {
      console.error('Error loading material data:', error);
      throw error;
    }
  }

  async loadFullDataset() {
    try {
      const response = await fetch('/materials-data.json.gz');
      if (!response.ok) throw new Error('Network response was not ok');

      const compressedData = await response.arrayBuffer();
      const decompressedData = await this.decompressData(compressedData);
      const fullData = JSON.parse(decompressedData);

      // Update memory cache
      fullData.forEach(material => this.memoryCache.set(material.id, material));

      // Try to update IndexedDB if available
      if (this.hasIndexedDB) {
        try {
          await this.storeMaterialData(fullData);
        } catch (e) {
          console.warn('Failed to store full dataset in IndexedDB');
        }
      }

      this.notifyUpdateListeners();
    } catch (error) {
      console.warn('Failed to load full dataset:', error);
    }
  }

  async getMaterial(id) {
    // Always check memory cache first
    if (this.memoryCache.has(id)) {
      return this.memoryCache.get(id);
    }

    // Try IndexedDB if available
    if (this.hasIndexedDB) {
      try {
        const material = await this.db.get('materials', id);
        if (material) {
          this.memoryCache.set(id, material);
          return material;
        }
      } catch (e) {
        console.warn('Failed to fetch from IndexedDB');
      }
    }

    return null;
  }

  onUpdate(callback) {
    this.updateListeners.add(callback);
    return () => this.updateListeners.delete(callback);
  }

  notifyUpdateListeners() {
    for (const listener of this.updateListeners) {
      listener();
    }
  }
}

// React hook
export function useMaterialService() {
  const [materialService] = useState(() => new MaterialDataService());
  const [isLoading, setIsLoading] = useState(true);
  const [isUpdated, setIsUpdated] = useState(false);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function init() {
      try {
        await materialService.initStorage();
        await materialService.loadMaterialData();
        setIsLoading(false);
      } catch (e) {
        setError(e);
        setIsLoading(false);
      }
    }
    init();

    const unsubscribe = materialService.onUpdate(() => {
      setIsUpdated(true);
    });

    return unsubscribe;
  }, []);

  return { materialService, isLoading, isUpdated, error };
}
// service-worker.js
const CACHE_NAME = 'materials-cache-v1';
const INITIAL_MATERIALS_URL = '/data/initial-materials.json';
const FULL_MATERIALS_URL = '/materials-data.json.gz';
const DB_NAME = 'MaterialsDB';
const STORE_NAME = 'materials';
const DB_VERSION = 1;

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return cache.add(INITIAL_MATERIALS_URL);
    })
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    Promise.all([
      caches.keys().then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter((name) => name !== CACHE_NAME)
            .map((name) => caches.delete(name))
        );
      }),
      loadFullMaterialsData()
    ])
  );
});

async function loadFullMaterialsData() {
  try {
    const response = await fetch(FULL_MATERIALS_URL);
    if (!response.ok) throw new Error('Network response was not ok');

    const compressedData = await response.arrayBuffer();
    const decompressedData = await decompressData(compressedData);
    const materialData = JSON.parse(decompressedData);

    const db = await openDB();
    await updateDatabase(db, materialData);

    // Notify all clients that the full data is available
    const clients = await self.clients.matchAll();
    clients.forEach(client => {
      client.postMessage({ 
        type: 'MATERIALS_UPDATE_COMPLETE',
        timestamp: new Date().toISOString()
      });
    });
  } catch (error) {
    console.error('Error loading full material data:', error);
  }
}

async function openDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);
    
    request.onupgradeneeded = (event) => {
      const db = event.target.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'id' });
      }
    };
  });
}

async function updateDatabase(db, materialData) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite');
    const store = tx.objectStore(STORE_NAME);
    
    materialData.forEach(material => {
      store.put(material);
    });
    
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

self.addEventListener('fetch', (event) => {
  if (event.request.url.endsWith(INITIAL_MATERIALS_URL)) {
    event.respondWith(
      caches.match(event.request).then((response) => {
        return response || fetch(event.request);
      })
    );
  }
});
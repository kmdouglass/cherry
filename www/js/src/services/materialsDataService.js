import { useState, useEffect } from 'react';

import {
  DATABASE_NAME,
  KEY_SEPARATOR,
  MSG_CLOSE_DB_CONNECTION,
  MSG_DB_CLOSED,
  MSG_FETCH_FULL_DATA,
  MSG_FETCH_INITIAL_DATA,
  MSG_FULL_DATA_FETCHED,
  MSG_INITIALIZED,
  OBJECT_STORE_NAME } from './materialsDataConstants';

const INITIAL_DATA_URL = `${__webpack_public_path__}data/initial-materials-data.json`;
const FULL_DATA_URL = `${__webpack_public_path__}data/full-materials-data.json`;

export class MaterialsDataService {
  #worker;

  constructor() {
    this.worker = new Worker(new URL("./materialsDataWorker.js", import.meta.url));
    this.worker.onmessage = (event) => {
      console.debug('Received message from the worker:', event.data);
    }
  }

  /*
   * Open a connection to the database and create the materials object store if it doesn't exist.
   */
  async openDBConnection() {
    let req = indexedDB.open(DATABASE_NAME, 1);
    
    return new Promise((resolve, reject) => {
      req.onsuccess = e => {
        let db = e.target.result;
        resolve(db);
      }
      req.onerror = e => {
        reject(new Error(`Failed to open the database connection: ${e.target.error?.message}`));
      }
    })
  }

  async getShelves() {
    const shelves = new Set();

    return this.openDBConnection()
      .then(db => {
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(OBJECT_STORE_NAME, "readonly");
          const store = transaction.objectStore(OBJECT_STORE_NAME);          
          const cursorRequest = store.openCursor();

          cursorRequest.onsuccess = (event) => {
              const cursor = event.target.result;
              if (cursor) {
                  // Split the cursor's key at the first colon to get the shelf name
                  shelves.add(cursor.key.split(KEY_SEPARATOR)[0]);
                  cursor.continue();
              } else {
                  // We're done - convert Set to Array and resolve
                  resolve([db, Array.from(shelves)]);
              }
          };
          
          cursorRequest.onerror = () => reject(cursorRequest.error);
        });
      })
      .then(([db, shelfNames]) => {
        db.close();
        return shelfNames;
      })
}

  async workerInitStorage() {
    this.worker.postMessage([MSG_FETCH_INITIAL_DATA, INITIAL_DATA_URL]);

    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.worker.onmessage = (event) => {
        if (event.data === MSG_INITIALIZED) {
          resolve();
        } else {
          reject(new Error(`Failed to initialize storage: ${event.data}`));
        }
      }
    });
}

  async workerFetchFullData() {
    this.worker.postMessage([MSG_FETCH_FULL_DATA, FULL_DATA_URL]);
  
    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.worker.onmessage = (event) => {
        if (event.data === MSG_FULL_DATA_FETCHED) {
          resolve();
        } else {
          console.error("Failed to fetch full data");
          reject(new Error("Failed to fetch full data"));
        }
      }
    });  
  }

  async workerCloseDBConnection() {
    this.worker.postMessage([MSG_CLOSE_DB_CONNECTION]);

    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.worker.onmessage = (event) => {
        if (event.data === MSG_DB_CLOSED) {
          resolve();
        } else {
          reject(new Error("Failed to close the database connection"));
        }
      }
    });
  }
}

// React hook
export function useMaterialsService() {
  const [materialsService] = useState(() => new MaterialsDataService());
  const [isLoadingInitialData, setIsLoadingInitialData] = useState(true);
  const [isLoadingFullData, setIsLoadingFullData] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function init() {
      try {
        // Initialize storage and fetch initial data
        await materialsService.workerInitStorage();
        setIsLoadingInitialData(false);

        // Fetch full data
        await materialsService.workerFetchFullData();
        setIsLoadingFullData(false);

        // Close the database connection
        await materialsService.workerCloseDBConnection();
      } catch (e) {
        console.error(e);
        setError(e);
        setIsLoadingInitialData(false);
        setIsLoadingFullData(false);
      }
      
    }
    init();

    return () => {
      this.worker.terminate();
    }
  }, []);

  return { materialsService, isLoadingInitialData, isLoadingFullData, error };
}

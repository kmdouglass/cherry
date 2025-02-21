import { useState, useEffect } from 'react';

import {
  INDEX_SHELF_NAME,
  INDEX_SHELF_BOOK_NAME,
  KEY_SEPARATOR,
  MSG_CLOSE_DB_CONNECTION,
  MSG_DB_CLOSED,
  MSG_FETCH_FULL_DATA,
  MSG_FETCH_INITIAL_DATA,
  MSG_FULL_DATA_FETCHED,
  MSG_INITIALIZED,
  OBJECT_STORE_NAME } from './materialsDataConstants';
import { DATABASE_NAME } from "./sharedConstants";

const INITIAL_DATA_URL = `${__webpack_public_path__}data/initial-materials-data.json`;
const FULL_DATA_URL = `${__webpack_public_path__}data/full-materials-data.json`;

export class MaterialsDataService {
  #worker;
  #selectedMaterials;
  #subscribers;

  constructor() {
    this.#worker = new Worker(new URL("./materialsDataWorker.js", import.meta.url));
    this.#worker.onmessage = (event) => {
      console.debug('Received message from the worker:', event.data);
    }

    this.#selectedMaterials = new Map();
    this.#subscribers = new Set();
  }

  // Allows React to subscribe to state changes
  subscribe(callback) {
    this.#subscribers.add(callback);
    return () => {
      this.#subscribers.delete(callback);
    };
  }

  notifySubscribers() {
    this.#subscribers.forEach(callback => callback());
  }

  get selectedMaterials() {
    return this.#selectedMaterials;
  }

  set selectedMaterials(materials) {
    this.#selectedMaterials = materials;
    this.notifySubscribers();
  }

  async addMaterialToSelectedMaterials(key) {
    const material = await this.getMaterialFromDB(key);

    if (material) {
      const newMaterials = new Map(this.selectedMaterials);
      newMaterials.set(key, material);
      this.selectedMaterials = newMaterials;
    }
  }

  clearSelectedMaterials() {
    this.selectedMaterials = new Map();
  }

  /*
   * Get a material from the database.
   */
  async getMaterialFromDB(key) {
    return this.openDBConnection()
      .then(db => {
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(OBJECT_STORE_NAME, "readonly");
          const store = transaction.objectStore(OBJECT_STORE_NAME);
          const request = store.get(key);

          request.onsuccess = () => {
            resolve(request.result);
          };

          request.onerror = () => reject(request.error);
        });
      }
    );
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

  /*
   * Returns a map of all the unique shelves in the database.
   *
   * The map key is the shelf component of the key and the value is the full name of the shelf.
   */
  async getShelves() {
    // key: shelf key, value: shelf full name
    const shelves = new Map();
    let shelfKey;

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
                  shelfKey = cursor.key.split(KEY_SEPARATOR)[0];

                  // We assume the full name is the same for all keys with the same shelf name
                  shelves.set(shelfKey, cursor.value.shelf);
                  cursor.continue();
              } else {
                  // We're done
                  resolve([db, shelves]);
              }
          };
          
          cursorRequest.onerror = () => reject(cursorRequest.error);
        });
      })
      .then(([db, shelves]) => {
        db.close();
        return shelves;
      })
  }

  /*
   * Returns a map of all the books on the given shelf.
   *
   * Parameters:
   *  shelf: string, the shelf full name.
   */
  async getBooksOnShelf(shelf) {
    const books = new Map();
    let bookKey;

    return this.openDBConnection()
      .then(db => {
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(OBJECT_STORE_NAME, "readonly");
          const store = transaction.objectStore(OBJECT_STORE_NAME);
          const index = store.index(INDEX_SHELF_NAME);
          const keyRange = IDBKeyRange.only(shelf);
          const cursorRequest = index.openCursor(keyRange);

          cursorRequest.onsuccess = (event) => {
            const cursor = event.target.result;
            if (cursor) {
              bookKey = cursor.primaryKey.split(KEY_SEPARATOR)[1];
              books.set(bookKey, cursor.value.book);
              cursor.continue();
            } else {
              resolve([db, books]);
            }
          };

          cursorRequest.onerror = () => reject(request.error);
        });
      })
      .then(([db, books]) => {
        db.close();
        return books;
      })
  }

  /*
   * Returns a map of all the pages in the given book and shelf.
   *
   * Parameters:
   *     book: string, the book full name.
   */
  async getPagesInBookOnShelf(book, shelf) {
    const pages = new Map();
    let pageKey;

    return this.openDBConnection()
      .then(db => {
        return new Promise((resolve, reject) => {
          const transaction = db.transaction(OBJECT_STORE_NAME, "readonly");
          const store = transaction.objectStore(OBJECT_STORE_NAME);
          const index = store.index(INDEX_SHELF_BOOK_NAME);
          const keyRange = IDBKeyRange.only([shelf, book]);
          const cursorRequest = index.openCursor(keyRange);

          cursorRequest.onsuccess = (event) => {
            const cursor = event.target.result;
            if (cursor) {
              pageKey = cursor.primaryKey.split(KEY_SEPARATOR)[2];
              pages.set(pageKey, cursor.value.page);
              cursor.continue();
            } else {
              resolve([db, pages]);
            }
          };

          cursorRequest.onerror = () => reject(request.error);
        });
      })
      .then(([db, pages]) => {
        db.close();
        return pages;
      })
  }

  async workerInitStorage() {
    this.#worker.postMessage([MSG_FETCH_INITIAL_DATA, INITIAL_DATA_URL]);

    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.#worker.onmessage = (event) => {
        if (event.data === MSG_INITIALIZED) {
          resolve();
        } else {
          reject(new Error(`Failed to initialize storage: ${event.data}`));
        }
      }
    });
}

  async workerFetchFullData() {
    this.#worker.postMessage([MSG_FETCH_FULL_DATA, FULL_DATA_URL]);
  
    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.#worker.onmessage = (event) => {
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
    this.#worker.postMessage([MSG_CLOSE_DB_CONNECTION]);

    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.#worker.onmessage = (event) => {
        if (event.data === MSG_DB_CLOSED) {
          resolve();
        } else {
          reject(new Error("Failed to close the database connection"));
        }
      }
    });
  }

  terminateWorker() {
    this.#worker.terminate();
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
      materialsService.terminateWorker();
    }
  }, []);

  return { materialsService, isLoadingInitialData, isLoadingFullData, error };
}

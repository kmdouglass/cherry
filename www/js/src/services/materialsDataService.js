import { useState, useEffect } from 'react';

import {
  DATABASE_NAME,
  MSG_CLOSE_DB_CONNECTION,
  MSG_DB_CLOSED,
  MSG_FETCH_FULL_DATA,
  MSG_FETCH_INITIAL_DATA,
  MSG_FULL_DATA_FETCHED,
  MSG_INITIALIZED } from './materialsDataConstants';

const INITIAL_DATA_URL = `${__webpack_public_path__}data/initial-materials-data.json`;
const FULL_DATA_URL = `${__webpack_public_path__}data/full-materials-data.json`;

export class MaterialsDataService {
  #dbConnection;
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
  openDBConnection() {
    let req = indexedDB.open(DATABASE_NAME, 1);
    req.onsuccess = e => {
      this.#dbConnection = e.target.result;
    }
    req.onerror = e => {
      throw new Error(`Failed to open the database connection: ${e.target.error?.message}`);
    }
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

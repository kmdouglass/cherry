import { useState, useEffect } from 'react';

import { MSG_CLOSE_DB_CONNECTION, MSG_DB_CLOSED, MSG_FETCH_FULL_DATA, MSG_FETCH_INITIAL_DATA, MSG_FULL_DATA_FETCHED, MSG_INITIALIZED } from './materialsDataConstants';

const INITIAL_DATA_URL = `${__webpack_public_path__}data/initial-materials-data.json`;
const FULL_DATA_URL = `${__webpack_public_path__}data/full-materials-data.json`;

export class MaterialsDataService {
  constructor() {
    this.worker = new Worker(new URL("./materialsDataWorker.js", import.meta.url));

    this.worker.onmessage = (event) => {
      console.log('Received message from the worker:', event.data);
    }

  }

  async initStorage() {
    this.worker.postMessage([MSG_FETCH_INITIAL_DATA, INITIAL_DATA_URL]);

    // Wait for the worker to finish
    return new Promise((resolve, reject) => {
      this.worker.onmessage = (event) => {
        if (event.data === MSG_INITIALIZED) {
          resolve();
        } else {
          reject(new Error("Failed to initialize storage"));
        }
      }
    });
}

  async fetchFullData() {
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

  async closeDBConnection() {
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
        await materialsService.initStorage();
        setIsLoadingInitialData(false);

        // Fetch full data
        await materialsService.fetchFullData();
        setIsLoadingFullData(false);

        // Close the database connection
        await materialsService.closeDBConnection();
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

import { useState, useEffect } from 'react';

import { MSG_FETCH_INITIAL_DATA, MSG_INITIALIZED } from './materialsDataConstants';

const INITIAL_DATA_URL = `${__webpack_public_path__}data/initial-materials-data.json`;
const FULL_DATA_URL = "/data/full-materials-data.json";

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
        console.log('Received message from the worker:', event.data);
        if (event.data === MSG_INITIALIZED) {
          resolve();
        } else {
          reject(new Error("Failed to initialize storage"));
        }
      }
    });


  }
}

// React hook
export function useMaterialsService() {
  const [materialsService] = useState(() => new MaterialsDataService());
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    async function init() {
      try {
        await materialsService.initStorage();
      } catch (e) {
        setError(e);
      }
      setIsLoading(false);
    }
    init();

    return () => {
      this.worker.terminate();
    }
  }, []);

  return { materialsService, isLoading, error };
}

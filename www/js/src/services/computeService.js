import { useState, useEffect } from 'react';

import { MSG_IN_INIT } from './computeContants';

export class ComputeService {
    #worker;

    constructor() {
        this.#worker = new Worker(new URL("./computeWorker.js", import.meta.url));
        this.#worker.onmessage = (event) => {
            console.debug('Received message from the worker:', event.data);
        }
    }

    async initWorker() {
        this.#worker.postMessage([MSG_IN_INIT, null]);
    }

    test() {
        this.#worker.postMessage(["Hello from the main thread!", null]);
    }

    terminateWorker() {
        this.#worker.terminate();
    }
}

// React hook
export function useComputeService() {
    const [computeService] = useState(() => new ComputeService());
    const [isInitializing, setIsInitializing] = useState(true);

    useEffect(() => {
        async function init() {
            try{
                await computeService.initWorker();
                setIsInitializing(false);
            } catch (error) {
                console.error("Failed to initialize the worker:", error);
            }
        }
        init();

        return () => {
            console.debug("Terminating compute service worker");
            computeService.terminateWorker();
        };
    }, []);

    return { computeService, isInitializing };
}

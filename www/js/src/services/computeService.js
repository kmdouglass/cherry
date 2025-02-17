import { useState, useEffect, use } from 'react';

export class ComputeService {
    #worker;

    constructor() {
        this.#worker = new Worker(new URL("./computeWorker.js", import.meta.url));
        this.#worker.onmessage = (event) => {
            console.debug('Received message from the worker:', event.data);
        }
    }

    test() {
        this.#worker.postMessage("Hello from the main thread!");
    }

    terminateWorker() {
        this.#worker.terminate();
    }
}

// React hook
export function useComputeService() {
    const [computeService] = useState(() => new ComputeService());

    useEffect(() => {
        return () => {
            console.debug("Terminating worker");
            computeService.terminateWorker();
        };
    }, []);

    return computeService;
}

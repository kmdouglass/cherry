import { useState, useEffect } from 'react';

import { MSG_IN_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from './computeContants';

export class ComputeService {
    #worker;

    constructor() {
        this.#worker = new Worker(new URL("./computeWorker.js", import.meta.url));
        this.#worker.onmessage = (event) => {
            console.debug('Received message from the worker:', event.data);
        }
    }

    test() {
        this.#worker.postMessage(["Hello from the main thread!", null]);
    }

    async initWorker() {
        this.#worker.postMessage([MSG_IN_INIT, null]);

        // Wait for the worker to finish initializing
        return new Promise((resolve, reject) => {
            this.#worker.onmessage = (event) => {
                if (event.data === MSG_OUT_INIT) {
                    resolve();
                } else {
                    reject(new Error(`Failed to initialize compute service: ${event.data}`));
                }
            }
        });
    }

    setWorkerMessageHandler(handler) {
        this.#worker.onmessage = handler;
    }

    terminateWorker() {
        this.#worker.terminate();
    }

    compute(specs) {
        this.#worker.postMessage([MSG_IN_COMPUTE, specs]);
    }
}

// React hook
export function useComputeService() {
    const [computeService] = useState(() => new ComputeService());
    const [isComputeServiceInitializing, setIsInitializing] = useState(true);

    useEffect(() => {
        async function init() {
            try{
                await computeService.initWorker();

                // Set up the message handler for the worker to replace the one for initialization
                computeService.setWorkerMessageHandler(event => {
                    console.debug('Received message from the worker:', event.data);
                });
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

    return { computeService, isComputeServiceInitializing };
}

import { useState, useEffect } from 'react';

import { MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from './computeContants';

export class ComputeService {
    #worker;
    #results;

    constructor() {
        this.#worker = new Worker(new URL("./computeWorker.js", import.meta.url));
        this.#worker.onmessage = (event) => {
            console.debug('Received message from the worker:', event.data);
        }

        this.#results = {};
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

    getResults() {
        return this.#results;
    }

    setResults(results) {
        this.#results = results;
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

                    const msg = event.data[0];
                    const arg = event.data[1];

                    switch (msg) {
                        case MSG_OUT_COMPUTE:
                            console.debug("Received computed results from the worker:", arg);
                            break;
                        default:
                            console.error("Unknown message from the worker:", event.data);
                    }
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

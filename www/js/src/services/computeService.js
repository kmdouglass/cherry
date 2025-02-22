import { useState, useEffect } from 'react';

import { MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from './computeContants';

export class ComputeService {
    #worker;
    #results;
    #subscribers;
    #requestID;

    constructor() {
        this.#worker = new Worker(new URL("./computeWorker.js", import.meta.url));
        this.#worker.onmessage = (_) => { }

        this.#results = {};
        this.#subscribers = new Set();
        this.#requestID = 0;
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

    // Allows React to subscribe to state changes
    subscribe(callback) {
        this.#subscribers.add(callback);
        return () => {
            this.#subscribers.delete(callback);
        };
    }

    #notifySubscribers() {
        this.#subscribers.forEach(callback => callback());
    }

    compute(specs) {
        // Append the request ID to the specs
        specs.requestID = this.#requestID;
        this.#worker.postMessage([MSG_IN_COMPUTE, specs]);
        this.#requestID++;
    }

    get results() {
        return this.#results;
    }

    set results(results) {
        this.#results = results;
        this.#notifySubscribers();
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
                    const msg = event.data[0];
                    const arg = event.data[1];

                    switch (msg) {
                        case MSG_OUT_COMPUTE:
                            computeService.results = arg;
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
            computeService.terminateWorker();
        };
    }, []);

    return { computeService, isComputeServiceInitializing };
}

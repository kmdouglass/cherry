/**
 * This service is responsible for managing the compute worker.
 * @module services/computeService
 */

import { useState, useEffect } from 'react';

import { EVENT_COMPUTE_REQUESTED, EVENT_COMPUTE_FINISHED, EVENT_WORKER_BUSY, EVENT_WORKER_IDLE, MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from './computeContants';

/**
 * A message from the worker.
 * @typedef {object} MessageFromWorker
 * @property {string} msg - The message type.
 * @property {number} requestID - The request ID.
 * @property {object} data - The computed rays.
 */

/**
 * Default message handler for messages from the worker.
 * @param {ComputeService} computeService - The compute service instance.
 * @returns {function} The message handler for the worker.
 */
const DEFAULT_ON_MESSAGE = (computeService) => {
    /**
    * @param {MessageEvent} event - The message event from the worker.
    * @param {MessageFromWorker} event.data - The data sent from the worker.
    */
    return event => {
        const msg = event.data;

        switch (msg.msg) {
            case MSG_OUT_COMPUTE:
                computeService.results = msg;
                break;
            default:
                console.error("Unknown message from the worker:", event.data);
        }
    }
}

export class ComputeService {
    #requestID;
    #results;
    #subscribers;
    #worker;
    #workerIdle;


    constructor(worker ) {
        this.#results = {};
        this.#requestID = -1;
        this.#subscribers = {};
        this.#worker = worker;
        this.#workerIdle = true;
    }

    async initWorker() {
        const ret = new Promise((resolve, reject) => {
            this.#worker.onmessage = (event) => {
                if (event.data.msg === MSG_OUT_INIT) {
                    resolve();
                } else {
                    reject(new Error(`Failed to initialize compute service: ${event.data}`));
                }
            }
        });

        this.#worker.postMessage({ msg: MSG_IN_INIT });

        return ret;
    }

    setWorkerMessageHandler(handler) {
        this.#worker.onmessage = handler;
    }

    terminateWorker() {
        this.#worker.terminate();
    }

    /**
     * Allows listeners to subscribe to events from the service.
     * @param {string} event - The event to subscribe to.
     * @param {function(any)} callback - The callback to call when the event is emitted. Receives optional data.
     * @returns {function} A function to unsubscribe from the event.
     */
    subscribe(event, callback) {
        if (!this.#subscribers[event]) {
            this.#subscribers[event] = [];
        }
        this.#subscribers[event].push(callback);

        return () => {
            this.#subscribers[event] = this.#subscribers[event].filter(cb => cb !== callback);
        }
    }

    /**
     * Notifies all subscribers of an event.
     * @param {string} event - The event to notify subscribers of.
     * @param {any} [data] - Optional data to pass to the subscribers.
     */
    #notifySubscribers(event, data = undefined) {
        if (!this.#subscribers[event]) return;
        this.#subscribers[event].forEach(callback => callback(data));
    }

    /**
     * Sends a compute request to the worker.
     * @param {object} specs - The system specs to compute.
     */
    compute(specs) {
        this.#requestID++;

        if (this.#workerIdle) {
            this.#workerIdle = false;
            this.#notifySubscribers(EVENT_WORKER_BUSY);
        }
        this.#notifySubscribers(EVENT_COMPUTE_REQUESTED, { requestID: this.#requestID });
        this.#worker.postMessage({msg: MSG_IN_COMPUTE, specs, requestID: this.#requestID});
    }

    get requestID() {
        return this.#requestID;
    }

    get results() {
        return this.#results;
    }

    /**
     * @param {MessageFromWorker} msg - The message from the worker.
     */
    set results(msg) {
        this.#results = msg;
        this.#notifySubscribers(EVENT_COMPUTE_FINISHED, { requestID: msg.requestID });
        
        this.#workerIdle = msg.requestID === this.#requestID;
        if (this.#workerIdle) {
            this.#notifySubscribers(EVENT_WORKER_IDLE);
        }
    }
}

// React hook
export function useComputeService() {
    const [computeService] = useState(() => new ComputeService(new Worker(new URL("./computeWorker.js", import.meta.url))));
    const [isComputeServiceInitializing, setIsInitializing] = useState(true);

    useEffect(() => {
        async function init() {
            try{
                await computeService.initWorker();

                // Set up the message handler for the worker to replace the one for initialization
                computeService.setWorkerMessageHandler(DEFAULT_ON_MESSAGE(computeService));
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

import { MSG_IN_INIT, MSG_OUT_INIT } from "./computeContants";

import { initializeWasm } from "../wasmLoader";

let wasmModule;

onmessage = function (event) {
    console.debug("Received message from the main thread:", event.data);
    
    const message = event.data[0];
    const arg = event.data[1];

    switch (message) {
        case MSG_IN_INIT:
            console.debug("Initializing the worker");
            initializeWasm(true)
                .then((module) => {
                    wasmModule = module;
                    postMessage([MSG_OUT_INIT, null]);
                })
                .catch((error) => {
                    console.error("Failed to initialize the worker:", error);
                });

            break;
        default:
            console.error("Unknown message from the main thread:", event.data);
    }
}

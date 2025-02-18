import { MSG_IN_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from "./computeContants";

import { initializeWasm } from "../wasmLoader";

let wasmModule;

onmessage = function (event) {
    console.debug("Received message from the main thread:", event.data);
    
    const message = event.data[0];
    const arg = event.data[1];

    switch (message) {
        case MSG_IN_INIT:
            initializeWasm(true)
                .then((module) => {
                    wasmModule = module;
                    postMessage(MSG_OUT_INIT);
                })
                .catch((error) => {
                    console.error("Failed to initialize the worker:", error);
                });

            break;
        case MSG_IN_COMPUTE:
            console.debug("Computing the optical system: ", arg);
            break;
        default:
            console.error("Unknown message from the main thread:", event.data);
    }
}

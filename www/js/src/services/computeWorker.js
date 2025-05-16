import { MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from "./computeContants";

import { initializeWasm } from "../wasmLoader";

let wasmModule;
let opticalSystem;

onmessage = function (event) {
    const msg = event.data.msg;

    switch (msg) {
        case MSG_IN_INIT:
            initializeWasm(true)
                .then((module) => {
                    wasmModule = module;
                    opticalSystem = new wasmModule.OpticalSystem();

                    this.postMessage({msg: MSG_OUT_INIT});
                })
                .catch((error) => {
                    console.error("Failed to initialize the worker:", error);
                });

            break;

        case MSG_IN_COMPUTE:
            const specs = event.data.specs;
            const requestID = event.data.requestID;
            const { surfaces, gaps, fields, aperture, wavelengths, gapMode } = specs;


            let results;
            let errorMessage = null;
            try {

                opticalSystem.setSurfaces(surfaces);
                opticalSystem.setGaps(gaps, gapMode);
                opticalSystem.setFields(fields);
                opticalSystem.setAperture(aperture);
                opticalSystem.setWavelengths(wavelengths);
                opticalSystem.build();

                results = opticalSystem.trace();
            } catch (error) {
                results = {
                    "results": [],
                }
                errorMessage = error instanceof Error ? error.message : "Error creating optical system";
            }

            const message = {
                msg: MSG_OUT_COMPUTE,
                requestID,
                data: results.results,
                errorMessage,
            }

            this.postMessage(message);

            break;
        default:
            console.error("Unknown message from the main thread:", event.data);
    }
}

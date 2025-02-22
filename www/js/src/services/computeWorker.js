import { MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from "./computeContants";

import { initializeWasm } from "../wasmLoader";

let wasmModule;
let opticalSystem;

onmessage = function (event) {
    const msg = event.data[0];
    const arg = event.data[1];

    switch (msg) {
        case MSG_IN_INIT:
            initializeWasm(true)
                .then((module) => {
                    wasmModule = module;
                    opticalSystem = new wasmModule.OpticalSystem();

                    this.postMessage(MSG_OUT_INIT);
                })
                .catch((error) => {
                    console.error("Failed to initialize the worker:", error);
                });

            break;

        case MSG_IN_COMPUTE:
            const { surfaces, gaps, fields, aperture, wavelengths, gapMode, requestID } = arg;

            opticalSystem.setSurfaces(surfaces);
            opticalSystem.setGaps(gaps, gapMode);
            opticalSystem.setFields(fields);
            opticalSystem.setAperture(aperture);
            opticalSystem.setWavelengths(wavelengths);
            opticalSystem.build();

            const rays = opticalSystem.trace();

            this.postMessage([MSG_OUT_COMPUTE, rays]);

            break;
        default:
            console.error("Unknown message from the main thread:", event.data);
    }
}

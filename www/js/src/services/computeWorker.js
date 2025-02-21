import { MSG_IN_COMPUTE, MSG_OUT_COMPUTE, MSG_IN_INIT, MSG_OUT_INIT } from "./computeContants";

import { initializeWasm } from "../wasmLoader";

let wasmModule;
let opticalSystem;

onmessage = function (event) {
    console.debug("Received message from the main thread:", event.data);
    
    const message = event.data[0];
    const arg = event.data[1];

    switch (message) {
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
            console.debug("Computing full 3D ray trace: ", arg);

            const { surfaces, gaps, fields, aperture, wavelengths, gapMode } = arg;
            opticalSystem.setSurfaces(surfaces);
            opticalSystem.setGaps(gaps, gapMode);
            opticalSystem.setFields(fields);
            opticalSystem.setAperture(aperture);
            opticalSystem.setWavelengths(wavelengths);
            opticalSystem.build();

            const rays = opticalSystem.trace();
            this.postMessage([MSG_OUT_COMPUTE, rays]);
                
            console.debug("3D ray trace complete: ", rays);

            break;
        default:
            console.error("Unknown message from the main thread:", event.data);
    }
}

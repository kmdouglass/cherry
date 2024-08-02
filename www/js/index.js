import init, { OpticalSystem } from "./pkg/cherry_js.js";
import { renderSystem } from "./modules/rendering.js"
//import { surfaces, gaps, aperture, fields } from "./modules/petzval_lens.js";
import { surface_specs, gap_specs, aperture_spec, field_specs, wavelengths } from "./modules/planoconvex_lens_v2.js";

const WorkerHandle = class {
    #isBusy = false;
    #queue = [];
    constructor() {
        this.worker = new Worker(new URL("./modules/worker.js", import.meta.url));
    }

    async #initWithTimeout() {
        return new Promise((resolve, reject) => {
            setTimeout(() => {
                reject("Timeout");
            }, 1000);

            this.worker.onmessage = (event) => {
                resolve(event.data);
            };
        });
    }

    async init() {
        this.worker.postMessage("init");
        await this.#initWithTimeout();

        this.worker.onmessage = (event) => {
            console.log("Received message from worker: ", event.data);

            this.#isBusy = false;
            if (this.#queue.length > 0) {
                this.postMessage(this.#queue.shift());
            }
        };
    }

    postMessage(message) {
        // Keep one message maximum in the queue
        if (this.#queue.length > 0) {
            this.#queue.length = 0;
        }

        // Queue the message if the worker is busy
        if (this.#isBusy) {
            this.#queue.push(message);
            return;
        }

        this.#isBusy = true;
        this.worker.postMessage(message);
    }

    terminate() {
        this.worker.terminate();
    }
}

init().then(async () => {
    //let workerHandle = new WorkerHandle();
    //await workerHandle.init();

    // Fetch JSON data for the glass catalog
    //const response = await fetch("/assets/catalog-nk.json");
    //const catalog = await response.json();
    //console.log(catalog);

    let opticalSystem = new OpticalSystem();

    //Build the optical system
    opticalSystem.setSurfaces(surface_specs);
    opticalSystem.setGaps(gap_specs);
    opticalSystem.setAperture(aperture_spec);
    opticalSystem.setFields(field_specs);
    opticalSystem.setWavelengths(wavelengths);
    opticalSystem.build();

    console.log(opticalSystem);

    let descr = opticalSystem.describe();
    console.log(descr);

    // Render the system -- SVG
    renderSystem(opticalSystem);

    // Send the data to the worker
    //let message = {
    //    surfaces: surfaces,
    //    gaps: gaps,
    //    aperture: aperture,
    //    fields: fields
    //};
    //workerHandle.postMessage(message);
});

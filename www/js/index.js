import init, { WasmSystemModel } from "./pkg/cherry_js.js";
import { renderSystem } from "./modules/rendering.js"
//import { surfaces, gaps, aperture, fields } from "./modules/petzval_lens.js";
import { surfaces, gaps, aperture, fields } from "./modules/planoconvex_lens.js";

const WorkerHandle = class {
    results;
    #isBusy = false;
    #queue = [];
    constructor() {
        this.worker = new Worker(new URL("./modules/worker.js", import.meta.url));

        this.worker.onmessage = (event) => {
            this.results = event.data;

            this.#isBusy = false;
            if (this.#queue.length > 0) {
                this.postMessage(this.#queue.shift());
            }
        };
    }

    get isBusy() {
        return this.#isBusy;
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

init().then(() => {
    let workerHandle = new WorkerHandle();
    let wasmSystemModel = new WasmSystemModel();

    //Build the optical system
    wasmSystemModel.setSurfaces(surfaces);
    wasmSystemModel.setGaps(gaps);
    wasmSystemModel.setAperture(aperture);
    wasmSystemModel.setFields(fields);
    wasmSystemModel.build();

    let descr = wasmSystemModel.describe();
    console.log(descr);

    // Render the system -- SVG
    renderSystem(wasmSystemModel);

    // Perform the full ray trace
    const start = performance.now();
    const results = wasmSystemModel.trace();
    const end = performance.now();
    console.log(`Full ray trace took ${end - start} milliseconds.`);

    console.log(results);
});

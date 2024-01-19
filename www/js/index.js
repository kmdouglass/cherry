import init, { WasmSystemModel } from "./pkg/cherry_js.js";
import { renderSystem } from "./modules/rendering.js"
//import { surfaces, gaps, aperture, fields } from "./modules/petzval_lens.js";
import { surfaces, gaps, aperture, fields } from "./modules/planoconvex_lens.js";

init().then(() => {
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

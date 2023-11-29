import { WasmSystemModel } from "cherry";
import { renderSystem } from "./modules/rendering.js"
import { surfaces, gaps, aperture, fields } from "./modules/petzval_lens.js";
// import { surfaces, gaps, aperture, fields } from "./modules/planoconvex_lens.js";

let wasmSystemModel = new WasmSystemModel();

//Build the optical system
wasmSystemModel.setSurfaces(surfaces);
wasmSystemModel.setGaps(gaps);
wasmSystemModel.setApertureV2(aperture);  // TODO Change name once the old setAperture is removed
wasmSystemModel.setFields(fields);
wasmSystemModel.build();

let descr = wasmSystemModel.describe();
console.log(descr);

// Render the system -- SVG
renderSystem(wasmSystemModel);

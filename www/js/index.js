import { WasmSystemModel } from "cherry";
import { center, draw, renderSystem, resultsToRayPaths, scaleFactor, toCanvasCoordinates } from "./modules/rendering.js"
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

// Render the system -- SVG
renderSystem(wasmSystemModel);

/////////////////////////////////////////////////
// HTML5 Canvas
// Render the surfaces -- canvas
const canvas = document.getElementById("systemModelCanvas");
const ctx = canvas.getContext("2d");
canvas.width = window.innerWidth * 0.5;
canvas.height = window.innerHeight * 0.5;

let numSamplesPerSurface = 20;
let surfSamples = [];
const numSurfaces = wasmSystemModel.surfaces().length;
for (let i = 0; i < numSurfaces; i++) {
    let samples = wasmSystemModel.sampleSurfYZ(i, numSamplesPerSurface);
    surfSamples.push({"samples": samples});
}

let sf = scaleFactor(surfSamples, canvas.width, canvas.height, 0.9);
let centerSamples = center(surfSamples);  // system x, y, z coordinates
let canvasCenterCoords = [canvas.width / 2, canvas.height / 2];  // canvas x, y coordinates
let canvasSurfs = toCanvasCoordinates(surfSamples, centerSamples, canvasCenterCoords, sf);

draw(canvasSurfs, ctx, "black", 1.0);

// Trace rays through the system
let results2 = wasmSystemModel.rayTrace();
let rayPaths2 = resultsToRayPaths(results2);
let transformedRayPaths2 = toCanvasCoordinates(rayPaths2, centerSamples, canvasCenterCoords, sf);
draw(transformedRayPaths2, ctx, "red", 1.0);

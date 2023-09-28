import { WasmSystemModel } from "cherry";
import { centerOfMass, draw, resultsToRayPaths, scaleFactor, toCanvasCoordinates } from "./modules/rendering.js"
import { surfaces, gaps } from "./modules/petzval_lens.js";

let wasmSystemModel = new WasmSystemModel();

// Set the system aperture to an entrance pupil diameter to 40 mm
let aperture = {"EntrancePupilDiameter": { diam: 40 }};
wasmSystemModel.setAperture(aperture);

// Loop over surfaces and gaps and insert them into the system model
for (let i = 0; i < surfaces.length; i++) {
    let surface = surfaces[i];
    let gap = gaps[i];
    wasmSystemModel.insertSurfaceAndGap(i + 1, surface, gap);
}

// Set the object space thickness to 200 mm
wasmSystemModel.setObjectSpace(1.0, 200);

console.log("Surfaces:", wasmSystemModel.surfaces());
console.log("Gaps:", wasmSystemModel.gaps());

// Plot the surfaces
const canvas = document.getElementById("systemModelCanvas");
const ctx = canvas.getContext("2d");
canvas.width = window.innerWidth * 0.8;
canvas.height = window.innerHeight * 0.8;

let numSamplesPerSurface = 20;
let surfSamples = [];
const numSurfaces = wasmSystemModel.surfaces().length;
for (let i = 0; i < numSurfaces; i++) {
    let samples = wasmSystemModel.sampleSurfYZ(i, numSamplesPerSurface);
    surfSamples.push({"samples": samples});
}

let sf = scaleFactor(surfSamples, canvas.width, canvas.height, 0.5);
let comSamples = centerOfMass(surfSamples);  // system x, y, z coordinates
let canvasCenterCoords = [canvas.width / 2, canvas.height / 2];  // canvas x, y coordinates
let canvasSurfs = toCanvasCoordinates(surfSamples, comSamples, canvasCenterCoords, sf);

draw(canvasSurfs, ctx, "black", 1.0);

// Trace rays through the system
let results = wasmSystemModel.rayTrace();
let rayPaths = resultsToRayPaths(results);
let transformedRayPaths = toCanvasCoordinates(rayPaths, comSamples, canvasCenterCoords, sf);
draw(transformedRayPaths, ctx, "red", 1.0);

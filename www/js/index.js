import { WasmSystemModel } from "cherry";
import { centerOfMass, centerOfMassV2, draw, resultsToRayPaths, scaleFactor, toCanvasCoordinates } from "./modules/rendering.js"
import { surfaces, gaps, aperture, fields } from "./modules/petzval_lens.js";
// import { surfaces, gaps, aperture, fields } from "./modules/planoconvex_lens.js";

let wasmSystemModel = new WasmSystemModel();

//Build the optical system
wasmSystemModel.setSurfaces(surfaces);
wasmSystemModel.setGaps(gaps);
wasmSystemModel.setApertureV2(aperture);  // TODO Change name once the old setAperture is removed
wasmSystemModel.setFields(fields);
wasmSystemModel.build();

const descr = wasmSystemModel.describe();
console.log(descr);

// Render the system -- SVG
const SVG_NS = "http://www.w3.org/2000/svg";

const rendering = document.getElementById("systemRendering");

const svg = document.createElementNS(SVG_NS, "svg");
svg.setAttribute("width", window.innerWidth * 0.5);
svg.setAttribute("height", window.innerHeight * 0.5);
svg.setAttribute("fill", "none");
svg.setAttribute("stroke", "black");

rendering.appendChild(svg);

const comSamplesSVG = centerOfMassV2(descr);
console.log(comSamplesSVG);

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

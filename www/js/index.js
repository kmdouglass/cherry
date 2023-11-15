import { WasmSystemModel } from "cherry";
import { center, centerOfMass, descrToSVGCoordinates, draw, drawSVG, rayPathsToSVGCoordinates, resultsToRayPaths, resultsToRayPathsV2, scaleFactor, scaleFactorV2, toCanvasCoordinates } from "./modules/rendering.js"
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
const SVG_NS = "http://www.w3.org/2000/svg";

const rendering = document.getElementById("systemRendering");

const svg = document.createElementNS(SVG_NS, "svg");
svg.setAttribute("width", window.innerWidth * 0.5);
svg.setAttribute("height", window.innerHeight * 0.5);
svg.setAttribute("fill", "none");
svg.setAttribute("stroke", "black");

rendering.appendChild(svg);

const sfSVG = scaleFactorV2(descr, svg.getAttribute("width"), svg.getAttribute("height"), 0.5);
const centerSystem = center(descr);
const centerSVG = [svg.getAttribute("width") / 2, svg.getAttribute("height") / 2];
descr = descrToSVGCoordinates(descr, centerSystem, centerSVG, sfSVG);
console.log(descr);

drawSVG(descr.mods.svg_surface_samples, svg, "black", 1.0);

// Trace rays through the system
const results = wasmSystemModel.rayTrace();
let rayPaths = resultsToRayPathsV2(results);
let transformedRayPaths = rayPathsToSVGCoordinates(rayPaths, centerSystem, centerSVG, sfSVG);

drawSVG(transformedRayPaths, svg, "red", 1.0);

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

let sf = scaleFactor(surfSamples, canvas.width, canvas.height, 0.5);
let comSamples = centerOfMass(surfSamples);  // system x, y, z coordinates
let canvasCenterCoords = [canvas.width / 2, canvas.height / 2];  // canvas x, y coordinates
let canvasSurfs = toCanvasCoordinates(surfSamples, comSamples, canvasCenterCoords, sf);

draw(canvasSurfs, ctx, "black", 1.0);

// Trace rays through the system
let results2 = wasmSystemModel.rayTrace();
let rayPaths2 = resultsToRayPaths(results2);
let transformedRayPaths2 = toCanvasCoordinates(rayPaths2, comSamples, canvasCenterCoords, sf);
draw(transformedRayPaths2, ctx, "red", 1.0);

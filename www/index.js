import { WasmSystemModel } from "cherry";
import { centerOfMass, draw, scaleFactor, toCanvasCoordinates } from "./modules/rendering.js"

let wasmSystemModel = new WasmSystemModel();

let surface1 = {"RefractingCircularConic": {"diam": 25.0, "roc": 25.8, "k": 0.0}};
let gap1 = {"n": 1.515, "thickness": 5.3};
let surface2 = {"RefractingCircularFlat": {"diam": 25.0}};
let gap2 = {"n": 1.0, "thickness": 46.6};

wasmSystemModel.insertSurfaceAndGap(1, surface1, gap1);
wasmSystemModel.insertSurfaceAndGap(2, surface2, gap2);

console.log("Surfaces:", wasmSystemModel.surfaces());
console.log("Gaps:", wasmSystemModel.gaps());

// Plot the surfaces
const canvas = document.getElementById("systemModelCanvas");
const ctx = canvas.getContext("2d");
canvas.width = window.innerWidth * 0.8;
canvas.height = window.innerHeight * 0.8;

let numSamplesPerSurface = 20;
let surfaces = [];
const numSurfaces = wasmSystemModel.surfaces().length;
for (let i = 0; i < numSurfaces; i++) {
    let samples = wasmSystemModel.sampleSurfYZ(i, numSamplesPerSurface);
    surfaces.push({"samples": samples});
}

let sf = scaleFactor(surfaces, canvas.width, canvas.height, 0.5);
let comSamples = centerOfMass(surfaces);  // system x, y, z coordinates
let canvasCenterCoords = [canvas.width / 2, canvas.height / 2];  // canvas x, y coordinates
let canvasSurfs = toCanvasCoordinates(surfaces, comSamples, canvasCenterCoords, sf);

draw(canvasSurfs, ctx, "black", 1.0);

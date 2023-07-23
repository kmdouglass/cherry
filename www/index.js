import { SystemModel } from "cherry";

const canvas = document.getElementById("systemModelCanvas");
const ctx = canvas.getContext("2d");

/*
    * Computes the center of mass of a system of surfaces by averaging the coordinates.
    * surfaces: an array of surface objects
    * returns: com, the coordinates of the center of mass
*/
function centerOfMass(surfaces) {
    let com = [0, 0, 0];
    let nPoints = 0;
    
    for (let surface of surfaces) {
        for (let sample of surface.samples) {
            com[0] += sample[0];
            com[1] += sample[1];
            com[2] += sample[2];
            nPoints++;
        }
    }

    com[0] /= nPoints;
    com[1] /= nPoints;
    com[2] /= nPoints;

    return com;
}

/*
    * Compute the bounding box of a system of surfaces.
    * surfaces: an array of surface objects.
    * returns: [yMin, zMin, yMax, zMax]
*/
function boundingBox(surfaces) {
    let yMin = Infinity;
    let yMax = -Infinity;
    let zMin = Infinity;
    let zMax = -Infinity;

    for (let surface of surfaces) {
        for (let sample of surface.samples) {
            yMin = Math.min(yMin, sample[1]);
            yMax = Math.max(yMax, sample[1]);
            zMin = Math.min(zMin, sample[2]);
            zMax = Math.max(zMax, sample[2]);
        }
    }

    return [yMin, zMin, yMax, zMax];
}

/*
    * Determine a scaling factor to fit a system of surfaces into a canvas.
    * surfaces: an array of surfaces, each of which is an array of [r, z] points
    * canvasWidth: the width of the canvas
    * canvasHeight: the height of the canvas
    * fillFactor: the fraction of the canvas to fill in the bigger dimension
    * returns: the scaling factor
*/
function findScaleFactor(surfaces, canvasWidth, canvasHeight, fillFactor = 0.9) {
    let [yMin, zMin, yMax, zMax] = boundingBox(surfaces);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(canvasHeight / yRange, canvasWidth / zRange);
    return scaleFactor;
}

/*
    * Transforms a system of elements into the canvas coordinate system.
    * surfaces: an array of elements (surfaces or rays)
    * comSamples: the center of mass of the system in system coordinates
    * canvasCenterCoords: the center of the canvas in x, y canvas coordinates
    * scaleFactor: the factor by which to scale the surfaces
    * returns: an array of transformed elements
*/
function toCanvasCoordinates(elements, comSamples, canvasCenterCoords, scaleFactor = 6) {
    let transformedSurfaces = [];
    for (let surface of elements) {
        let transformedSamples = [];
        for (let sample of surface.samples) {
            // Transpose the y and z coordinates because the canvas y-axis points down.
            // Take the negative of the y-coordinate because it points down the screen.
            // Shift the center of mass of the samples to that of the canvas.
            transformedSamples.push([
                canvasCenterCoords[0] + scaleFactor * (sample[2] - comSamples[2]),
                canvasCenterCoords[1] - scaleFactor * (sample[1] - comSamples[1])
            ]);
        }
        transformedSurfaces.push({"samples": transformedSamples});

    }

    return transformedSurfaces;
}

/*
    * Converts rays trace results to a series of points (ray paths) to draw on the canvas.
    * rays: an array of an array of ray objects at each surface
    * returns: an array of an array of points to draw on the canvas
*/
function resultsToRayPaths(rayTraceResults) {
    let numRays = rayTraceResults[0].length;
    let rayPaths = Array.from(Array(numRays), () => {return {"samples": []};});
    for (let surface of rayTraceResults) {
        for (let ray_id = 0; ray_id < numRays; ray_id++) {
            let ray = surface[ray_id];
            rayPaths[ray_id].samples.push(ray.pos);
        }
    }

    return rayPaths;
}

function draw(elements, ctx, color, lineWidth) {
    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;
    ctx.beginPath();
    for (let element of elements) {
        ctx.moveTo(element.samples[0][0], element.samples[0][1]);
        for (let sample of element.samples) {
            ctx.lineTo(sample[0], sample[1]);
        }
    }

    ctx.stroke();
}

/***************************************************************************************************
App starts here
*/

let system = new SystemModel();

const btn = document.querySelector("button");
btn.addEventListener("click", function () {
    let results = system.rayTrace();
    let rayPaths = resultsToRayPaths(results);
    let transformedRayPaths = toCanvasCoordinates(rayPaths, comSamples, canvasCenterCoords, scaleFactor);
    draw(transformedRayPaths, ctx, "red", 1.0);
});

canvas.width = window.innerWidth * 0.8;
canvas.height = window.innerHeight * 0.8;

// Create a f = 50.1 mm planoconvex lens comprised of two surfaces, the first one being spherical.
// This corrseponds to Thorlabs part no. LA1255.
const diam0 = 25.0; // mm
const n0 = 1.515; // refractive index of glass
const roc0 = 25.8; // mm
const K0 = 0;  // spherical
const thickness0 = 5.3;  // mm
const diam1 = 25.0; // mm
const n1 = 1.0; // refractive index of air
const backFocalLength= 46.6; // mm

// Create a system with the two surfaces
system.pushSurfObjOrImgPlane(0, 25.0);
system.pushSurfRefrCircConic(10.0, diam0, n0, roc0, K0);
system.pushSurfRefrCircFlat(10.0 + thickness0, diam1, n1);
system.pushSurfObjOrImgPlane(10.0 + thickness0 + backFocalLength, 25.0);

// Plot the surfaces
let numSamplesPerSurface = 20;
let surfaces = [];
for (let i = 0; i < system.numSurfaces(); i++) {
    let samples = system.sampleSurfYZ(i, numSamplesPerSurface);
    surfaces.push({"samples": samples});
}

let scaleFactor = findScaleFactor(surfaces, canvas.width, canvas.height, 0.5);
let comSamples = centerOfMass(surfaces);  // system x, y, z coordinates
let canvasCenterCoords = [canvas.width / 2, canvas.height / 2];  // canvas x, y coordinates
let canvasSurfs = toCanvasCoordinates(surfaces, comSamples, canvasCenterCoords, scaleFactor);

draw(canvasSurfs, ctx, "black", 1.0);
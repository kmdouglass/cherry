/*
    * Computes the center of mass of a system of surfaces by averaging the coordinates.
    * surfaces: an array of surface objects
    * returns: com, the coordinates of the center of mass
*/
export function centerOfMass(surfaces) {
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
    * Computes the center of the system's bounding box.
    * descr: a description of the optical system
    * returns: com, the coordinates of the center of mass
*/
export function center(descr) {
    const samples = descr.surface_model.surface_samples;
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBoxV2(samples);

    return [
        (xMin + xMax) / 2,
        (yMin + yMax) / 2,
        (zMin + zMax) / 2,
    ];
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
    * Compute the bounding box of a system description's surface samples.
    * samples: the surface samples of a system description
    * returns: [xMin, yMin, zMin, xMax, yMax, zMax]
*/
function boundingBoxV2(samples) {
    let xMin = Infinity;
    let xMax = -Infinity;
    let yMin = Infinity;
    let yMax = -Infinity;
    let zMin = Infinity;
    let zMax = -Infinity;

    for (let surfSamples of samples.values()) {
        for (let sample of surfSamples) {
            xMin = Math.min(xMin, sample[0]);
            xMax = Math.max(xMax, sample[0]);
            yMin = Math.min(yMin, sample[1]);
            yMax = Math.max(yMax, sample[1]);
            zMin = Math.min(zMin, sample[2]);
            zMax = Math.max(zMax, sample[2]);
        }
    }

    return [xMin, yMin, zMin, xMax, yMax, zMax];
}

/*
    * Determine a scaling factor to fit a system of surfaces into a canvas.
    * surfaces: an array of surfaces, each of which is an array of [r, z] points
    * canvasWidth: the width of the canvas
    * canvasHeight: the height of the canvas
    * fillFactor: the fraction of the canvas to fill in the bigger dimension
    * returns: the scaling factor
*/
export function scaleFactor(surfaces, canvasWidth, canvasHeight, fillFactor = 0.9) {
    let [yMin, zMin, yMax, zMax] = boundingBox(surfaces);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(canvasHeight / yRange, canvasWidth / zRange);
    return scaleFactor;
}

/*
    * Determine a scaling factor to fit a system of surfaces into a rendering area.
    * descr: a description of the optical system
    * width: the width of the drawing area
    * height: the height of the canvas
    * fillFactor: the fraction of the drawing area to fill in the bigger dimension
    * returns: the scaling factor
*/
export function scaleFactorV2(descr, width, height, fillFactor = 0.9) {
    const samples = descr.surface_model.surface_samples;

    let [yMin, zMin, yMax, zMax] = boundingBoxV2(samples);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(height / yRange, width / zRange);
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
export function toCanvasCoordinates(elements, comSamples, canvasCenterCoords, scaleFactor = 6) {
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
    * Transforms surface samples from a system description into the SVG coordinate system.
    * descr: a description of the optical system
    * systemCenter: the center of the system in system coordinates
    * svgCenter: the center of the SVG in x, y SVG coordinates
    * scaleFactor: the factor by which to scale the surfaces
    * returns: the system description containing the transformed surface samples
*/
export function toSVGCoordinates(descr, systemCenter, svgCenter, scaleFactor = 6) {
    const samples = descr.surface_model.surface_samples;
    let transformedSamples = new Map();
    for (let [surfId, surfSamples] of samples.entries()) {
        let transformedSurfSamples = [];
        for (let sample of surfSamples) {
            // Transpose the y and z coordinates because the SVG y-axis points down.
            // Take the negative of the y-coordinate because it points down the screen.
            // Shift the center of mass of the samples to that of the SVG.
            transformedSurfSamples.push([
                svgCenter[0] + scaleFactor * (sample[2] - systemCenter[2]),
                svgCenter[1] - scaleFactor * (sample[1] - systemCenter[1])
            ]);
        }
        transformedSamples.set(surfId, transformedSurfSamples);
    }

    // descr.mods contains any additional modifications to the system description not returned by the WASM layer
    // Create a mods key if it doesn't exist, then add {"svg_surface_samples": transformedSamples} to it
    let mods = descr.mods || {};
    mods = {...mods, ...{"svg_surface_samples": transformedSamples}};
    descr = {...descr, ...{"mods": mods}};

    return descr;
}

/*
    * Converts rays trace results to a series of points (ray paths) to draw on the canvas.
    * rays: an array of an array of ray objects at each surface
    * returns: an array of an array of points to draw on the canvas
*/
export function resultsToRayPaths(rayTraceResults) {
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

/*
    * Draws an array of elements to the canvas.
    * elements: an array of elements (surfaces or rays)
    * ctx: the canvas context
    * color: the color to draw the elements
    * lineWidth: the width of the lines to draw
*/
export function draw(elements, ctx, color, lineWidth) {
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

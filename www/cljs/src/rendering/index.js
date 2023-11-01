/*
    * Computes the geometrical center of the surfaces' bounding box.
    * surfaces: an array of surface objects
    * returns: com, the coordinates of the center of mass
*/
export function center(surfaces) {
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBox(surfaces);

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
    let xMin = Infinity;
    let xMax = -Infinity;
    let yMin = Infinity;
    let yMax = -Infinity;
    let zMin = Infinity;
    let zMax = -Infinity;

    for (let surface of surfaces) {
        for (let sample of surface.samples) {
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
    let [_xMin, yMin, zMin, _xMax, yMax, zMax] = boundingBox(surfaces);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(canvasHeight / yRange, canvasWidth / zRange);
    return scaleFactor;
}

/*
    * Transforms a system of elements into the canvas coordinate system.
    * surfaces: an array of elements (surfaces or rays)
    * centerSamples: the center of the system in system coordinates
    * canvasCenterCoords: the center of the canvas in x, y canvas coordinates
    * scaleFactor: the factor by which to scale the surfaces
    * returns: an array of transformed elements
*/
export function toCanvasCoordinates(elements, centerSamples, canvasCenterCoords, scaleFactor = 6) {
    let transformedSurfaces = [];
    for (let surface of elements) {
        let transformedSamples = [];
        for (let sample of surface.samples) {
            // Transpose the y and z coordinates because the canvas y-axis points down.
            // Take the negative of the y-coordinate because it points down the screen.
            // Shift the center of the samples to that of the canvas.
            transformedSamples.push([
                canvasCenterCoords[0] + scaleFactor * (sample[2] - centerSamples[2]),
                canvasCenterCoords[1] - scaleFactor * (sample[1] - centerSamples[1])
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

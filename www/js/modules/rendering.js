/*
    * Renders a system of surfaces.
    * wasmSystemModel: an optical system model
    * elementId: the id of the DOM element to render to
*/
export function renderSystem(wasmSystemModel, elementId = "systemRendering") {
    const SVG_NS = "http://www.w3.org/2000/svg";

    const rendering = document.getElementById(elementId);

    const svg = document.createElementNS(SVG_NS, "svg");
    svg.setAttribute("width", window.innerWidth * 0.5);
    svg.setAttribute("height", window.innerHeight * 0.5);
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "black");

    rendering.appendChild(svg);

    // Compute quantities required for rendering
    let descr = wasmSystemModel.describe();
    const sfSVG = scaleFactorV2(descr, svg.getAttribute("width"), svg.getAttribute("height"), 0.9);
    const centerSystem = centerV2(descr);
    const centerSVG = [svg.getAttribute("width") / 2, svg.getAttribute("height") / 2];

    // Prototype rendering by commands
    const cmds = commands(descr);
    console.log(cmds);
    
    // Draw surfaces
    let surfaceSamples = new Map();
    for (let [surfId, samples] of descr.surface_model.surface_samples.entries()) {
        surfaceSamples.set(surfId, samples);
    }
    surfaceSamples = surfacesIntoLenses(surfaceSamples, descr);
    const transformedSurfaceSamples = toSVGCoordinates(surfaceSamples, centerSystem, centerSVG, sfSVG);

    drawSVG(transformedSurfaceSamples, svg, "black", 1.0);

    // Trace rays through the system and draw them
    const results = wasmSystemModel.rayTrace();
    let rayPaths = resultsToRayPathsV2(results);
    const transformedRayPaths = toSVGCoordinates(rayPaths, centerSystem, centerSVG, sfSVG);

    drawSVG(transformedRayPaths, svg, "red", 1.0);
}

/*
    * Converts a set of surface samples to a set of rendering commands.
    * samples: a map of surface samples
    * descr: a description of the optical system
    * returns: an array of commands for the renderer
*/
function commands(descr) {
    let commands = [];
    for (let [surfId, surfSamples] of descr.surface_model.surface_samples.entries()) {
        const surfType = descr.surface_model.surface_types.get(surfId);
        let paths;

        if (surfType === "Stop") {
            paths = stopPath(surfSamples, descr);

            // A command is just an object containing data for the renderer
            commands.push({
                "type": surfType,
                "paths": paths,
                "colors": ["black"],
          });
        } else if (surfType === "ObjectPlane" || surfType === "ImagePlane") {
            commands.push({
                "type": surfType,
                "paths": [surfSamples],
                "colors": ["#dcdcdc"],
            });
        } else if (surfType === "RefractingCircularConic" || surfType === "RefractingCircularFlat") {
            commands.push({
                "type": surfType,
                "paths": [surfSamples],
                "colors": ["black"],
            });
        } else {
            console.error(`Unknown surface type: ${surfType}`);
        }
    }

    return commands;
}

///////////////////////////////////////////////////////////
// SVG rendering

/*
    * Converts paths defined by surface samples to paths for lenses.
    * surfaceSamples: a map of surface samples
    * descr: a description of the optical system
    * returns: a map of surface samples and connecting surfaces to form lenses
*/ 
function surfacesIntoLenses(surfaceSamples, descr) {
    for (let component of descr.component_model.components) {
        if (component["Element"]) {
            const surfIds = component["Element"]["surf_idxs"];
            let surfSamples = [descr.surface_model.surface_samples.get(surfIds[0]), descr.surface_model.surface_samples.get(surfIds[1])];
            const surfDiams = [descr.surface_model.diameters.get(surfIds[0]), descr.surface_model.diameters.get(surfIds[1])];

            // Find which surface has the smaller diameter
            let smallerSurfIdx = 0;
            let biggerSurfIdx = 1;
            if (surfDiams[0] > surfDiams[1]) {
                smallerSurfIdx = 1;
                biggerSurfIdx = 0;
            }
            const yExtent = surfDiams[biggerSurfIdx] / 2;

            // Extend the smaller surface to the same diameter as the larger surface by adding y points
            const firstPoint = surfSamples[smallerSurfIdx][0];
            const lastPoint = surfSamples[smallerSurfIdx][surfSamples[smallerSurfIdx].length - 1];
            
            surfaceSamples.get(surfIds[smallerSurfIdx]).unshift([firstPoint[0], -yExtent, firstPoint[2]]);
            surfaceSamples.get(surfIds[smallerSurfIdx]).push([lastPoint[0], yExtent, lastPoint[2]]);

            // Connect the lens surface endpoints to form a lens
            let bottomEndpoints = [surfaceSamples.get(surfIds[0])[0], surfaceSamples.get(surfIds[1])[0]];
            let topEndpoints = [surfaceSamples.get(surfIds[0])[surfaceSamples.get(surfIds[0]).length - 1], surfaceSamples.get(surfIds[1])[surfaceSamples.get(surfIds[1]).length - 1]];

            // Note that we go from the smaller surface to the larger one
            surfaceSamples.get(surfIds[smallerSurfIdx]).unshift([bottomEndpoints[smallerSurfIdx][0], bottomEndpoints[smallerSurfIdx][1], bottomEndpoints[biggerSurfIdx][2]]);
            surfaceSamples.get(surfIds[smallerSurfIdx]).push([topEndpoints[smallerSurfIdx][0], topEndpoints[smallerSurfIdx][1], topEndpoints[biggerSurfIdx][2]]);
        }
    }
    return surfaceSamples;
}

/*
    * Creates the path for a surface of type Stop.
    * surfaceSamples: a map of surface samples for the stop surface
    * descr: a description of the optical system
    * returns: an array of paths for the stop surface
*/
function stopPath(surfaceSamples, descr) {
    const bbox = boundingBoxV2(descr.surface_model.surface_samples);
    const yMin = bbox[1];
    const yMax = bbox[4];

    const surfYMin = surfaceSamples[0][1];
    const surfYMax = surfaceSamples[surfaceSamples.length - 1][1];
    const x = surfaceSamples[0][0];
    const z = surfaceSamples[0][2];

    let paths = [
        [[x, yMin, z], [x, surfYMin, z]],
        [[x, surfYMax, z], [x, yMax, z]]
    ];

    return paths;
}

/*
    * Computes the center of the system's bounding box.
    * descr: a description of the optical system
    * returns: com, the coordinates of the center of mass
*/
function centerV2(descr) {
    const samples = descr.surface_model.surface_samples;
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBoxV2(samples);

    return [
        (xMin + xMax) / 2,
        (yMin + yMax) / 2,
        (zMin + zMax) / 2,
    ];
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
    * Determine a scaling factor to fit a system of surfaces into a rendering area.
    * descr: a description of the optical system
    * width: the width of the drawing area
    * height: the height of the canvas
    * fillFactor: the fraction of the drawing area to fill in the bigger dimension
    * returns: the scaling factor
*/
function scaleFactorV2(descr, width, height, fillFactor = 0.9) {
    const samples = descr.surface_model.surface_samples;

    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBoxV2(samples);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(height / yRange, width / zRange);
    return scaleFactor;
}

/*
    * Transforms paths of 3D points to the SVG coordinate system.
*/
function toSVGCoordinates(paths, systemCenter, svgCenter, scaleFactor = 6) {
    let transformedPaths = new Map();
    for (let [pathId, pathSamples] of paths.entries()) {
        let transformedPathSamples = [];
        for (let sample of pathSamples) {
            // Transpose the y and z coordinates because the SVG y-axis points down.
            // Take the negative of the y-coordinate because it points down the screen.
            // Shift the center of mass of the samples to that of the SVG.
            transformedPathSamples.push([
                svgCenter[0] + scaleFactor * (sample[2] - systemCenter[2]),
                svgCenter[1] - scaleFactor * (sample[1] - systemCenter[1])
            ]);
        }
        transformedPaths.set(pathId, transformedPathSamples);
    }

    return transformedPaths;
}

/*
    * Converts rays trace results to a series of points (ray paths) to draw on the SVG.
    * rays: an array of an array of ray objects at each surface
    * returns: an array of an array of points to draw on the SVG
*/
function resultsToRayPathsV2(rayTraceResults) {
    let numRays = rayTraceResults[0].length;

    // Create an empty map of ray paths
    let rayPaths = new Map();
    for (let surface of rayTraceResults) {
        for (let ray_id = 0; ray_id < numRays; ray_id++) {
            let ray = surface[ray_id];
            rayPaths.set(ray_id, rayPaths.get(ray_id) || []);
            rayPaths.get(ray_id).push(ray.pos);
        }
    }

    return rayPaths;
}

/*
    * Draws paths defined by samples to an SVG.
    * paths: a map of paths, where each path is an array of 3D coordinates
    * svg: the SVG element to draw to
    * color: the color to draw the elements
    * lineWidth: the width of the lines to draw
*/
function drawSVG(paths, svg, color, lineWidth) {
    for (let [pathId, samples] of paths.entries()) {
        let path = document.createElementNS("http://www.w3.org/2000/svg", "path");
        let d = `M ${samples[0][0]} ${samples[0][1]}`;
        for (let sample of samples) {
            d += ` L ${sample[0]} ${sample[1]}`;
        }
        path.setAttribute("d", d);
        path.setAttribute("stroke", color);
        path.setAttribute("stroke-width", lineWidth);
        path.setAttribute("fill", "none");
        svg.appendChild(path);
    }
}


///////////////////////////////////////////////
// Canvas rendering
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

export function center(samples) {
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBox(samples);

    return [
        (xMin + xMax) / 2,
        (yMin + yMax) / 2,
        (zMin + zMax) / 2,
    ];
}

/*
    * Compute the bounding box of a system of surfaces.
    * surfaces: an array of surface objects.
    * returns: [xMin, yMin, zMin, xMax, yMax, zMax]
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
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBox(surfaces);
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

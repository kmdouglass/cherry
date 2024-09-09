/*
    * Renders a system of surfaces.
    * description: an optical system description
    * elementId: the id of the DOM element to render to
*/
export function renderCutaway(description, rawRayPaths, svgElement) {
    svgElement.innerHTML = "";

    // Compute the SVG width in pixels
    const width = getSvgWidthInPixels(svgElement);

    // Compute quantities required for rendering
    const sfSVG = scaleFactor(description, width, svgElement.getAttribute("height"), 0.9);
    const centerSystem = center(description);
    const centerSVG = [width / 2, svgElement.getAttribute("height") / 2];

    const rayPaths = resultsToRayPaths(rawRayPaths);

    // Create the rendering commands
    const cmds = commands(description, rayPaths, centerSystem, centerSVG, sfSVG);
    drawCommands(cmds, svgElement);
}

function commands(descr, rayPaths, centerSystem, centerSVG, sf) {
    let commands = [];
    let paths;

    // Create paths that connect lenses (these go in the background)
    paths = surfacesIntoLenses(descr);
    paths = toSVGCoordinates(paths, centerSystem, centerSVG, sf);
    commands.push({
        "type": "Lens",
        "paths": paths,
        "color": "black",
        "stroke-width": 1.0,
        "stroke-linejoin": "bevel",
        "close-path": true,
    });

    for (let [surfId, surfSamples] of descr.cutaway_view.path_samples.entries()) {
        const surfType = descr.cutaway_view.surface_types.get(surfId);

        if (surfType === "Stop") {
            paths = stopPath(surfSamples, descr);
            paths = toSVGCoordinates(paths, centerSystem, centerSVG, sf);

            // A command is just an object containing data for the renderer
            commands.push({
                "type": surfType,
                "paths": paths,
                "color": "black",
                "stroke-width": 1.0,
          });
        } else if (surfType === "Object" || surfType === "Image" || surfType === "Probe") {
            paths = toSVGCoordinates([surfSamples], centerSystem, centerSVG, sf);
            commands.push({
                "type": surfType,
                "paths": paths,
                "color": "#999999",
                "stroke-width": 1.0,
            });
        } else if (surfType === "Conic") {
            // These are the surface clear apertures
            paths = toSVGCoordinates([surfSamples], centerSystem, centerSVG, sf);
            commands.push({
                "type": surfType,
                "paths": paths,
                "color": "black",
                "stroke-width": 1.0,
        });
        } else {
            console.error(`Unknown surface type: ${surfType}`);
        }
    }

    // Create ray paths
    // Loop over rayPaths map and convert the underlying array of paths to SVG coordinates
    for (let [submodel, submodelRayPaths] of rayPaths) {
        paths = toSVGCoordinates(submodelRayPaths, centerSystem, centerSVG, sf);
        commands.push({
            "type": "Rays",
            "paths": paths,
            "color": "red",
            "stroke-width": 0.5,
        });
    }
    return commands;
}

function drawCommands(commands, svgElement) {
    for (let command of commands) {
        command.paths.forEach(function(path, i) {
            if (path.length == 0) {
                // Nothing to draw
            } else {

                let pathElement = document.createElementNS("http://www.w3.org/2000/svg", "path");
                let d = `M ${path[0][0]} ${path[0][1]}`;
                for (let point of path) {
                    d += ` L ${point[0]} ${point[1]}`;
                }
                if (command["close-path"]) {
                    d += " Z";
                }
                pathElement.setAttribute("d", d);
                pathElement.setAttribute("stroke", command.color);
                pathElement.setAttribute("stroke-width", command["stroke-width"] || 1.0);
                pathElement.setAttribute("stroke-linejoin", command["stroke-linejoin"] || "miter");
                pathElement.setAttribute("fill", "none");
                svgElement.appendChild(pathElement);
            }
        });
    }
}

function surfacesIntoLenses(descr) {
    const surfaceSamples = descr.cutaway_view.path_samples;

    let paths = new Array();
    let topPath;
    let bottomPath;
    for (let component of descr.components_view) {
        if (component["Element"]) {
            let path = new Array();
            topPath = new Array();
            bottomPath = new Array();

            const surfIds = component["Element"]["surf_idxs"];
            const surfSamples = [descr.cutaway_view.path_samples.get(surfIds[0]), descr.cutaway_view.path_samples.get(surfIds[1])];
            const surfSemiDiams = [descr.cutaway_view.semi_diameters.get(surfIds[0]), descr.cutaway_view.semi_diameters.get(surfIds[1])];

            // Find which surface has the smaller diameter
            let smallerSurfIdx = 0;
            let biggerSurfIdx = 1;
            if (surfSemiDiams[0] > surfSemiDiams[1]) {
                smallerSurfIdx = 1;
                biggerSurfIdx = 0;
            }
            const yExtent = surfSemiDiams[biggerSurfIdx];

            // Extend the smaller surface to the same diameter as the larger surface by adding y points
            const firstPoint = surfSamples[smallerSurfIdx][0];
            const lastPoint = surfSamples[smallerSurfIdx][surfSamples[smallerSurfIdx].length - 1];

            topPath.push([lastPoint[0], yExtent, lastPoint[2]]);
            bottomPath.push([firstPoint[0], -yExtent, firstPoint[2]]);

            // Connect the lens surface endpoints to form a lens
            let bottomEndpoints = [surfaceSamples.get(surfIds[0])[0], surfaceSamples.get(surfIds[1])[0]];
            let topEndpoints = [surfaceSamples.get(surfIds[0])[surfaceSamples.get(surfIds[0]).length - 1], surfaceSamples.get(surfIds[1])[surfaceSamples.get(surfIds[1]).length - 1]];

            // Note that we go from the smaller surface to the larger one
            topPath.push([topEndpoints[smallerSurfIdx][0], yExtent, topEndpoints[biggerSurfIdx][2]]);
            bottomPath.push([bottomEndpoints[smallerSurfIdx][0], -yExtent, bottomEndpoints[biggerSurfIdx][2]]);

            // Build the path from the bottom of the smaller surface
            path.push(bottomPath[0]);
            path = path.concat(surfSamples[smallerSurfIdx]);
            path = path.concat(topPath);
            path = path.concat(surfSamples[biggerSurfIdx].toReversed());
            path.push(bottomPath[1]);
            path.push(bottomPath[0]);
            
            paths.push(path);
        }
    }

    return paths
}            

/*
    * Creates the path for a surface of type Stop.
    * surfaceSamples: a map of surface samples for the stop surface
    * descr: a description of the optical system
    * returns: an array of paths for the stop surface
*/
function stopPath(surfaceSamples, descr) {
    const bbox = boundingBox(descr.cutaway_view.path_samples);
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
function center(descr) {
    const samples = descr.cutaway_view.path_samples;
    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBox(samples);

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
function boundingBox(samples) {
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
function scaleFactor(descr, width, height, fillFactor = 0.9) {
    const samples = descr.cutaway_view.path_samples;

    let [xMin, yMin, zMin, xMax, yMax, zMax] = boundingBox(samples);
    let yRange = yMax - yMin;
    let zRange = zMax - zMin;
    let scaleFactor = fillFactor * Math.min(height / yRange, width / zRange);
    
    return scaleFactor;
}

function toSVGCoordinates(paths, systemCenter, svgCenter, scaleFactor = 6) {
    let transformedPaths = new Array();
    for (let [_pathId, pathSamples] of paths.entries()) {
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
        transformedPaths.push(transformedPathSamples);
    }

    return transformedPaths;
}

export function resultsToRayPaths(rayTraceResults) {
    let rayPathsBySubmodel = new Map();

    // loop over each key value pair of the Map rayTraceResults
    for (let [submodel, rays] of rayTraceResults) {
        let rayPaths = submodelResultsToRayPaths(rays);

        rayPathsBySubmodel.set(submodel, rayPaths);
    }

    return rayPathsBySubmodel;
}

/*
    * Converts rays trace results to a series of points (ray paths) to draw on the SVG.
    * rays: an array of an array of ray objects at each surface
    * returns: an array of an array of points to draw on the SVG
*/
function submodelResultsToRayPaths(rayTraceResults) {
    let numRays = rayTraceResults[0].length;

    let rayPaths = new Map();
    for (let surface of rayTraceResults) {
        for (let ray_id = 0; ray_id < numRays; ray_id++) {
            if (ray_id < surface.length) {
                let ray = surface[ray_id];

                // check if ray is null or undefined
                if (ray == null) {
                    continue;
                }

                rayPaths.set(ray_id, rayPaths.get(ray_id) || []);
                rayPaths.get(ray_id).push(ray.pos);
            }
        }
    }

    return rayPaths;
}

function getSvgWidthInPixels(svgElement) {
    let width = svgElement.getAttribute("width");

    if (width.endsWith("%")) {
        // Remove the '%' character and convert to a float
        const percentage = parseFloat(width) / 100;

        // Get the parent element's width in pixels
        const parentWidth = svgElement.parentElement.clientWidth;

        // Calculate the width in pixels
        width = percentage * parentWidth;
    } else {
        // Convert the width to a float if it's already in pixels
        width = parseFloat(width);
    }

    return width;
}

/**
 * Renders a cutaway view of an optical system.
 * @module cutaway
 */

/**
 * Whether to show terminated paths past their cutoff points. Used for debugging.
 * @type {boolean}
 * @constant
 */
const HIDE_TERMINATED_RAYS = true;

/**
 * An array of coordinate component values, e.g. all x's, all y's, or all z's.
 * @typedef {Array<number>} Coordinates
 */

/**
 * An ordered list of coordinates to be drawn.
 * @typedef Path
 * @type {object}
 * @property {Array<Coordinates>} x - The x-coordinates of the path.
 * @property {Array<Coordinates>} y - The y-coordinates of the path.
 * @property {Array<Coordinates>} z - The z-coordinates of the path.
 * @property {number} [cutoff] - The index at which to stop drawing the path. Useful for rays that terminate early.
 */

/**
 * A collection of individual paths.
 * @typedef {Array<Path>} Paths
 */

/**
 * Renders a cutaway view of an optical system.
 * @param { import("./rayTracerTypes/rays").Description } description 
 * @param { import("./rayTracerTypes/rays").TraceResultsCollection } traceResultsCollection 
 * @param {object} svgElement 
 */
export function renderCutaway(description, traceResultsCollection, svgElement) {
    svgElement.innerHTML = "";

    // Compute the SVG width in pixels
    const width = getSvgWidthInPixels(svgElement);

    // Compute quantities required for rendering
    const sfSVG = scaleFactor(description, width, svgElement.getAttribute("height"), 0.9);
    const centerSystem = center(description);
    const centerSVG = [width / 2, svgElement.getAttribute("height") / 2];

    const rayPaths = resultsToRayPaths(traceResultsCollection);

    // Create the rendering commands
    const cmds = commands(description, rayPaths, centerSystem, centerSVG, sfSVG);
    drawCommands(cmds, svgElement);
}

function commands(descr, rayPaths, centerSystem, centerSVG, sf) {
    let commands = [];
    let path;
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

            commands.push({
                "type": surfType,
                "paths": paths,
                "color": "black",
                "stroke-width": 1.0,
          });
        } else if (surfType === "Object" || surfType === "Image" || surfType === "Probe") {
            path = coordsToPath(surfSamples);
            paths = toSVGCoordinates([path], centerSystem, centerSVG, sf);
            commands.push({
                "type": surfType,
                "paths": paths,
                "color": "#999999",
                "stroke-width": 1.0,
            });
        } else if (surfType === "Conic" && isLensSurface(descr, surfId)) {
            continue;
        } else if (surfType === "Conic") {
            // These are the surface clear apertures and are needed for unpaired surfaces
            path = coordsToPath(surfSamples);
            paths = toSVGCoordinates([path], centerSystem, centerSVG, sf);
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

    //Create ray paths
    // Loop over rayPaths map and convert the underlying array of paths to SVG coordinates
    paths = toSVGCoordinates(rayPaths, centerSystem, centerSVG, sf);
    commands.push({
        "type": "Rays",
        "paths": paths,
        "color": "red",
        "stroke-width": 0.5,
    });
    return commands;
}

function drawCommands(commands, svgElement) {
    for (let command of commands) {
        command.paths.forEach(function(path, i) {
            if (path.x.length == 0) {
                // Nothing to draw
            } else {
                let pathElement = document.createElementNS("http://www.w3.org/2000/svg", "path");
                let d = `M ${path.x[0]} ${path.y[0]}`;
                
                // Draw the points of the path
                for (let i = 1; i < path.x.length; i++) {
                    if (HIDE_TERMINATED_RAYS && path.cutoff !== undefined && i > path.cutoff) {
                        break;
                    }
                    d += ` L ${path.x[i]} ${path.y[i]}`;
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

/**
 * Creates the paths corresponding to the individual lenses in the optical system.
 * @param { import("./rayTracerTypes/rays").Description} descr - The description of the optical system.
 * @returns {Paths}
 */
function surfacesIntoLenses(descr) {
    const surfaceSamples = descr.cutaway_view.path_samples;

    let paths = new Array();
    let topPath;
    let bottomPath;
    for (let component of descr.components_view) {
        if (component["Element"]) {
            let path = {x: [], y: [], z: []};
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
            path.x.push(bottomPath[0][0]);
            path.y.push(bottomPath[0][1]);
            path.z.push(bottomPath[0][2]);

            // Add the lens surface points
            path.x = path.x.concat(surfSamples[smallerSurfIdx].map(sample => sample[0]));
            path.y = path.y.concat(surfSamples[smallerSurfIdx].map(sample => sample[1]));
            path.z = path.z.concat(surfSamples[smallerSurfIdx].map(sample => sample[2]));

            // Add the top of the lens
            path.x.push(topPath[0][0]);
            path.y.push(topPath[0][1]);
            path.z.push(topPath[0][2]);

            // Add the opposite side of the lens
            path.x = path.x.concat(surfSamples[biggerSurfIdx].toReversed().map(sample => sample[0]));
            path.y = path.y.concat(surfSamples[biggerSurfIdx].toReversed().map(sample => sample[1]));
            path.z = path.z.concat(surfSamples[biggerSurfIdx].toReversed().map(sample => sample[2]));

            // Add the bottom of the lens
            path.x.push(bottomPath[1][0]);
            path.y.push(bottomPath[1][1]);
            path.z.push(bottomPath[1][2]);

            // Close the path
            path.x.push(bottomPath[0][0]);
            path.y.push(bottomPath[0][1]);
            path.z.push(bottomPath[0][2]);
            
            paths.push(path);
        }
    }

    return paths
}            

/** 
 * Creates the path for a surface of type Stop.
 * @param {Array<[number, number, number>]} surfaceSamples: a map of surface samples for the stop surface
 * @param { import("./rayTracerTypes/rays.js").Description } descr: a description of the optical system
 * @returns {Paths} The paths for the stop surface
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
        {x: [x, x], y: [yMin, surfYMin], z: [z, z]},
        {x: [x, x], y: [surfYMax, yMax], z: [z, z]}
    ]

    return paths;
}

/*
    * Determines whether a surface is part of a lens.
    * descr: a description of the optical system
    * surfId: the id of the surface
*/
function isLensSurface(descr, surfId) {
    for (let component of descr.components_view) {
        if (component["Element"]) {
            const surfIds = component["Element"]["surf_idxs"];
            if (surfIds.includes(surfId)) {
                return true;
            }
        }
    }

    return false;
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
    * padding: the padding to add to the bounding box
    * returns: [xMin, yMin, zMin, xMax, yMax, zMax]
*/
function boundingBox(samples, padding=0.01) {
    let xMin = Infinity;
    let xMax = -Infinity;
    let yMin = Infinity;
    let yMax = -Infinity;
    let zMin = Infinity;
    let zMax = -Infinity;

    for (let surfSamples of samples.values()) {
        for (let sample of surfSamples) {
            xMin = Math.min(xMin, sample[0]) - padding;
            xMax = Math.max(xMax, sample[0]) + padding;
            yMin = Math.min(yMin, sample[1]) - padding;
            yMax = Math.max(yMax, sample[1]) + padding;
            zMin = Math.min(zMin, sample[2]) - padding;
            zMax = Math.max(zMax, sample[2]) + padding;
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

/**
 * Converts paths from system coordinates to SVG coordinates.
 * @param {Paths} paths - The paths to convert to SVG coordinates.
 * @param {[number, number]} systemCenter - The center of the system in system coordinates.
 * @param {[number, number]} svgCenter - The center of the SVG in SVG coordinates.
 * @param {number} scaleFactor - The scaling factor to apply to the paths to fit them in the SVG.
 * @returns {Paths} The paths in SVG coordinates.
 */
function toSVGCoordinates(paths, systemCenter, svgCenter, scaleFactor = 6) {
    let transformedPaths = new Array();
    for (let [_pathId, pathSamples] of paths.entries()) {
        let transformedPathSamples = {x: [], y: [], z: []};
        for (let i = 0; i < pathSamples.x.length; i++) {
            // Transpose the y and z coordinates because the SVG y-axis points down.
            // Take the negative of the y-coordinate because it points down the screen.
            // Shift the center of mass of the samples to that of the SVG.
            transformedPathSamples.x.push(svgCenter[0] + scaleFactor * (pathSamples.z[i] - systemCenter[2]));
            transformedPathSamples.y.push(svgCenter[1] - scaleFactor * (pathSamples.y[i] - systemCenter[1]));
            transformedPathSamples.z.push(0);
        }

        if (pathSamples.cutoff !== undefined) {
            transformedPathSamples.cutoff = pathSamples.cutoff;
        }
        transformedPaths.push(transformedPathSamples);
    }
    return transformedPaths;
}

/**
 * Converts an ordered set of coordinates to a path to draw on the SVG.
 * @param {Array<[number, number, number]>} coords
 * @returns {Path}
 */
function coordsToPath(coords) {
    let path = {x: [], y: [], z: []};
    for (let coord of coords) {
        path.x.push(coord[0]);
        path.y.push(coord[1]);
        path.z.push(coord[2]);
    }
    return path;
}

/**
 * Converts ray trace results to ray paths for drawing.
 * @param { import("./rayTracerTypes/rays").TraceResultsCollection } traceResultsCollection - The results of tracing rays through the optical system.
 * @returns {Paths} The ray paths.
 */
function resultsToRayPaths(traceResultsCollection) {
    let paths = new Array();

    for (let result of traceResultsCollection.results) {
        for (let i = 0; i < numRays(result.ray_bundle); i++) {
            let path = {x: [], y: [], z: []};
            let rayIntersections = getRayIntersections(result.ray_bundle, i);
            for (let j = 0; j < rayIntersections.length; j++) {
                path.x.push(rayIntersections[j].pos[0]);
                path.y.push(rayIntersections[j].pos[1]);
                path.z.push(rayIntersections[j].pos[2]);

                // If the ray terminated at this surface, stop drawing the path
                if (rayTerminatedAt(result.ray_bundle, i) === j + 1) {
                    path.cutoff = j;
                }
            }
            paths.push(path);
        }
    }

    return paths;
}

/**
 * Returns the number of rays in a ray bundle.
 * @param { import("./rayTracerTypes/rays").RayBundle } rayBundle - The ray bundle.
 * @returns {number} The number of rays.
 */
function numRays(rayBundle) {
    return rayBundle.rays.length / rayBundle.num_surfaces;
}

/**
 * Returns the surface ID where a ray terminated. Returns 0 if the ray did not terminate or is out of bounds.
 * @param { import("./rayTracerTypes/rays").RayBundle } rayBundle - The ray bundle.
 * @param {number} rayIndex - The index of the ray.
 * @returns {number} The surface ID where the ray terminated.
 */
function rayTerminatedAt(rayBundle, rayIndex) {
    const nRays = numRays(rayBundle);
    if (rayIndex >= nRays) {
        return 0;
    }

    return rayBundle.terminated[rayIndex];
}

/**
 * Returns the ray/surface intersections for a unique ray.
 * @param { import("./rayTracerTypes/rays").RayBundle } rayBundle - The ray bundle for a given wavelength and field.
 * @param {number} rayIndex - The index of the ray.
 * @returns {Array< import("./rayTracerTypes/rays").Ray >} The ray through all the surfaces, or an empty array if the ray index is out of bounds.
 */
function getRayIntersections(rayBundle, rayIndex) {
    const numSurfaces = rayBundle.num_surfaces;
    const nRays = numRays(rayBundle);
    let rays = new Array();

    if (rayIndex >= nRays) {
        return rays;
    }

    for (let i=rayIndex; i < numSurfaces * nRays; i = i + nRays) {
        rays.push(rayBundle.rays[i]);
    }

    return rays;
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

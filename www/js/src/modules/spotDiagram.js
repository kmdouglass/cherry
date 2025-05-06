/**
 * Tools for ray data calculations and transformation for spot diagrams.
 * @module spotDiagram
 */

/**
 * @typedef RayIntersections
 * @type {object}
 * @param {number} x - The x coordinate of the intersections.
 * @param {number} y - The y coordinate of the intersections.
 */

/**
 * @typedef FieldSpec
 * @type {object}
 * @param {string} [Angle] - The angle of the field.
 * @param {string} [PointSource] - The point source location of the field.
 */

/**
 * Transforms ray data from the ray tracer into a format for spot diagrams.
 * @param { import("./rayTracerTypes/rays").TraceResultsCollection } traceResultsCollection - The full trace results from the ray tracer.
 * @returns {object} The transformed ray data.
 */
export function traceResultsToSpotDiagram(traceResultsCollection) {
    let transformedResults = [];

    for (const result of traceResultsCollection) {
        const { wavelength_id, field_id, ray_bundle, chief_ray } = result;
        const numSurfaces = ray_bundle.num_surfaces;

        for (let surfaceId = 0; surfaceId < numSurfaces; surfaceId++) {
            const rayIntersections = rayBundleToRayIntersections(ray_bundle, surfaceId);
            const chiefRayIntersections = rayBundleToRayIntersections(chief_ray, surfaceId);

            transformedResults.push({
                wavelengthId: wavelength_id,
                fieldId: field_id,
                rayBundle: rayIntersections,
                chiefRay: chiefRayIntersections,
                surfaceId: surfaceId,
            });
        }
    }

    return transformedResults;
}

/**
 * Converts a ray bundle into transverse ray intersections at a given surface.
 * @param { import("./rayTracerTypes/rays").RayBundle } rayBundle - The ray bundle.
 * @param {number} surfaceIndex - The index of the surface to extract the ray intersections from.
 * @returns {RayIntersections} The ray intersections.
 */
function rayBundleToRayIntersections(rayBundle, surfaceIndex) {
    const nRays = numRays(rayBundle);
    const rayIntersectionsX = [];
    const rayIntersectionsY = [];

    for (let i = 0; i < nRays; i++) {
        const rayIndex = i + surfaceIndex * nRays;
        const terminalIndex = i;

        // Skip rays that terminated at this surface
        if (rayTerminatedAt(rayBundle, terminalIndex) !== 0) {
            continue;
        }

        const x = rayBundle.rays[rayIndex].pos[0];
        const y = rayBundle.rays[rayIndex].pos[1];

        rayIntersectionsX.push(x);
        rayIntersectionsY.push(y);
    }

    return {x: rayIntersectionsX, y: rayIntersectionsY};
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
 * Returns the surface ID where a ray terminated. Returns 0 if the ray did not terminate.
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
 * Converts the field specs to a format for spot diagrams.
 * @param {FieldSpec[]} fieldSpecs - The field specs from the ray tracer.
 * @returns {object[]} The transformed field specs.
 */
export function fieldSpecsToSpotDiagram(fieldSpecs) {
    const newSpecs = [];

    for (const fieldSpec of fieldSpecs) {
        if (fieldSpec.Angle) {
            const {angle, } = fieldSpec.Angle;
            newSpecs.push({
                value: angle,
                units: "°",
                type: "angle",
            });
        }
        if (fieldSpec.PointSource) {
            const {y, } = fieldSpec.PointSource;
            newSpecs.push({
                value: y,
                units: "mm",
                type: "y",
            });
        }
    }

    return newSpecs;
}

/**
 * Converts wavelength specs to a format for spot diagrams.
 * @param {number[]} wavelengths - The wavelengths from the ray tracer.
 * @returns {object[]} The transformed wavelength specs.
 */
export function wavelengthSpecsToSpotDiagram(wavelengths) {
    const newSpecs = [];

    for (const wavelength of wavelengths) {
        newSpecs.push({
            value: wavelength,
            units: "μm",
        });
    }

    return newSpecs;
}

/**
 * @typedef {object} Validation
 * @property {boolean} isValid - Indicates if the inputs are valid.
 * @property {string} message - The validation message.
 */

/**
 * Validates the inputs for the spot diagram component.
 * @param {object[]} rayTraceResults - The ray trace results from the ray tracer.
 * @param {object[]} fields
 * @param {object[]} wavelengths
 * @returns {Validation} True if the inputs are valid, false otherwise.
 */
export function validateSpotDiagramInputs(
    rayTraceResults,
    fields,
    wavelengths,
) {
    const nFields = numFields(rayTraceResults);
    const nWavelengths = numWavelengths(rayTraceResults);
    const nSurfaces = numSurfaces(rayTraceResults);

    if (rayTraceResults.length !== nFields * nWavelengths * nSurfaces) {
        return {
            isValid: false,
            message: "There are missing rays in the ray trace results."
        };
    }

    if (fields.length !== nFields) {
        return {
            isValid: false,
            message: "The input fields do not match the number of fields in the ray trace results."
        };
    }

    if (wavelengths.length !== nWavelengths) {
        return {
            isValid: false,
            message: "The input wavelengths do not match the number of wavelengths in the ray trace results."
        };
    }

    return {
        isValid: true,
        message: ""
    };
}

function numFields(rayTraceResults) {
    const numFields = new Set();
    for (const result of rayTraceResults) {
        numFields.add(result.fieldId);
    }
    return numFields.size;
}

function numWavelengths(rayTraceResults) {
    const numWavelengths = new Set();
    for (const result of rayTraceResults) {
        numWavelengths.add(result.wavelengthId);
    }
    return numWavelengths.size;
}

function numSurfaces(rayTraceResults) {
    const numSurfaces = new Set();
    for (const result of rayTraceResults) {
        numSurfaces.add(result.surfaceId);
    }
    return numSurfaces.size;
}

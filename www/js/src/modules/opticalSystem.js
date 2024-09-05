/* Converts data from the UI state into inputs to the raytrace engine. */
export function convertUIStateToEngineFormat(surfaces) {
    const AIR = {"real": {"Constant": 1.0}, "imag": null};

    // QUESTION: Can float conversion be done any better?
    function createSurfaceSpec(surface) {
        if (surface.type === 'Object' || surface.type === 'Image') {
            return surface.type;
        } else if (surface.type === 'Conic') {
            return {
                "Conic": {
                    "semi_diameter": parseFloat(surface.semiDiam),
                    "radius_of_curvature": parseFloat(surface.roc) || Infinity,
                    "conic_constant": 0.0,
                    "surf_type": "Refracting"
                }
            };
        } else {
            // Default to a flat surface if type is unknown
            return {
                "Conic": {
                    "semi_diameter": parseFloat(surface.semiDiam),
                    "radius_of_curvature": Infinity,
                    "conic_constant": 0.0,
                    "surf_type": "Refracting"
                }
            };
        }
    }

    function createGapSpec(surface) {
        return {
            "thickness": parseFloat(surface.thickness) === 'Infinity' ? Infinity : (parseFloat(surface.thickness) || 0),
            "refractive_index": parseFloat(surface.n) ? {"real": {"Constant": parseFloat(surface.n)}, "imag": null} : AIR
        };
    }

    const surface_specs = surfaces.map(createSurfaceSpec);
    const gap_specs = surfaces.slice(0, -1).map(createGapSpec);

    return {
        surface_specs,
        gap_specs
    };
}

let instance = null;

/* Returns an opticalSystem singleton to the application. */
export function getOpticalSystem(wasmModule) {
  if (!instance && wasmModule) {
    instance = new wasmModule.OpticalSystem();
  }
  return instance;
}

/* Converts data from the UI state into inputs to the raytrace engine. */
export function convertUIStateToLibFormat(surfaces, fields, aperture, wavelengths, appModes, materialsService) {
    // QUESTION: Can float conversion be done any better?
    function createSurfaceSpec(surface) {
        if (surface.type === 'Object' || surface.type === 'Image' || surface.type === 'Probe') {
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
        } else if (surface.type === "Stop") {
            return {
                "Stop": {
                    "semi_diameter": parseFloat(surface.semiDiam)
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
        if (appModes.refractiveIndex) {
            return {
                "thickness": parseFloat(surface.thickness) === 'Infinity' ? Infinity : (parseFloat(surface.thickness) || 0),
                "refractive_index": parseFloat(surface.n)
            }
        } else {
            return {
                "thickness": parseFloat(surface.thickness) === 'Infinity' ? Infinity : (parseFloat(surface.thickness) || 0),
                "material": materialsService.selectedMaterials.get(surface.material || "") || ""
            }
        }
    }

    function createFieldSpec(field) {
        const angle = parseFloat(field.Angle.angle);
        const pupil_sampling = field.Angle.pupil_sampling;
        const pupil_sampling_type = Object.keys(pupil_sampling)[0];
        const spacing = parseFloat(pupil_sampling[pupil_sampling_type].spacing);

        return {
            "Angle": {
                "angle": angle,
                "pupil_sampling": {
                    [pupil_sampling_type]: {
                        "spacing": spacing
                    }
                }
            }
        };
    }

    function createApertureSpec(aperture) {
        const semiDiameter = parseFloat(aperture.EntrancePupil.semi_diameter);

        return {
            "EntrancePupil": {
                "semi_diameter": semiDiameter
            }
        };
    }

    function createWavelengthSpec(wavelengths) {
        // Don't take into account wavelengths if refractive index mode is enabled
        if (appModes.refractiveIndex) {
            return [0.5876];
        }
        return wavelengths.map(w => parseFloat(w));
    }

    const surfaceSpecs = surfaces.map(createSurfaceSpec);
    const gapSpecs = surfaces.slice(0, -1).map(createGapSpec);
    const fieldSpecs = fields.map(createFieldSpec);
    const apertureSpec = createApertureSpec(aperture);
    const wavelengthSpecs = createWavelengthSpec(wavelengths);

    return {
        surfaceSpecs,
        gapSpecs,
        fieldSpecs,
        apertureSpec,
        wavelengthSpecs,
    };
}

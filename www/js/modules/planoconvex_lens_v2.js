const AIR = {"real": {"Constant": 1.0}, "imag": null}
const NBK7 = {"real": {"Constant": 1.515}, "imag": null}

export const surface_specs = [
    "Object",
    {"Conic": {"semi_diameter": 12.5, "radius_of_curvature": 25.8, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 12.5, "radius_of_curvature": Infinity, "conic_constant": 0.0, "surf_type": "Refracting"}},
    "Image",
]

export const gap_specs = [
    {"thickness": Infinity, "refractive_index": AIR},
    {"thickness": 5.3, "refractive_index": NBK7},
    {"thickness": 46.6, "refractive_index": AIR},
]

export const aperture_spec = {"EntrancePupil": { "semi_diameter": 5.0 }}

export const field_specs = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
]

export const wavelengths = [0.5876]

// From https://www.comsol.com/blogs/how-to-create-complex-lens-geometries-for-ray-optics-simulations/
const AIR = {"real": {"Constant": 1.0}, "imag": null}

export const surface_specs = [
    "Object",
    {"Conic": {"semi_diameter": 28.478, "radius_of_curvature": 99.56266, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 26.276, "radius_of_curvature": -86.84002, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 21.02, "radius_of_curvature": -1187.63858, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Stop": {"semi_diameter": 16.631}},
    {"Conic": {"semi_diameter": 20.543, "radius_of_curvature": 57.47491, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 20.074, "radius_of_curvature": -54.61685, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 16.492, "radius_of_curvature": -614.68633, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 17.297, "radius_of_curvature": -38.17110, "conic_constant": 0.0, "surf_type": "Refracting"}},
    {"Conic": {"semi_diameter": 18.94, "radius_of_curvature": Infinity, "conic_constant": 0.0, "surf_type": "Refracting"}},
    "Image",
]



export const gap_specs = [
    {"thickness": Infinity, "refractive_index": AIR},
    {"thickness": 13.0, "refractive_index": {"real": {"Constant": 1.5168}, "imag": null}},
    {"thickness": 4.0, "refractive_index": {"real": {"Constant": 1.6645}, "imag": null}},
    {"thickness": 40.0, "refractive_index": AIR},
    {"thickness": 40.0, "refractive_index": AIR},
    {"thickness": 12.0, "refractive_index": {"real": {"Constant": 1.6074}, "imag": null}},
    {"thickness": 3.0, "refractive_index": {"real": {"Constant": 1.6727}, "imag": null}},
    {"thickness": 46.82210, "refractive_index": AIR},
    {"thickness": 2.0, "refractive_index": {"real": {"Constant": 1.6727}, "imag": null}},
    {"thickness": 1.87179, "refractive_index": AIR},
]

export const aperture_spec = {"EntrancePupil": { "semi_diameter": 5.0 }}

export const field_specs = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
]

export const wavelengths = [0.5876]

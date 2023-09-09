// From https://www.comsol.com/blogs/how-to-create-complex-lens-geometries-for-ray-optics-simulations/
export const surfaces = [
    {"RefractingCircularConic": {"diam": 56.956, "roc": 99.56266, "k": 0.0}},
    {"RefractingCircularConic": {"diam": 52.552, "roc": -86.84002, "k": 0.0}},
    {"RefractingCircularConic": {"diam": 42.04, "roc": -1187.63858, "k": 0.0}},
    {"Stop": {"diam": 33.262}},
    {"RefractingCircularConic": {"diam": 41.086, "roc": 57.47491, "k": 0.0}},
    {"RefractingCircularConic": {"diam": 40.148, "roc": -54.61685, "k": 0.0}},
    {"RefractingCircularConic": {"diam": 32.984, "roc": -614.68633, "k": 0.0}},
    {"RefractingCircularConic": {"diam": 34.594, "roc": -38.17110, "k": 0.0}},
    {"RefractingCircularFlat": {"diam": 37.88}},
];

export const gaps = [
    {"n": 1.5168, "thickness": 13.0},
    {"n": 1.6645, "thickness": 4.0},
    {"n": 1.0, "thickness": 40.0},
    {"n": 1.0, "thickness": 40.0},
    {"n": 1.6074, "thickness": 12.0},
    {"n": 1.6727, "thickness": 3.0},
    {"n": 1.0, "thickness": 46.82210},
    {"n": 1.6727, "thickness": 2.0},
    {"n": 1.0, "thickness": 1.87179},
];

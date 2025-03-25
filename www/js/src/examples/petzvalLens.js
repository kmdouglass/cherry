const surfaces = [
    { type: "Object", n: 1, thickness: "Infinity", semiDiam: 25.0, roc: "" },
    { type: "Conic", n: 1.5168, thickness: 13.0, semiDiam: 28.478, roc: 99.56266 },
    { type: "Conic", n: 1.6645, thickness: 4.0, semiDiam: 26.276, roc: -86.84002 },
    { type: "Conic", n: 1.0, thickness: 40.0, semiDiam: 21.02, roc: -1187.63858 },
    { type: "Stop", n: 1.0, thickness: 40.0, semiDiam: 16.631},
    { type: "Conic", n: 1.6074, thickness: 12.0, semiDiam: 20.543, roc: 57.47491 },
    { type: "Conic", n: 1.6727, thickness: 3.0, semiDiam: 20.074, roc: -54.61685 },
    { type: "Conic", n: 1.0, thickness: 46.82210, semiDiam: 20.074, roc: -614.68633 },
    { type: "Conic", n: 1.6727, thickness: 2.0, semiDiam: 17.297, roc: -38.17110 },
    { type: "Conic", n: 1.0, thickness: 1.87179, semiDiam: 18.94, roc: "Infinity" },
    { type: "Image", n: "", thickness: "", semiDiam: 25.0, roc: "" },
];

const fields = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
];

const aperture = {"EntrancePupil": { "semi_diameter": 10.0 }};

const wavelengths = [0.5876];

const appModes = { fieldType: "Angle", "refractiveIndex": true };

const exampleData = {
    "surfaces": surfaces,
    "fields": fields,
    "aperture": aperture,
    "wavelengths": wavelengths,
    "appModes": appModes,
};

export default exampleData;

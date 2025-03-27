const surfaces= [
    { type: 'Object', n: 1, thickness: 200.0, semiDiam: 12.7, roc: '' },
    { type: 'Conic', n: 1.517, thickness: 3.6, semiDiam: 12.7, roc: 102.4 },
    { type: 'Conic', n: 1, thickness: 196.1684, semiDiam: 12.7, roc: -102.4 },
    { type: 'Image', n: '', thickness: '', semiDiam: 12.7, roc: '' },
];

const fields = [
    {"PointSource": {"x": 0, "y": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"PointSource": {"x": 0, "y": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
];

const aperture = {"EntrancePupil": { "semi_diameter": 5.0 }};

const wavelengths = [0.5876];

const appModes = { fieldType: "PointSource", "refractiveIndex": true };

const exampleData = {
    "surfaces": surfaces,
    "fields": fields,
    "aperture": aperture,
    "wavelengths": wavelengths,
    "appModes": appModes,
}; 

export default exampleData;

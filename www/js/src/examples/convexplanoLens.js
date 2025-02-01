const surfaces= [
    { type: 'Object', n: 1, thickness: 'Infinity', semiDiam: 12.5, roc: '' },
    { type: 'Conic', n: 1.515, thickness: 5.3, semiDiam: 12.5, roc: 25.8 },
    { type: 'Conic', n: 1, thickness: 46.6, semiDiam: 12.5, roc: 'Infinity' },
    { type: 'Image', n: '', thickness: '', semiDiam: 12.5, roc: '' },
];

const fields = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
];

const aperture = {"EntrancePupil": { "semi_diameter": 5.0 }};

const wavelengths = [0.5876];

const appModes = { "refractiveIndex": true };

const exampleData = {
    "surfaces": surfaces,
    "fields": fields,
    "aperture": aperture,
    "wavelengths": wavelengths,
    "appModes": appModes,
}; 

export default exampleData;
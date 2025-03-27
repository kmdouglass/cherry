const surfaces= [
    { type: "Object", material: "other:air:Ciddor", thickness: "Infinity", semiDiam: 12.5, roc: "" },
    { type: "Conic", material: "glass:BK7:SCHOTT" , thickness: 5.3, semiDiam: 12.5, roc: 25.8 },
    { type: "Conic", material: "other:air:Ciddor", thickness: 46.6, semiDiam: 12.5, roc: "Infinity" },
    { type: "Image", material: "", thickness: "", semiDiam: 12.5, roc: "" },
];

const fields = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
];

const aperture = {"EntrancePupil": { "semi_diameter": 5.0 }};

const wavelengths = [0.4861 ,0.5876, 0.6563];

const appModes = { fieldType: "Angle", "refractiveIndex": false };

const exampleData = {
    "surfaces": surfaces,
    "fields": fields,
    "aperture": aperture,
    "wavelengths": wavelengths,
    "appModes": appModes,
};

export default exampleData;

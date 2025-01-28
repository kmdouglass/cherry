const surfaces= [
    { type: 'Object', material: { key: "other:air:Ciddor " }, thickness: 'Infinity', semiDiam: 12.5, roc: '' },
    { type: 'Conic', material: { key: "glass:BK7:SCHOTT" }, thickness: 5.3, semiDiam: 12.5, roc: 25.8 },
    { type: 'Conic', material: { key: "other:air:Ciddor" }, thickness: 46.6, semiDiam: 12.5, roc: 'Infinity' },
    { type: 'Image', material: {}, thickness: '', semiDiam: 12.5, roc: '' },
];

const fields = [
    {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
];

const aperture = {"EntrancePupil": { "semi_diameter": 5.0 }};

const wavelengths = [0.5876];

selectedMaterials = [
    { key: "glass:BK7:SCHOTT", name: "BK7 / N-BK7 (SCHOTT)" },
    { key: "other:air:Ciddor", name: "Air / Ciddor 1996: n 0.23–1.690 µm" }
];

const exampleData = {
    "surfaces": surfaces,
    "fields": fields,
    "aperture": aperture,
    "wavelengths": wavelengths,
    "selectedMaterials": selectedMaterials
};

export default exampleData;

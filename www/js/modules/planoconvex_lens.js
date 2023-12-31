export const surfaces = [
    {"ObjectPlane": {"diam": 50}},
    {"RefractingCircularConic": {"diam": 25.0, "roc": 25.8, "k": 0.0}},
    {"RefractingCircularFlat": {"diam": 25.0}},
    {"ImagePlane": {"diam": 50}},
]

export const gaps = [
    {"n": 1.0, "thickness": Infinity},
    {"n": 1.515, "thickness": 5.3},
    {"n": 1.0, "thickness": 46.6},
]

export const aperture = {"EntrancePupilDiameter": { diam: 10 }}

export const fields = [
    {"Angle": {"angle": 0, "wavelength": 0.5876, "sampling": {"SqGrid": {"spacing": 0.1}}}},
    {"Angle": {"angle": 5, "wavelength": 0.5876, "sampling": {"SqGrid": {"spacing": 0.1}}}}
]

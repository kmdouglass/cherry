import { convertUIStateToEngineFormat } from "./modules/opticalSystem";

import { useMemo, useState } from "react";

import "./css/cherry.css";
import CutawayView from "./components/CutawayView";
import Navbar from "./components/Navbar";
import DataEntry from "./components/DataEntry";

function App({ wasmModule }) {
    // Application state and initial values.
    const [surfaces, setSurfaces] = useState([
        { type: 'Object', n: 1, thickness: 'Infinity', semiDiam: 12.5, roc: '' },
        { type: 'Conic', n: 1.515, thickness: 5.3, semiDiam: 12.5, roc: 25.8 },
        { type: 'Conic', n: 1, thickness: 46.6, semiDiam: 12.5, roc: 'Infinity' },
        { type: 'Image', n: '', thickness: '', semiDiam: 12.5, roc: '' },
    ]);
    const [fields, setFields] = useState([
        {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
        {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
    ]);
    const [aperture, setAperture] = useState({"EntrancePupil": { "semi_diameter": 5.0 }});
    const [wavelengths, setWavelengths] = useState([0.5876]);
    const [description, setDescription] = useState(null);
    const [rawRayPaths, setRawRayPaths] = useState(null);

    // Update the optical system during each render
    useMemo(() => {
        if (wasmModule) {
            const opticalSystem = new wasmModule.OpticalSystem();
            const { surfaceSpecs, gapSpecs, fieldSpecs } = convertUIStateToEngineFormat(surfaces, fields);

            //Build the optical system
            opticalSystem.setSurfaces(surfaceSpecs);
            opticalSystem.setGaps(gapSpecs);
            opticalSystem.setFields(fieldSpecs);
            opticalSystem.setAperture(aperture);
            opticalSystem.setWavelengths(wavelengths);
            opticalSystem.build();

            console.log("Surface specs:", surfaceSpecs);
            console.log("Gap specs:", gapSpecs);
            console.log("Field specs:", fieldSpecs);
            console.log("Aperture:", aperture);
            console.log("Wavelengths:", wavelengths);

            console.log("Fields:", fields);

            // Render the optical system
            setDescription(opticalSystem.describe());
            setRawRayPaths(opticalSystem.traceChiefAndMarginalRays());

            console.log("Rendered optical system.");
        }
    }, [wasmModule, surfaces, fields, aperture]);


    return (
        <div className="App">
            <Navbar />
            <div className="container">
                <CutawayView description={description} rawRayPaths={rawRayPaths} />
                <DataEntry
                    surfaces={surfaces} setSurfaces={setSurfaces}
                    fields={fields} setFields={setFields}
                    aperture={aperture} setAperture={setAperture}                
                />
            </div>
        </div>
    );
}

export default App;

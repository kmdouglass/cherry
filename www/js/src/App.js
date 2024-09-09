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
        { type: 'Conic', n: 1, thickness: 46.6, semiDiam: 12.5, roc: 100 },
        { type: 'Image', n: '', thickness: '', semiDiam: 12.5, roc: '' },
    ]);
    const [fields, setFields] = useState([
        {"Angle": {"angle": 0, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}},
        {"Angle": {"angle": 5, "pupil_sampling": {"SquareGrid": {"spacing": 0.1}}}}
    ]);
    const [aperture, setAperture] = useState({"EntrancePupil": { "semi_diameter": 5.0 }});
    const [wavelengths, setWavelengths] = useState([0.5876]);
    const [description, setDescription] = useState(null);
    const [ rawRayPaths, setRawRayPaths ] = useState(null);

    // Update the optical system during each render
    useMemo(() => {
        if (wasmModule) {
            const opticalSystem = new wasmModule.OpticalSystem();
            const { surface_specs, gap_specs } = convertUIStateToEngineFormat(surfaces);

            //Build the optical system
            opticalSystem.setSurfaces(surface_specs);
            opticalSystem.setGaps(gap_specs);
            opticalSystem.setFields(fields);
            opticalSystem.setAperture(aperture);
            opticalSystem.setWavelengths(wavelengths);
            opticalSystem.build();

            // Render the optical system
            setDescription(opticalSystem.describe());
            setRawRayPaths(opticalSystem.traceChiefAndMarginalRays());

            console.log("Surface specs:", surface_specs);
            console.log("Gap specs:", gap_specs);
            console.log("Fields:", fields);
            console.log("Aperture:", aperture);
            console.log("Wavelengths:", wavelengths);

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

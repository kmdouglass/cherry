import { convertUIStateToEngineFormat, getOpticalSystem } from "./modules/opticalSystem";

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

    const opticalSystem = getOpticalSystem(wasmModule);

    // Update the optical system during each render
    const systemData = useMemo(() => {
        if (wasmModule) {
            try {
                //console.debug("Raw surfaces:", surfaces);

                const { surfaceSpecs, gapSpecs, fieldSpecs, apertureSpec } = convertUIStateToEngineFormat(surfaces, fields, aperture);

                //Build the optical system
                opticalSystem.setSurfaces(surfaceSpecs);
                opticalSystem.setGaps(gapSpecs);
                opticalSystem.setFields(fieldSpecs);
                opticalSystem.setAperture(apertureSpec);
                opticalSystem.setWavelengths(wavelengths);
                opticalSystem.build();

                //console.log("Surface specs:", surfaceSpecs);
                //console.log("Gap specs:", gapSpecs);
                //console.log("Field specs:", fieldSpecs);
                //console.log("Aperture:", aperture);
                //console.log("Wavelengths:", wavelengths);

                //console.log("Fields:", fields);

                // Render the optical system
                const newDescription = opticalSystem.describe();
                const newRayPaths = opticalSystem.traceChiefAndMarginalRays();

                setDescription(newDescription);
                setRawRayPaths(newRayPaths);

                return {
                    "description": newDescription,
                    "newRayPaths": newRayPaths
                }
            } catch (e) {
                console.error(e);
                return {
                    "description": null,
                    "newRayPaths": null
                }
            }
        }
    }, [wasmModule, surfaces, fields, aperture, wavelengths]);


    return (
        <div className="App">
            <Navbar 
                surfaces={surfaces} setSurfaces={setSurfaces}
                fields={fields} setFields={setFields}
                aperture={aperture} setAperture={setAperture}
                wavelengths={wavelengths} setWavelengths={setWavelengths}
                description={systemData.description}
            />
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

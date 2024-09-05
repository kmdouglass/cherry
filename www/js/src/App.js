import { convertUIStateToEngineFormat } from "./modules/opticalSystem";

import { useMemo, useState } from "react";

import "./css/cherry.css";
import CutawayView from "./components/CutawayView";
import Navbar from "./components/Navbar";
import DataEntry from "./components/DataEntry";

function App({ wasmModule }) {
    // Application state
    const [surfaces, setSurfaces] = useState([
        { type: 'Object', n: 1, thickness: 'Infinity', diam: 25, roc: '' },
        { type: 'Conic', n: 1.515, thickness: 5.3, diam: 25, roc: 25.8 },
        { type: 'Conic', n: 1, thickness: 46.6, diam: 25, roc: 100 },
        { type: 'Image', n: '', thickness: '', diam: 25, roc: '' },
    ]);
    const [fields, setFields] = useState(null);
    const [aperture, setAperture] = useState(null);
    const [results, setResults] = useState(null);

    // Update the optical system during each render
    useMemo(() => {
        if (wasmModule) {
            const opticalSystem = new wasmModule.OpticalSystem();
            const { surface_specs, gap_specs } = convertUIStateToEngineFormat(surfaces);
            console.log("Surface specs:", surface_specs);
            console.log("Gap specs:", gap_specs);
            console.log("Rendered optical system.");
        }
    }, [wasmModule, surfaces, fields, aperture]);


    return (
        <div className="App">
            <Navbar />
            <div className="container">
                <CutawayView results={results} />
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

import { useState } from "react";

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

import { convertUIStateToLibFormat, getOpticalSystem } from "./modules/opticalSystem";
import { useComputeService } from "./services/computeService";
import { useMaterialsService } from "./services/materialsDataService";

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";

import "./css/cherry.css";
import showAlert from "./modules/alerts";
import CutawayView from "./components/CutawayView";
import Navbar from "./components/Navbar";
import SpecsExplorer from "./components/explorers/SpecsExplorer";
import MaterialsExplorer from "./components/explorers/MaterialsExplorer";

function App({ wasmModule }) {
    // Load the material data
    const { materialsService, isLoadingInitialData, isLoadingFullData, error } = useMaterialsService();

    // Start the compute service and results listener
    const { computeService, isComputeServiceInitializing } = useComputeService();
    const results = useSyncExternalStore(
        (onStoreChange) => computeService.subscribe(onStoreChange),
        () => computeService.results
    )

    // GUI state
    const [activeExplorersTab, setExplorersActiveTab] = useState('specs');
    const [invalidSpecsFields, setInvalidSpecsFields] = useState({});
    const [appModes, setAppModes] = useState({ refractiveIndex: true });

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
    const [convertedSpecs, setConvertedSpecs] = useState({});
    const [description, setDescription] = useState(null);
    const [rawRayPaths, setRawRayPaths] = useState(null);

    // Update the optical system during each render
    const systemData = useMemo(() => {
        if (wasmModule) {
            try {
                const opticalSystem = getOpticalSystem(wasmModule);

                const { surfaceSpecs, gapSpecs, fieldSpecs, apertureSpec, wavelengthSpecs } = convertUIStateToLibFormat(
                    surfaces,
                    fields,
                    aperture,
                    wavelengths,
                    appModes,
                    materialsService,
                );
                const gapMode = appModes.refractiveIndex ? wasmModule.GapMode.RefractiveIndex : wasmModule.GapMode.Material;

                //Build the optical system
                opticalSystem.setSurfaces(surfaceSpecs);
                opticalSystem.setGaps(gapSpecs, gapMode);
                opticalSystem.setFields(fieldSpecs);
                opticalSystem.setAperture(apertureSpec);
                opticalSystem.setWavelengths(wavelengthSpecs);
                opticalSystem.build();

                const newConvertedSpecs = {
                    "surfaces": surfaceSpecs,
                    "gaps": gapSpecs,
                    "fields": fieldSpecs,
                    "aperture": apertureSpec,
                    "wavelengths": wavelengthSpecs,
                    "gapMode": gapMode,
                    ...convertedSpecs
                }
                setConvertedSpecs(newConvertedSpecs);

                // Render the optical system
                const newDescription = opticalSystem.describe();
                const newRayPaths = opticalSystem.traceChiefAndMarginalRays();

                setDescription(newDescription);
                setRawRayPaths(newRayPaths);

                return {
                    "description": newDescription,
                    "newRayPaths": newRayPaths
                }
            } catch (error) {
                showAlert(error instanceof Error ? error.message : "Error creating optical system");
                console.error(error);
                return {
                    "description": null,
                    "newRayPaths": null
                }
            }
        }
    }, [wasmModule, surfaces, fields, aperture, wavelengths, appModes]);

    // Send the optical system to the compute service
    useEffect(() => {
        if (isComputeServiceInitializing) return;

        // Compute full results for the optical system
        computeService.compute(convertedSpecs);
    }, [convertedSpecs, isComputeServiceInitializing]);


    const handleExplorersTabClick = (tab) => {
        // Don't allow switching tabs if SpecsExplorer cell is invalid
        if (thereAreInvalidSpecsFields(invalidSpecsFields)) return;
        setExplorersActiveTab(tab);
    }

    const thereAreInvalidSpecsFields = (invalidFieldsObj) => {
        // Check that the fields object is not empty.
        // Javascript makes me so sad
        return !(Object.keys(invalidFieldsObj).length === 0) && invalidFieldsObj.constructor === Object;
    }

    const renderSpecsExplorerTabContent = () => {
        switch(activeExplorersTab) {
            case 'specs':
                return <SpecsExplorer
                    surfaces={surfaces} setSurfaces={setSurfaces}
                    fields={fields} setFields={setFields}
                    aperture={aperture} setAperture={setAperture}
                    wavelengths={wavelengths} setWavelengths={setWavelengths}
                    invalidFields={invalidSpecsFields} setInvalidFields={setInvalidSpecsFields}
                    appModes={appModes} setAppModes={setAppModes}
                    materialsService={materialsService}
                />;
            case 'materials':
                return <MaterialsExplorer materialsService={materialsService} isLoadingFullData={isLoadingFullData} />;
            default:
                return null;
        }
    }

    // --------------------------------------------------------------------------------
    // Rendering
    if (isLoadingInitialData || isComputeServiceInitializing) {
        return <div>Loading...</div>;
    }

    // TODO Handle error
    //if (error) {
    //    return <div>Error loading materials: {error.message}</div>;
    //}

    return (
        <div className="App">
            <Navbar 
                surfaces={surfaces} setSurfaces={setSurfaces}
                fields={fields} setFields={setFields}
                aperture={aperture} setAperture={setAperture}
                wavelengths={wavelengths} setWavelengths={setWavelengths}
                description={systemData.description}
                appModes={appModes} setAppModes={setAppModes}
                materialsService={materialsService}
            />
            <div className="container">
                <CutawayView description={description} rawRayPaths={rawRayPaths} />
                
                <div className="tabs is-centered is-small is-toggle is-toggle-rounded">
                    <ul>
                        <li className={activeExplorersTab === 'specs' ? 'is-active' : ''}>
                            <a onClick={() => handleExplorersTabClick('specs')}>Specs</a>
                        </li>
                        <li className={activeExplorersTab === 'materials' ? 'is-active' : ''}>
                            <a onClick={() => handleExplorersTabClick('materials')}>Materials</a>
                        </li>
                    </ul>
                </div>

                <div className="box">
                    {renderSpecsExplorerTabContent()}
                </div>

            </div>
        </div>
    );
}

export default App;

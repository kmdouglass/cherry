import { useEffect, useState } from 'react';
import { EVENT_WORKER_IDLE, EVENT_WORKER_BUSY } from "../services/computeContants";
import { fieldSpecsToSpotDiagram, traceResultsToSpotDiagram, validateSpotDiagramInputs, wavelengthSpecsToSpotDiagram } from "../modules/spotDiagram";
import { SpotDiagramsGrid } from "react-optics-diagrams";


const SpotDiagram = ( {fields, wavelengths, computeService} ) => {
    const [workerIdle, setWorkerIdle] = useState(true);

    // Register the workerIdle state with the compute service
    useEffect(() => {
        const unsubscribe = computeService.subscribe(EVENT_WORKER_IDLE, () => setWorkerIdle(true));
        return () => unsubscribe();
    }, []);

    // Register the workerBusy state with the compute service
    useEffect(() => {
        const unsubscribe = computeService.subscribe(EVENT_WORKER_BUSY, () => setWorkerIdle(false));
        return () => unsubscribe();
    }, []);

    const rawRayTraceResults = workerIdle ? computeService.results : {data: []};
    const rayTraceResults = traceResultsToSpotDiagram(rawRayTraceResults.data);

    const fieldSpecs = fieldSpecsToSpotDiagram(fields);
    const wavelengthSpecs = wavelengthSpecsToSpotDiagram(wavelengths);

    // Validate the inputs
    const { isValid, } = validateSpotDiagramInputs(rayTraceResults, fieldSpecs, wavelengthSpecs);

    if (isValid) {
        return (
            <SpotDiagramsGrid
                rayTraceResults={rayTraceResults}
                wavelengths={wavelengthSpecs}
                fieldSpecs={fieldSpecs}
            />
        );
    } else {
        return (
            <p>Invalid inputs for the Spot Diagram.</p>
        );
    }
}

export default SpotDiagram;

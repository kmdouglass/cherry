import init, { WasmSystemModel } from '../pkg/cherry_js.js';


async function initWorker () {

    let model;
    let isReady = false;

    globalThis.onmessage = async function (e) {
        console.log('Message received by worker from main script: ', e.data);
        if (e.data === 'init') {
            await init();
            
            model = new WasmSystemModel();
            isReady = true;
            
            globalThis.postMessage('ready');
            return;
        }

        if (isReady) {
            model.setSurfaces(e.data.surfaces);
            model.setGaps(e.data.gaps);
            model.setAperture(e.data.aperture);
            model.setFields(e.data.fields);
            model.build();

            // Perform the full ray trace
            const start = performance.now();
            const results = model.trace();
            const end = performance.now();
            console.log(`Full ray trace took ${end - start} milliseconds.`);

            globalThis.postMessage(results);
        }        
    };
}

initWorker();

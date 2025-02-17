import init, { OpticalSystem, GapMode } from "../pkg/cherry_js.js";

let wasmModule = null;

/*
    * Initialize the WASM module and return it.
    *
    * This function will only initialize the WASM module once. If it has already been initialized,
    * it will return the existing module. If you want to force a new module to be created, pass
    * `true` as the `forceNew` parameter.
    * 
    * @param {boolean} forceNew - If true, a new WASM module will be returned no matter what.
*/
export async function initializeWasm(forceNew = false) {
    if (wasmModule && !forceNew) {
        return wasmModule;
    }

    try {
        await init();
        wasmModule = { OpticalSystem, GapMode };
        return wasmModule;
    } catch (error) {
        console.error("Failed to initialize WASM module:", error);
        throw error;
    }
}

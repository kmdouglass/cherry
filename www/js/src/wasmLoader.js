import init, { OpticalSystem } from "../pkg/cherry_js.js";

let wasmModule = null;

export async function initializeWasm() {
    if (wasmModule) {
        return wasmModule;
    }

    try {
        await init();
        wasmModule = { OpticalSystem };
        return wasmModule;
    } catch (error) {
        console.error("Failed to initialize WASM module:", error);
        throw error;
    }
}

import init, { OpticalSystem, GapMode } from "../pkg/cherry_js.js";

let wasmModule = null;

export async function initializeWasm() {
    if (wasmModule) {
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

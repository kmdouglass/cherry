import { WasmSystemModel } from "cherry";

let wasmSystemModel = new WasmSystemModel();

let surface1 = {"RefractingCircularConic": {"diam": 25.0, "roc": 1.0, "k": 0.0}};
let gap1 = {"n": 1.5, "thickness": 1.0};
let surface2 = {"RefractingCircularConic": {"diam": 25.0, "roc": -1.0, "k": 0.0}};
let gap2 = {"n": 1.0, "thickness": 10.0};

wasmSystemModel.insertSurfaceAndGap(1, surface1, gap1);
wasmSystemModel.insertSurfaceAndGap(2, surface2, gap2);

console.log(wasmSystemModel);

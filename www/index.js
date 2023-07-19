import { SystemModel } from "cherry";

// Create f = 50.1 mm a planoconvex lens comprised of two surfaces, the first one being spherical.
// This corrseponds to Thorlabs part no. LA1255.
const diam0 = 25.0; // mm
const n0 = 1.515; // refractive index of glass
const roc0 = 25.8; // mm
const K0 = 0;  // spherical
const thickness0 = 5.3;  // mm
const diam1 = 25.0; // mm
const n1 = 1.0; // refractive index of air
const backFocalLength= 46.6; // mm


// Create a system with the two surfaces
let system = new SystemModel();
system.pushSurfRefrCircConic(0, diam0, n0, roc0, K0);
system.pushSurfRefrCircFlat(thickness0, diam1, n1);
let samples = system.sampleSurfYZ(0, 20);

console.log(samples);
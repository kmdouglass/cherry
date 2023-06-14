import { SystemModel } from "cherry";

const PT_COLOR = "#000000";
const SCALE = 25;

// Create a new optical system model
const model = SystemModel.new();

// Get the canvas element and context
const canvas = document.getElementById("systemModelCanvas");
const ctx = canvas.getContext("2d");
const width = canvas.width;
const height = canvas.height;

// Draw the system at the center of the canvas by looping over each point and drawing a circle at each y, z coordinate
const pts = model.render();
pts.forEach((pt) => {
    ctx.beginPath();
    ctx.arc((width / 2 + pt[2]* SCALE), (height / 2 + pt[1] * SCALE), 1, 0, 2 * Math.PI);
    ctx.fillStyle = PT_COLOR;
    ctx.fill();
});

console.log(pts);
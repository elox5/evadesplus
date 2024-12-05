import { clearCanvas, drawCircle, setBackground, setupCanvas } from "./rendering.js";

async function main() {
    setupCanvas(32, 24, 13.5);
    setBackground("#aaa");
    clearCanvas();

    drawCircle(0, 0, 0.5);
    drawCircle(3, 0, 0.3, "red", true);
}
window.main = main;
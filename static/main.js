import { clearCanvas, drawCircle, drawGrid, setBackground, setupCanvas } from "./rendering.js";

async function main() {
    setupCanvas(32, 24, 14);
    setBackground("#aaa");
    clearCanvas();
    drawGrid(0.5, 0.5);

    drawCircle(0, 0, 0.5);
    drawCircle(3, 0, 0.3, "red", true);
}
window.main = main;
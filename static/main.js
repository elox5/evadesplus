import { clearCanvas, drawCircle, drawGrid, drawRect, setBackground, setupCanvas } from "./rendering.js";

async function main() {
    setupCanvas(32, 24, 14);
    setBackground("#aaa");
    clearCanvas();
    drawGrid();

    drawCircle(0, 0, 0.5);
    drawCircle(3, 0, 0.3, "red", true);

    drawRect(-5, -3, 4, 5);
}
window.main = main;
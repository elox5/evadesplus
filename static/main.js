import { clearCanvas, drawCircle, drawGrid, drawRect, setBackground, setDrawOffset, setupCanvas } from "./rendering.js";

async function main() {
    setupCanvas(32, 24, 14);
    setDrawOffset(53, 2);
    setBackground("#aaa");

    clearCanvas();
    drawGrid();

    drawRect(45, -3, 4, 5, "gold");

    drawCircle(53, 2, 0.5, "red");
    drawCircle(45, -3, 0.5, "#777", true);
}
window.main = main;
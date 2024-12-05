import { clearCanvas, drawCircle, setBackground, setupCanvas } from "./rendering.js";

async function main() {
    setupCanvas(32, 24, 14);
    setBackground("#aaa");
    clearCanvas(true);

    drawCircle(0, 0, 0.5);
    drawCircle(3, 0, 0.3, "red", true);
}
window.main = main;
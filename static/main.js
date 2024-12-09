import { renderSettings, clearCanvas, drawCircle, drawGrid, drawRect, setBackground, setDrawOffset, setupCanvas, drawLine, drawCircleOutline, drawCircleFrame, drawRectOutline, drawRectFrame, drawText } from "./rendering.js";
import { input, setupInput, inputSettings } from "./input.js";
import { connect, establishInputConnection, establishRenderConnection } from "./networking.js";

const canvasWrapper = document.querySelector("#canvas-wrapper");
const connectionPanel = document.querySelector("#connection-panel");
const nameInput = document.querySelector("#name-input");
const connectButton = document.querySelector("#connect-button");

async function main() {
    renderSettings.tileSize = 40;
    renderSettings.canvasWidth = 24;
    renderSettings.canvasHeight = 14;

    setupCanvas();
    setBackground("#aaa");

    connectButton.onclick = handleConnection;
}
window.main = main;

async function handleConnection() {
    let name = nameInput.value;

    console.log("Connecting");


    if (name.trim().length === 0) {
        return;
    }

    await connect(name);
    establishInputConnection();
    establishRenderConnection(handleRenderUpdate);
    setupInput();

    canvasWrapper.classList.remove("hidden");
    connectionPanel.classList.add("hidden");
}

function handleRenderUpdate(data) {
    const offsetXBytes = data.slice(0, 4);
    const offsetYBytes = data.slice(4, 8);

    const offsetX = new Float32Array(offsetXBytes.buffer)[0];
    const offsetY = new Float32Array(offsetYBytes.buffer)[0];

    const length = data[8];

    const nodes = [];

    for (let i = 0; i < length; i++) {
        let idx = 9 + i * 17;

        let xBytes = data.slice(idx, idx + 4);
        let yBytes = data.slice(idx + 4, idx + 8);
        let rBytes = data.slice(idx + 8, idx + 12);
        let colorBytes = data.slice(idx + 12, idx + 16);
        let hasBorder = data[idx + 16] === 1;

        let x = new Float32Array(xBytes.buffer)[0];
        let y = new Float32Array(yBytes.buffer)[0];
        let radius = new Float32Array(rBytes.buffer)[0];
        let r = colorBytes[0];
        let g = colorBytes[1];
        let b = colorBytes[2];
        let a = colorBytes[3];
        let color = `rgba(${r}, ${g}, ${b}, ${a})`;

        nodes.push({
            x,
            y,
            radius,
            color,
            hasBorder
        })
    }

    renderFrame({ x: offsetX, y: offsetY }, [], nodes);
}

function renderFrame(offset, rects, entities) {
    clearCanvas();
    setDrawOffset(offset.x, offset.y);

    for (const rect of rects) {
        drawRect(rect.x, rect.y, rect.w, rect.h, rect.color, rect.alpha);
    }

    drawGrid();

    for (const entity of entities) {
        drawCircle(entity.x, entity.y, entity.radius, entity.color, entity.border);
    }

    let range = inputSettings.mouseInputRange;
    drawLine(offset.x, offset.y, offset.x + (input.x * range), offset.y + (input.y * range), "yellow", 2);
    drawCircleOutline(offset.x, offset.y, range, "orange", 2);
    drawText(offset.x, offset.y + 1, "Player", "black", 16, "bold");
}
import { renderSettings, clearCanvas, drawCircle, drawRect, setDrawOffset, setupCanvas, drawLine, drawCircleOutline, drawText, renderArea } from "./rendering.js";
import { input, setupInput, inputSettings } from "./input.js";
import { connect, establishUniConnection, establishInputConnection, establishRenderConnection } from "./networking.js";

const gameContainer = document.querySelector("#game-container");
const connectionPanel = document.querySelector("#connection-panel");
const nameInput = document.querySelector("#name-input");
const connectButton = document.querySelector("#connect-button");
const areaName = document.querySelector("#area-name");

async function main() {
    renderSettings.tileSize = 40;

    connectButton.onclick = handleConnection;
}
window.onload = main;

async function handleConnection() {
    let name = nameInput.value;

    console.log("Connecting");


    if (name.trim().length === 0) {
        return;
    }

    await connect(name);
    establishInputConnection();
    establishRenderConnection(handleRenderUpdate);
    establishUniConnection([
        {
            header: "ADEF",
            callback: handleAreaUpdate,
        }
    ]);
    setupInput();

    gameContainer.classList.remove("hidden");
    connectionPanel.classList.add("hidden");

    setupCanvas();
}

function handleAreaUpdate(data) {
    const widthBytes = data.slice(0, 4);
    const heightBytes = data.slice(4, 8);

    const width = new Float32Array(widthBytes.buffer)[0];
    const height = new Float32Array(heightBytes.buffer)[0];

    const colorBytes = data.slice(8, 12);
    const r = colorBytes[0];
    const g = colorBytes[1];
    const b = colorBytes[2];
    const a = colorBytes[3];
    const color = `rgba(${r}, ${g}, ${b}, ${a})`;

    const nameLengthBytes = data.slice(12, 16);
    const nameLength = new Uint32Array(nameLengthBytes.buffer)[0];

    const nameBytes = data.slice(16, 16 + nameLength);
    const name = new TextDecoder().decode(nameBytes);

    areaName.innerHTML = name;

    renderArea(width, height, color);

    console.log("Area update:", name, width, height);
}

const nodes = [];

function handleRenderUpdate(data) {
    const offsetXBytes = data.slice(0, 4);
    const offsetYBytes = data.slice(4, 8);

    const offsetX = new Float32Array(offsetXBytes.buffer)[0];
    const offsetY = new Float32Array(offsetYBytes.buffer)[0];

    const render = data[8] === 1;

    const lengthBytes = data.slice(9, 13);
    const length = new Uint32Array(lengthBytes.buffer)[0];

    let idx = 13;

    for (let i = 0; i < length; i++) {
        let xBytes = data.slice(idx, idx + 4);
        let yBytes = data.slice(idx + 4, idx + 8);
        let rBytes = data.slice(idx + 8, idx + 12);
        let colorBytes = data.slice(idx + 12, idx + 16);
        let hasBorder = data[idx + 16] === 1;
        let nameLength = data[idx + 17];

        idx += 18;

        let node = {};

        node.x = new Float32Array(xBytes.buffer)[0];
        node.y = new Float32Array(yBytes.buffer)[0];
        node.radius = new Float32Array(rBytes.buffer)[0];
        let r = colorBytes[0];
        let g = colorBytes[1];
        let b = colorBytes[2];
        let a = colorBytes[3];
        node.color = `rgba(${r}, ${g}, ${b}, ${a})`;
        node.hasBorder = hasBorder;

        if (nameLength > 0) {
            node.name = String.fromCharCode(...data.slice(idx, idx + nameLength));
            idx += nameLength;
        }

        nodes.push(node);
    }

    if (render) {
        renderFrame({ x: offsetX, y: offsetY }, [], nodes);
        nodes.length = 0;
    }
}

function renderFrame(offset, rects, nodes) {
    clearCanvas();
    setDrawOffset(offset.x, offset.y);

    for (const rect of rects) {
        drawRect(rect.x, rect.y, rect.w, rect.h, rect.color, rect.hasBorder);
    }

    for (const node of nodes) {
        drawCircle(node.x, node.y, node.radius, node.color, node.hasBorder);

        if (node.name !== undefined) {
            drawText(node.x, node.y + 1, node.name, "black", 16, "bold");
        }
    }

    let range = inputSettings.mouseInputRange;
    drawLine(offset.x, offset.y, offset.x + (input.x * range), offset.y + (input.y * range), "yellow", 2);
    drawCircleOutline(offset.x, offset.y, range, "orange", 2);
}
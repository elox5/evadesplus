import { renderSettings, setupCanvas, renderArea, renderFrame } from "./rendering.js";
import { setupInput } from "./input.js";
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

    console.log("Connecting...");

    if (name.trim().length === 0) {
        console.log("Name empty");
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

    const wallsLengthBytes = data.slice(12, 14);
    const wallsLength = new Uint16Array(wallsLengthBytes.buffer)[0];

    const safeZonesLengthBytes = data.slice(14, 16);
    const safeZonesLength = new Uint16Array(safeZonesLengthBytes.buffer)[0];

    let idx = 16;

    const walls = [];
    const safeZones = [];

    for (let i = 0; i < wallsLength; i++) {
        let xBytes = data.slice(idx, idx + 4);
        let yBytes = data.slice(idx + 4, idx + 8);
        let wBytes = data.slice(idx + 8, idx + 12);
        let hBytes = data.slice(idx + 12, idx + 16);

        let x = new Float32Array(xBytes.buffer)[0];
        let y = new Float32Array(yBytes.buffer)[0];
        let w = new Float32Array(wBytes.buffer)[0];
        let h = new Float32Array(hBytes.buffer)[0];

        walls.push({ x, y, w, h });

        idx += 16;
    }

    for (let i = 0; i < safeZonesLength; i++) {
        let xBytes = data.slice(idx, idx + 4);
        let yBytes = data.slice(idx + 4, idx + 8);
        let wBytes = data.slice(idx + 8, idx + 12);
        let hBytes = data.slice(idx + 12, idx + 16);

        let x = new Float32Array(xBytes.buffer)[0];
        let y = new Float32Array(yBytes.buffer)[0];
        let w = new Float32Array(wBytes.buffer)[0];
        let h = new Float32Array(hBytes.buffer)[0];

        safeZones.push({ x, y, w, h });

        idx += 16;
    }

    const nameLengthBytes = data.slice(idx, idx + 4);
    const nameLength = new Uint32Array(nameLengthBytes.buffer)[0];

    const nameBytes = data.slice(idx + 4, idx + 4 + nameLength);
    const name = new TextDecoder().decode(nameBytes);

    areaName.innerHTML = name;

    renderArea(width, height, color, walls, safeZones);
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
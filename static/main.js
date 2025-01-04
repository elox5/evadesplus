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
    let name = nameInput.value.trim();

    console.log("Connecting...");

    if (name.length === 0) {
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
    const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

    const wallsLengthBytes = data.slice(12, 14);
    const wallsLength = new Uint16Array(wallsLengthBytes.buffer)[0];

    const safeZonesLengthBytes = data.slice(14, 16);
    const safeZonesLength = new Uint16Array(safeZonesLengthBytes.buffer)[0];

    const portalsLengthBytes = data.slice(16, 18);
    const portalsLength = new Uint16Array(portalsLengthBytes.buffer)[0];

    let idx = 18;

    const walls = [];
    const safeZones = [];
    const portals = [];

    for (let i = 0; i < wallsLength; i++) {
        const xBytes = data.slice(idx, idx + 4);
        const yBytes = data.slice(idx + 4, idx + 8);
        const wBytes = data.slice(idx + 8, idx + 12);
        const hBytes = data.slice(idx + 12, idx + 16);

        const x = new Float32Array(xBytes.buffer)[0];
        const y = new Float32Array(yBytes.buffer)[0];
        const w = new Float32Array(wBytes.buffer)[0];
        const h = new Float32Array(hBytes.buffer)[0];

        walls.push({ x, y, w, h });

        idx += 16;
    }

    for (let i = 0; i < safeZonesLength; i++) {
        const xBytes = data.slice(idx, idx + 4);
        const yBytes = data.slice(idx + 4, idx + 8);
        const wBytes = data.slice(idx + 8, idx + 12);
        const hBytes = data.slice(idx + 12, idx + 16);

        const x = new Float32Array(xBytes.buffer)[0];
        const y = new Float32Array(yBytes.buffer)[0];
        const w = new Float32Array(wBytes.buffer)[0];
        const h = new Float32Array(hBytes.buffer)[0];

        safeZones.push({ x, y, w, h });

        idx += 16;
    }

    for (let i = 0; i < portalsLength; i++) {
        const xBytes = data.slice(idx, idx + 4);
        const yBytes = data.slice(idx + 4, idx + 8);
        const wBytes = data.slice(idx + 8, idx + 12);
        const hBytes = data.slice(idx + 12, idx + 16);

        const x = new Float32Array(xBytes.buffer)[0];
        const y = new Float32Array(yBytes.buffer)[0];
        const w = new Float32Array(wBytes.buffer)[0];
        const h = new Float32Array(hBytes.buffer)[0];

        const r = data[idx + 16];
        const g = data[idx + 17];
        const b = data[idx + 18];
        const a = data[idx + 19];

        const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

        portals.push({ x, y, w, h, color });

        idx += 20;
    }

    const nameLengthBytes = data.slice(idx, idx + 4);
    const nameLength = new Uint32Array(nameLengthBytes.buffer)[0];

    const nameBytes = data.slice(idx + 4, idx + 4 + nameLength);
    const name = new TextDecoder().decode(nameBytes);

    areaName.innerHTML = name;

    renderArea(width, height, color, walls, safeZones, portals);
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
        node.color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;
        node.hasBorder = hasBorder;

        if (nameLength > 0) {
            const decoder = new TextDecoder("utf-8");
            node.name = decoder.decode(data.slice(idx, idx + nameLength));

            if (node.name.endsWith("%d")) {
                node.name = node.name.slice(0, node.name.length - 2);
                node.downed = true;
            }

            idx += nameLength;
        }

        nodes.push(node);
    }

    if (render) {
        renderFrame({ x: offsetX, y: offsetY }, [], nodes);
        nodes.length = 0;
    }
}
import { setupCanvas, renderArea, renderFrame } from "./rendering.js";
import { setupInput } from "./input.js";
import { connect, establishUniConnection, establishInputConnection, establishRenderConnection } from "./networking.js";
import { reportBandwidth, reportFrameStart } from "./metrics.js";
import { leaderboard } from "./leaderboard.js";
import { chat } from "./chat.js";

const gameContainer = document.querySelector("#game-container");
const connectionPanel = document.querySelector("#connection-panel");
const nameInput = document.querySelector("#name-input");
const connectButton = document.querySelector("#connect-button");
const areaNameHeading = document.querySelector("#area-name");

export const cache = {};

async function main() {
    await initCache();

    connectButton.disabled = false;
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

    connectButton.disabled = true;

    await connect(name);
    establishInputConnection();
    establishRenderConnection(handleRenderUpdate);
    establishUniConnection([
        {
            header: "ADEF",
            callback: handleAreaUpdate,
        },
        {
            header: "LBAD",
            callback: handleLeaderboardAdd,
        },
        {
            header: "LBRM",
            callback: handleLeaderboardRemove,
        },
        {
            header: "LBTR",
            callback: handleLeaderboardTransfer,
        },
        {
            header: "LBSD",
            callback: handleLeaderboardSetDowned,
        },
        {
            header: "LBST",
            callback: handleLeaderboardStateUpdate,
        },
        {
            header: "CHBR",
            callback: handleChatBroadcast
        },
    ]);
    setupInput();

    gameContainer.classList.remove("hidden");
    connectionPanel.classList.add("hidden");

    setupCanvas();
}

async function initCache() {
    const json = await fetch("/cache").then((response) => response.json());

    for (const [key, value] of Object.entries(json)) {
        cache[key] = value;
    }

    console.log("Cache loaded");
    console.log(cache);
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

    const areaNameLength = data[idx];
    idx++;

    const areaNameBytes = data.slice(idx, idx + areaNameLength);
    const areaName = new TextDecoder().decode(areaNameBytes);
    idx += areaNameLength;

    const mapIdLength = data[idx];
    idx++;

    const mapIdBytes = data.slice(idx, idx + mapIdLength);
    const mapId = new TextDecoder().decode(mapIdBytes);

    const map = cache.maps.find(m => m.id === mapId);

    const name = `${map.name} - ${areaName}`;

    areaNameHeading.innerHTML = name;
    areaNameHeading.style.color = map.text_color;

    renderArea(width, height, color, walls, safeZones, portals);
}

const nodes = [];

function handleRenderUpdate(data) {
    reportBandwidth(data.length);

    const offsetXBytes = data.slice(0, 4);
    const offsetYBytes = data.slice(4, 8);

    const offsetX = new Float32Array(offsetXBytes.buffer)[0];
    const offsetY = new Float32Array(offsetYBytes.buffer)[0];

    const render = data[8] === 1;

    const lengthBytes = data.slice(9, 13);
    const length = new Uint32Array(lengthBytes.buffer)[0];

    let idx = 13;

    for (let i = 0; i < length; i++) {
        const xBytes = data.slice(idx, idx + 4);
        const yBytes = data.slice(idx + 4, idx + 8);
        const rBytes = data.slice(idx + 8, idx + 12);
        const colorBytes = data.slice(idx + 12, idx + 16);
        const flags = data[idx + 16];
        const nameLength = data[idx + 17];

        idx += 18;

        const node = {};

        node.x = new Float32Array(xBytes.buffer)[0];
        node.y = new Float32Array(yBytes.buffer)[0];
        node.radius = new Float32Array(rBytes.buffer)[0];

        const r = colorBytes[0];
        const g = colorBytes[1];
        const b = colorBytes[2];
        const a = colorBytes[3];
        node.color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;
        node.hasBorder = (flags & 1) === 1;
        node.isHero = (flags & 2) === 2;
        node.downed = (flags & 4) === 4;
        node.ownHero = (flags & 8) === 8;

        if (nameLength > 0) {
            const decoder = new TextDecoder("utf-8");
            node.name = decoder.decode(data.slice(idx, idx + nameLength));

            idx += nameLength;
        }

        nodes.push(node);
    }

    if (render) {
        reportFrameStart();

        renderFrame({ x: offsetX, y: offsetY }, [], nodes);
        nodes.length = 0;
    }
}

function handleLeaderboardAdd(data) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    const areaOrderBytes = data.slice(8, 10);
    const areaOrder = new Uint16Array(areaOrderBytes.buffer)[0];

    const downed = data[10] === 1;

    const playerNameLength = data[11];
    const areaNameLength = data[12];
    const mapIdLength = data[13];

    let idx = 14;

    const decoder = new TextDecoder("utf-8");

    const playerName = decoder.decode(data.slice(idx, idx + playerNameLength));
    idx += playerNameLength;

    const areaName = decoder.decode(data.slice(idx, idx + areaNameLength));
    idx += areaNameLength;

    const mapId = decoder.decode(data.slice(idx, idx + mapIdLength));

    leaderboard.add(playerId, playerName, areaOrder, areaName, mapId, downed);
}

function handleLeaderboardRemove(data) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    leaderboard.remove(playerId);
}

function handleLeaderboardTransfer(data) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    const areaOrderBytes = data.slice(8, 10);
    const areaOrder = new Uint16Array(areaOrderBytes.buffer)[0];

    const areaNameLength = data[10];
    const mapIdLength = data[11];

    let idx = 12;

    const decoder = new TextDecoder("utf-8");

    const areaName = decoder.decode(data.slice(idx, idx + areaNameLength));
    idx += areaNameLength;

    const mapId = decoder.decode(data.slice(idx, idx + mapIdLength));

    leaderboard.transfer(playerId, areaOrder, areaName, mapId);
}

function handleLeaderboardSetDowned(data) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    const downed = data[8] === 1;

    leaderboard.setDowned(playerId, downed);
}

function handleLeaderboardStateUpdate(data) {
    const entryCount = data[0];

    let idx = 1;

    for (let i = 0; i < entryCount; i++) {
        const playerIdBytes = data.slice(idx, idx + 8);
        const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

        const areaOrderBytes = data.slice(idx + 8, idx + 10);
        const areaOrder = new Uint16Array(areaOrderBytes.buffer)[0];

        const downed = data[idx + 10] === 1;

        const playerNameLength = data[idx + 11];
        const areaNameLength = data[idx + 12];
        const mapIdLength = data[idx + 13];

        idx += 14;

        const decoder = new TextDecoder("utf-8");

        const playerName = decoder.decode(data.slice(idx, idx + playerNameLength));
        idx += playerNameLength;

        const areaName = decoder.decode(data.slice(idx, idx + areaNameLength));
        idx += areaNameLength;

        const mapId = decoder.decode(data.slice(idx, idx + mapIdLength));
        idx += mapIdLength;

        leaderboard.add(playerId, playerName, areaOrder, areaName, mapId, downed);
    }
}

function handleChatBroadcast(data) {
    const decoder = new TextDecoder("utf-8");

    const messageType = data[0];
    const senderId = new BigUint64Array(data.slice(1, 9).buffer)[0];

    const nameLength = data[9];
    const messageLength = data[10];

    let idx = 11;

    const name = decoder.decode(data.slice(idx, idx + nameLength));
    idx += nameLength;

    const message = decoder.decode(data.slice(idx, idx + messageLength));

    chat.receiveMessage(message, senderId, name, messageType);
}
import { setup_canvas as setup_canvas, renderArea, render_frame } from "./rendering.js";
import { setup_input as setup_input } from "./input.js";
import { connect, establish_uni_connection as establish_uni_connection, establishInputConnection as establish_input_connection, establish_render_connection as establish_render_connection } from "./networking.js";
import { report_bandwidth, reportFrameStart as report_frame_start } from "./metrics.js";
import { leaderboard } from "./leaderboard.js";
import { chat } from "./chat.js";
import { Portal, Rect, RenderNode } from "./types.js";

const game_container = document.querySelector("#game-container") as HTMLDivElement;
const connection_panel = document.querySelector("#connection-panel") as HTMLDivElement;
const connect_button = document.querySelector("#connect-button") as HTMLButtonElement;
const area_name_heading = document.querySelector("#area-name") as HTMLHeadingElement;

type MapData = {
    id: string,
    name: string,
    text_color: string,
}

export type CommandData = {
    name: string,
    description: string,
    usage: string,
    aliases: string[],
}

type PlayerData = {
    player_id: bigint,
    map_id: string,
};

type Cache = {
    maps: MapData[],
    commands: CommandData[],
    current_players: PlayerData[],
}

export const cache: Cache = {
    maps: [],
    commands: [],
    current_players: [],
};

async function main() {
    await init_cache();

    connect_button.disabled = false;
    connect_button.onclick = handle_connection;
}
window.onload = main;

async function handle_connection() {
    const name_input = document.querySelector("#name-input") as HTMLInputElement;
    const name = name_input.value.trim();

    console.log("Connecting...");

    if (name.length === 0) {
        console.log("Name empty");
        return;
    }

    connect_button.disabled = true;

    await connect(name);
    establish_input_connection();
    establish_render_connection(handle_render_update);
    establish_uni_connection([
        {
            header: "ADEF",
            callback: handle_area_update,
        },
        {
            header: "LBAD",
            callback: handle_leaderboard_add,
        },
        {
            header: "LBRM",
            callback: handle_leaderboard_remove,
        },
        {
            header: "LBTR",
            callback: handle_leaderboard_transfer,
        },
        {
            header: "LBSD",
            callback: handleLeaderboardSetDowned,
        },
        {
            header: "LBST",
            callback: handle_leaderboard_state_update,
        },
        {
            header: "CHBR",
            callback: handle_chat_broadcast
        },
    ]);
    setup_input();

    game_container.classList.remove("hidden");
    connection_panel.classList.add("hidden");

    setup_canvas();
}

async function init_cache() {
    const json = await fetch("/cache").then((response) => response.json());

    cache.maps = json["maps"];
    cache.commands = json["commands"];

    console.log("Cache loaded");
    console.log(cache);
}

function handle_area_update(data: Uint8Array) {
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

    const walls: Rect[] = [];
    const safeZones: Rect[] = [];
    const portals: Portal[] = [];

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

    if (!map) {
        console.error(`Map '${mapId}' not found in cache`);
        return;
    }

    const name = `${map.name} - ${areaName}`;

    area_name_heading.innerHTML = name;
    area_name_heading.style.color = map.text_color;

    renderArea(width, height, color, walls, safeZones, portals);
}

const nodes: RenderNode[] = [];

function handle_render_update(data: Uint8Array) {
    report_bandwidth(data.length);

    const offsetXBytes = data.slice(0, 4);
    const offsetYBytes = data.slice(4, 8);

    const offsetX = new Float32Array(offsetXBytes.buffer)[0];
    const offsetY = new Float32Array(offsetYBytes.buffer)[0];

    const render = data[8] === 1;

    const lengthBytes = data.slice(9, 13);
    const length = new Uint32Array(lengthBytes.buffer)[0];

    let idx = 13;

    for (let i = 0; i < length; i++) {
        const x_bytes = data.slice(idx, idx + 4);
        const y_bytes = data.slice(idx + 4, idx + 8);
        const r_bytes = data.slice(idx + 8, idx + 12);
        const color_bytes = data.slice(idx + 12, idx + 16);
        const flags = data[idx + 16];
        const name_length = data[idx + 17];

        idx += 18;

        const x = new Float32Array(x_bytes.buffer)[0];
        const y = new Float32Array(y_bytes.buffer)[0];
        const radius = new Float32Array(r_bytes.buffer)[0];

        const r = color_bytes[0];
        const g = color_bytes[1];
        const b = color_bytes[2];
        const a = color_bytes[3];
        const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;
        const has_outline = (flags & 1) === 1;
        const is_hero = (flags & 2) === 2;
        const downed = (flags & 4) === 4;
        const own_hero = (flags & 8) === 8;

        const node: RenderNode = {
            x,
            y,
            radius,
            color,
            has_outline,
            is_hero,
            downed,
            own_hero,
        };

        if (name_length > 0) {
            const decoder = new TextDecoder("utf-8");
            node.name = decoder.decode(data.slice(idx, idx + name_length));

            idx += name_length;
        }

        nodes.push(node);
    }

    if (render) {
        report_frame_start();

        render_frame({ x: offsetX, y: offsetY }, nodes);
        nodes.length = 0;
    }
}

function handle_leaderboard_add(data: Uint8Array) {
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

function handle_leaderboard_remove(data: Uint8Array) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    leaderboard.remove(playerId);
}

function handle_leaderboard_transfer(data: Uint8Array) {
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

function handleLeaderboardSetDowned(data: Uint8Array) {
    const playerIdBytes = data.slice(0, 8);
    const playerId = new BigUint64Array(playerIdBytes.buffer)[0];

    const downed = data[8] === 1;

    leaderboard.set_downed(playerId, downed);
}

function handle_leaderboard_state_update(data: Uint8Array) {
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

function handle_chat_broadcast(data: Uint8Array) {
    const decoder = new TextDecoder("utf-8");

    const messageType = data[0];
    const senderId = new BigUint64Array(data.slice(1, 9).buffer)[0];

    const nameLength = data[9];
    const messageLength = data[10];

    let idx = 11;

    const name = decoder.decode(data.slice(idx, idx + nameLength));
    idx += nameLength;

    const message = decoder.decode(data.slice(idx, idx + messageLength));

    chat.receive_message(message, senderId, name, messageType);
}
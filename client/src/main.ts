import { setup_canvas as setup_canvas, renderArea as render_area, render_frame } from "./rendering.js";
import { setup_input as setup_input } from "./input.js";
import { connect, establish_uni_connection as establish_uni_connection, establishInputConnection as establish_input_connection, establish_render_connection as establish_render_connection } from "./networking.js";
import { report_bandwidth, reportFrameStart as report_frame_start } from "./metrics.js";
import { leaderboard } from "./leaderboard.js";
import { chat, MessageType } from "./chat.js";
import { Portal, Rect, RenderNode } from "./types.js";
import { BinaryParser } from "./binary_parser.js";

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
            callback: handle_leaderboard_set_downed,
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
    const parser = new BinaryParser(data);

    const width = parser.read_f32();
    const height = parser.read_f32();

    const [r, g, b, a] = parser.read_rgba();

    const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

    const walls_length = parser.read_u16();
    const safe_zones_length = parser.read_u16();
    const portals_length = parser.read_u16();

    const walls: Rect[] = [];
    const safe_zones: Rect[] = [];
    const portals: Portal[] = [];

    for (let i = 0; i < walls_length; i++) {
        const rect = parser.read_rect();
        walls.push(rect);
    }

    for (let i = 0; i < safe_zones_length; i++) {
        const rect = parser.read_rect();
        safe_zones.push(rect);
    }

    for (let i = 0; i < portals_length; i++) {
        const x = parser.read_f32();
        const y = parser.read_f32();
        const w = parser.read_f32();
        const h = parser.read_f32();

        const [r, g, b, a] = parser.read_rgba();

        const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

        portals.push({ x, y, w, h, color });
    }

    const area_name = parser.read_length_u8_string();
    const map_id = parser.read_length_u8_string();

    const map = cache.maps.find(m => m.id === map_id);

    if (!map) {
        console.error(`Map '${map_id}' not found in cache`);
        return;
    }

    const name = `${map.name} - ${area_name}`;

    area_name_heading.innerHTML = name;
    area_name_heading.style.color = map.text_color;

    render_area(width, height, color, walls, safe_zones, portals);
}

const nodes: RenderNode[] = [];

function handle_render_update(data: Uint8Array) {
    report_bandwidth(data.length);
    const parser = new BinaryParser(data);

    const offset = parser.read_vector2();

    const [render] = parser.read_flags();

    const node_count = parser.read_u32();

    for (let i = 0; i < node_count; i++) {
        const x = parser.read_f32();
        const y = parser.read_f32();
        const radius = parser.read_f32();

        const [r, g, b, a] = parser.read_rgba();
        const color = `rgba(${r}, ${g}, ${b}, ${a / 255})`;

        const [has_outline, is_hero, downed, own_hero] = parser.read_flags();

        const name = parser.read_length_u8_string();

        const node: RenderNode = {
            x,
            y,
            radius,
            color,
            has_outline,
            is_hero,
            downed,
            own_hero,
            name
        };

        nodes.push(node);
    }

    if (render) {
        report_frame_start();

        render_frame(offset, nodes);
        nodes.length = 0;
    }
}

function handle_leaderboard_add(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const player_id = parser.read_u64();
    const area_order = parser.read_u16();
    const [downed] = parser.read_flags();

    const player_name = parser.read_length_u8_string()!;
    const area_name = parser.read_length_u8_string()!;
    const map_id = parser.read_length_u8_string()!;

    leaderboard.add(player_id, player_name, area_order, area_name, map_id, downed);
}

function handle_leaderboard_remove(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const player_id = parser.read_u64();
    leaderboard.remove(player_id);
}

function handle_leaderboard_transfer(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const player_id = parser.read_u64();
    const area_order = parser.read_u16();

    const area_name = parser.read_length_u8_string()!;
    const map_id = parser.read_length_u8_string()!;

    leaderboard.transfer(player_id, area_order, area_name, map_id);
}

function handle_leaderboard_set_downed(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const player_id = parser.read_u64();
    const [downed] = parser.read_flags();

    leaderboard.set_downed(player_id, downed);
}

function handle_leaderboard_state_update(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const entry_count = parser.read_u8();

    for (let i = 0; i < entry_count; i++) {
        const player_id = parser.read_u64();
        const area_order = parser.read_u16();
        const [downed] = parser.read_flags();

        const player_name = parser.read_length_u8_string()!;
        const area_name = parser.read_length_u8_string()!;
        const map_id = parser.read_length_u8_string()!;

        leaderboard.add(player_id, player_name, area_order, area_name, map_id, downed);
    }
}

function handle_chat_broadcast(data: Uint8Array) {
    const parser = new BinaryParser(data);

    const message_type = parser.read_u8() as MessageType;
    const sender_id = parser.read_u64();

    const name = parser.read_length_u8_string()!;
    const message = parser.read_length_u8_string()!;

    chat.receive_message(message, sender_id, name, message_type);
}
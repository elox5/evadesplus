import { BinaryStream } from "./binary_stream.js";
import { chat, chat_settings } from "./chat.js";
import { try_execute_command } from "./commands.js";
import { input, input_settings, lock_mouse_input } from "./input.js";
import { leaderboard } from "./leaderboard.js";
import { cache, return_to_menu } from "./main.js";
import { metric_settings, start_ping, report_ping } from "./metrics.js";
import { Vector2 } from "./types.js";

export let transport: WebTransport;

export let networkSettings = {
    input_update_rate: 1000 / 60,
}

type NetworkData = {
    ping_interval: number | undefined,
    input_interval: number | undefined
}

let data: NetworkData = {
    ping_interval: undefined,
    input_interval: undefined
}

export async function connect(name: string) {
    const url = window.location.origin;
    let certificate = await get_certificate();

    transport = new WebTransport(url, {
        serverCertificateHashes: [
            {
                algorithm: "sha-256",
                value: certificate.buffer,
            }
        ]
    });

    console.log(`Establishing WebTransport connection at ${url}`);

    await transport.ready;

    console.log("Connected");

    window.onpagehide = () => {
        close_webtransport_connection();
    }

    const encoder = new TextEncoder();

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();
    await writer.write(encoder.encode(`NAME${name}`));
    await writer.close();

    initialize_ping_meter();

    console.log("Sent player data");
}

async function close_webtransport_connection() {
    transport.close({ closeCode: 1000, reason: "ClientDisconnected" });

    await transport.closed;

    if (data.ping_interval !== undefined) {
        clearInterval(data.ping_interval);
    }
    if (data.input_interval !== undefined) {
        clearInterval(data.input_interval);
    }

    console.log("Closed WebTransport connection");
}

export async function disconnect() {
    await close_webtransport_connection();

    cache.current_players = [];
    leaderboard.clear();
    lock_mouse_input();
    chat.clear();

    return_to_menu();
}

async function get_certificate() {
    const response = await fetch("/cert");
    let digest = await response.json();
    digest = JSON.parse(digest);

    return new Uint8Array(digest);
}


let lastInput: Vector2 = {
    x: 0,
    y: 0,
}

async function initialize_ping_meter() {
    data.ping_interval = setInterval(async () => {
        start_ping();
        const stream = await transport.createBidirectionalStream();

        const readable = stream.readable;
        const writer = stream.writable.getWriter();

        await writer.write(new TextEncoder().encode("ping"));
        await writer.close();

        await read_stream(readable, [
            {
                header: "pong",
                callback: report_ping
            }
        ]);
    }, metric_settings.ping_frequency);
}

export async function send_chat_message(msg: string) {
    if (chat_settings.auto_reply && chat.reply_target !== undefined && cache.current_players.some(p => p.player_id === chat.reply_target) && !msg.startsWith("/")) {
        msg = `/reply ${msg}`;
    }

    if (msg.startsWith("/")) {
        let { executed, message } = try_execute_command(msg);

        if (executed) return;

        msg = message;
    }

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();

    const encoder = new TextEncoder();
    await writer.write(encoder.encode(`CHAT${msg}`));

    await writer.close();
}

async function send_input(writer: WritableStreamDefaultWriter, input: Vector2) {
    if (lastInput.x === input.x && lastInput.y === input.y) {
        return;
    }

    const inputArray = new Float32Array([input.x, input.y]);
    const data = new Uint8Array(inputArray.buffer);

    lastInput.x = input.x;
    lastInput.y = input.y;

    await writer.write(data);
}

export function establish_input_connection() {
    const stream = transport.datagrams.writable;
    const writer = stream.getWriter();

    data.input_interval = setInterval(async () => {
        send_input(writer, input);
    }, networkSettings.input_update_rate);
}

type StreamCallback = {
    header: string,
    callback: (data: BinaryStream) => void
}

export async function establish_render_connection(callback: (stream: BinaryStream) => void) {
    const reader = transport.datagrams.readable.getReader();
    while (true) {
        const { value, done } = await reader.read();

        if (done) {
            break;
        }

        const stream = new BinaryStream(value);

        callback(stream);
    }
}

export async function establish_uni_connection(callbacks: StreamCallback[]) {
    const reader = transport.incomingUnidirectionalStreams.getReader();
    while (true) {
        const { value, done } = await reader.read();

        if (done) {
            break;
        }

        read_stream(value, callbacks);
    }
}

async function read_stream(stream: ReadableStream, callbacks: StreamCallback[]) {
    const reader = stream.getReader();

    while (true) {
        const { value, done } = await reader.read();
        const data = value;

        if (done) {
            break;
        }

        const stream = new BinaryStream(data);
        const header = stream.read_string(4);

        for (const callback of callbacks) {
            if (callback.header === header) {
                callback.callback(stream);
            }
        }
    }
}
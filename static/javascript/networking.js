import { tryExecuteCommand } from "./commands.js";
import { input } from "./input.js";
import { metricSettings, startPing, reportPing } from "./metrics.js";

export let transport;

export let networkSettings = {
    inputUpdateRate: 1000 / 60,
}

export async function connect(name) {
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
        console.log("Closing WebTransport connection");
        transport.close();
    }

    const encoder = new TextEncoder();

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();
    await writer.write(encoder.encode(`NAME${name}`));
    await writer.close();

    initializePingMeter();

    console.log("Sent player data");
}

async function get_certificate() {
    const response = await fetch("/cert");
    let digest = await response.json();
    digest = JSON.parse(digest);

    return new Uint8Array(digest);
}


let lastInput = {
    x: 0,
    y: 0
}

async function initializePingMeter() {
    setInterval(async () => {
        startPing();
        const stream = await transport.createBidirectionalStream();

        const readable = stream.readable;
        const writer = stream.writable.getWriter();

        await writer.write(new TextEncoder().encode("ping"));
        await writer.close();

        await readStream(readable, [
            {
                header: "pong",
                callback: (data) => {
                    reportPing(data);
                }
            }
        ]);
    }, metricSettings.pingFrequency);
}

export async function sendChatMessage(message) {
    if (message.startsWith("/")) {
        let executed = tryExecuteCommand(message);

        if (executed) return;
    }

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();

    const encoder = new TextEncoder();
    await writer.write(encoder.encode(`CHAT${message}`));

    await writer.close();
}

async function sendInput(writer, input) {
    if (lastInput.x === input.x && lastInput.y === input.y) {
        return;
    }

    const inputArray = new Float32Array([input.x, input.y]);
    const data = new Uint8Array(inputArray.buffer);

    lastInput.x = input.x;
    lastInput.y = input.y;

    await writer.write(data);
}

export function establishInputConnection() {
    const stream = transport.datagrams.writable;
    const writer = stream.getWriter();

    setInterval(async () => {
        sendInput(writer, input);
    }, networkSettings.inputUpdateRate);
}

export async function establishRenderConnection(callback) {
    const reader = transport.datagrams.readable.getReader();
    while (true) {
        const { value, done } = await reader.read();

        if (done) {
            break;
        }

        callback(value);
    }
}

export async function establishUniConnection(callbacks) {
    const reader = transport.incomingUnidirectionalStreams.getReader();
    while (true) {
        const { value, done } = await reader.read();

        if (done) {
            break;
        }

        readStream(value, callbacks);
    }
}

async function readStream(stream, callbacks) {
    const reader = stream.getReader();
    while (true) {
        const { value, done } = await reader.read();
        const data = value;

        if (done) {
            break;
        }

        const header = new TextDecoder().decode(data.slice(0, 4));

        for (const callback of callbacks) {
            if (callback.header === header) {
                callback.callback(data.slice(4));
            }
        }
    }
}
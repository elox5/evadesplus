import { input } from "./input.js";

const url = "https://localhost:3333";

export let transport;

export let networkSettings = {
    inputUpdateRate: 1000 / 60,
}

export async function connect() {
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

    window.onbeforeunload = () => {
        transport.close();
    }
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
import { BinaryReader } from "./binary_reader.js";

export class WsConnector {
    private ws: WebSocket | null = null;

    private connected(): boolean {
        return this.ws !== null;
    }

    async init() {
        if (this.ws !== null) return;

        console.log("Establishing WebSocket connection...");

        const ws = new WebSocket("ws://localhost:3335");
        ws.binaryType = "arraybuffer";

        ws.onopen = () => {
            console.log(`Established WebSocket connection on ${ws?.url}`);
        }

        ws.onmessage = (message) => {
            const data = new BinaryReader(message.data);

            console.log("Message from server: ", data.read_string(5));
        }

        ws.onclose = () => {
            console.log("WebSocket connection closed");
        }

        ws.onerror = (error) => {
            console.error("WebSocket error:", error);
        }

        this.ws = ws;
    }

    ready() {
        return new Promise<void>((resolve, reject) => {
            if (this.ws?.readyState == WebSocket.OPEN) return resolve();
            this.ws?.addEventListener('open', () => resolve(), { once: true });
            this.ws?.addEventListener('error', reject, { once: true });
        })
    }

    async send(header: string, data: Uint8Array) {
        if (!this.connected()) return;

        const encoder = new TextEncoder();
        const headerBytes = encoder.encode(header.substring(0, 4));

        const msg = new Uint8Array(4 + data.length);
        msg.set(headerBytes, 0);
        msg.set(data, 4);

        this.ws?.send(msg);
    }
}
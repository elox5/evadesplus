import { BinaryReader } from "./binary_reader.js";

export class WsConnector {
    private ws: WebSocket | null = null;

    private handlers: MessageHandler[] = [];

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

            this.handle_message(data);
        }

        ws.onclose = () => {
            console.log("WebSocket connection closed");
        }

        ws.onerror = (error) => {
            console.error("WebSocket error:", error);
        }

        this.ws = ws;
    }

    register_handler(handler: MessageHandler) {
        this.handlers.push(handler);
    }

    async handle_message(message: BinaryReader) {
        const header = message.read_string(4);

        for (const handler of this.handlers) {
            if (handler.header === header) {
                handler.callback(message);
            }
        }
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

export const ws_connector = new WsConnector();

export type MessageHandler = {
    header: string,
    callback: (data: BinaryReader) => void
}
import { BinaryReader } from "./binary_reader.js";

export class WsConnector {
    private ws: WebSocket | null = null;

    private handlers: MessageHandler[] = [];
    private modules: WsModule[] = [];

    private game_load_handled: boolean = false;

    connected(): boolean {
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

        this.run_setup_callbacks();

        this.ws = ws;
    }

    async close() {
        if (!this.connected()) return;

        this.ws?.close();

        await this.wait_for_close();
    }

    register_handler(handler: MessageHandler) {
        this.handlers.push(handler);
    }

    register_module(module: WsModule) {
        this.modules.push(module);
        if (module.handlers !== undefined) {
            for (const handler of module.handlers) {
                this.handlers.push(handler);
            }
        }
    }

    run_setup_callbacks() {
        for (const mod of this.modules) {
            if (mod.setup !== undefined) {
                mod.setup();
            }
        }
    }

    run_game_load_callbacks() {
        for (const mod of this.modules) {
            if (mod.on_game_load !== undefined) {
                if (this.game_load_handled && mod.on_game_load.once) continue;

                mod.on_game_load.callback();
            }
        }

        this.game_load_handled = true;
    }

    run_cleanup_callbacks() {
        for (const mod of this.modules) {
            if (mod.cleanup !== undefined) {
                mod.cleanup();
            }
        }
    }

    async handle_message(message: BinaryReader) {
        const header = message.read_string(4);

        for (const handler of this.handlers) {
            if (handler.header === header) {
                handler.callback(message.clone());
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

    private wait_for_close() {
        return new Promise<void>((resolve, _) => {
            if (this.ws?.readyState == WebSocket.CLOSED) return resolve();
            this.ws?.addEventListener('close', () => resolve(), { once: true });
            this.ws?.addEventListener('close', () => resolve(), { once: true });
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

export class WsModule {
    handlers?: MessageHandler[] = [];

    setup?: () => void;
    cleanup?: () => void;
    on_game_load?: {
        callback: () => void;
        once: boolean,
    };
}
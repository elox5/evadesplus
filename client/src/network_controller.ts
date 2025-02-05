import { BinaryReader } from "./binary_reader.js";

export class NetworkController {
    private transport: WebTransport;
    private connected = false;

    private modules: NetworkModule[] = [];

    private uni_handlers: StreamHandler[] = [];
    private datagram_handlers: StreamHandler[] = [];

    is_connected(): boolean {
        return this.connected;
    }

    private async get_certificate() {
        const response = await fetch("/cert");
        let digest = await response.json();
        digest = JSON.parse(digest);

        return new Uint8Array(digest);
    }

    private async send_init_packet(name: string) {
        const encoder = new TextEncoder();

        const stream = await this.transport.createUnidirectionalStream();
        const writer = stream.getWriter();
        await writer.write(encoder.encode(`NAME${name}`));
        await writer.close();

        console.log("Sent init packet");
    }

    async connect(name: string) {
        if (this.connected) {
            console.warn("WebTransport connection already established");
            return;
        }

        const url = window.location.origin;
        let certificate = await this.get_certificate();

        this.transport = new WebTransport(url, {
            serverCertificateHashes: [
                {
                    algorithm: "sha-256",
                    value: certificate.buffer,
                }
            ]
        });

        console.log(`Establishing WebTransport connection at ${url}...`);

        await this.transport.ready;

        this.connected = true;

        console.log("Connected. Registering modules...");

        for (const module of this.modules) {
            module.register(this);
        }

        console.log("Modules registered.");

        this.send_init_packet(name);

        this.init_uni_handler();
        this.init_datagram_handler();

        window.onpagehide = () => {
            this.close_webtransport_connection();
        }
    }

    async disconnect() {
        await this.close_webtransport_connection();
        this.connected = false;
    }

    register_module(module: NetworkModule) {
        this.modules.push(module);
    }

    run_module_pre_register() {
        for (const module of this.modules) {
            if (module.pre_register !== undefined) module.pre_register();
        }
    }

    async register_uni_handler(header: string, callback: (data: BinaryReader) => void) {
        const handler = { header, callback };
        this.uni_handlers.push(handler);
    }

    async register_datagram_handler(header: string, callback: (data: BinaryReader) => void) {
        const handler = { header, callback };
        this.datagram_handlers.push(handler);
    }

    private async init_datagram_handler() {
        if (!this.connected) return;

        const reader = this.transport.datagrams.readable.getReader();

        while (true) {
            const { value, done } = await reader.read();
            const data = value as Uint8Array;

            if (done) {
                break;
            }

            const stream = new BinaryReader(data.buffer);

            for (const handler of this.datagram_handlers) {
                handler.callback(stream);
            }
        }
    }

    private async init_uni_handler() {
        if (!this.connected) return;

        const reader = this.transport.incomingUnidirectionalStreams.getReader();
        while (true) {
            const { value, done } = await reader.read();

            if (done) {
                break;
            }

            this.read_stream(value, this.uni_handlers);
        }
    }

    private async read_stream(stream: ReadableStream, callbacks: StreamHandler[]) {
        const reader = stream.getReader();

        while (true) {
            const { value, done } = await reader.read();
            const data = value as Uint8Array;

            if (done) {
                break;
            }

            const stream = new BinaryReader(data.buffer);
            const header = stream.read_string(4);

            for (const callback of callbacks) {
                if (callback.header === header) {
                    callback.callback(stream);
                }
            }
        }
    }

    private async close_webtransport_connection() {
        if (!this.connected) return;

        this.transport.close({ closeCode: 1000, reason: "ClientDisconnected" });

        await this.transport.closed;

        for (const module of this.modules) {
            if (module.cleanup !== undefined) module.cleanup();
        }

        this.modules = [];
        this.uni_handlers = [];
        this.datagram_handlers = [];

        console.log("Closed WebTransport connection");
    }

    create_datagram_writer(): WritableStreamDefaultWriter | null {
        if (!this.connected) return null;

        const stream = this.transport.datagrams.writable;
        const writer = stream.getWriter();

        return writer;
    }

    async create_uni_writer(): Promise<WritableStreamDefaultWriter | null> {
        if (!this.connected) return null;

        const stream = await this.transport.createUnidirectionalStream();

        return stream.getWriter();
    }

    async create_bi_stream(): Promise<WebTransportBidirectionalStream> {
        return this.transport.createBidirectionalStream();
    }
}

type StreamHandler = {
    header: string,
    callback: (data: BinaryReader) => void
}

export class NetworkModule {
    pre_register?: () => void;
    register: (controller: NetworkController) => void;
    cleanup?: () => void;
}

export const network_controller = new NetworkController();
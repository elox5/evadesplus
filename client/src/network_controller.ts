import { BinaryReader } from "./binary_reader.js";
import { report_bandwidth } from "./metrics.js";

export class NetworkController {
    private transport: WebTransport;
    private closed = true;

    private modules: NetworkModule[] = [];
    private setup_handled = false;
    private game_load_handled = false;

    private uni_handlers: StreamHandler[] = [];
    private datagram_handlers: StreamHandler[] = [];

    is_closed(): boolean {
        return this.closed;
    }

    private async get_certificate() {
        const response = await fetch("/cert");
        let digest = await response.json();
        digest = JSON.parse(digest);

        return new Uint8Array(digest);
    }

    private async init(name: string): Promise<bigint | "name_invalid" | InitError> {
        const encoder = new TextEncoder();

        const stream = await this.transport.createBidirectionalStream();
        const writer = stream.writable.getWriter();
        await writer.write(encoder.encode(`INIT${name}`));
        await writer.close();

        const reader = stream.readable.getReader();
        const { value } = await reader.read();

        const response = value as Uint8Array;
        const data = new BinaryReader(response.buffer);

        const init_response = data.read_u8() as InitResponse;

        if (init_response === InitResponse.InvalidName) {
            return "name_invalid";
        }

        if (init_response === InitResponse.Error) {
            const message = data.read_length_u8_string()!;
            const error: InitError = { message };

            return error;
        }

        const id = data.read_u64();

        this.run_init(data);

        return id;
    }

    async connect(name: string): Promise<bigint | "name_invalid" | "already_connected" | InitError> {
        if (!this.is_closed()) {
            console.warn("WebTransport connection already established");
            return "already_connected";
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

        this.closed = false;

        this.transport.closed.then(() => {
            this.closed = true;
        })

        console.log(`Establishing WebTransport connection at ${url}...`);

        await this.transport.ready;

        console.log("Connected.");

        this.run_setup();

        this.init_uni_handler();
        this.init_datagram_handler();

        window.onpagehide = () => {
            this.close_webtransport_connection();
        }

        const init_response = await this.init(name);

        if (typeof init_response !== "bigint") {
            this.disconnect();
        }

        return init_response;
    }

    async disconnect() {
        await this.close_webtransport_connection();
    }

    register_module(module: NetworkModule) {
        this.modules.push(module);

        if (module.uni_handlers !== undefined) {
            this.uni_handlers.push(...module.uni_handlers)
        };

        if (module.datagram_handlers !== undefined) {
            this.datagram_handlers.push(...module.datagram_handlers)
        };
    }

    private async init_datagram_handler() {
        if (this.is_closed()) return;

        const reader = this.transport.datagrams.readable.getReader();

        while (true) {
            const { value, done } = await reader.read();
            const data = value as Uint8Array;

            if (done || this.is_closed()) {
                break;
            }

            report_bandwidth(data.byteLength);

            const stream = new BinaryReader(data.buffer);

            for (const handler of this.datagram_handlers) {
                handler.callback(stream);
            }
        }
    }

    private async init_uni_handler() {
        if (this.is_closed()) return;

        const reader = this.transport.incomingUnidirectionalStreams.getReader();
        while (true) {
            const { value, done } = await reader.read();

            if (done || this.is_closed()) {
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

            if (done || this.is_closed()) {
                break;
            }

            report_bandwidth(data.byteLength);

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
        if (this.is_closed()) return;

        this.transport.close({ closeCode: 1000, reason: "ClientDisconnected" });

        await this.transport.closed;

        this.run_cleanup();

        console.log("Closed WebTransport connection");
    }

    create_datagram_writer(): WritableStreamDefaultWriter | null {
        if (this.is_closed()) return null;

        const stream = this.transport.datagrams.writable;
        const writer = stream.getWriter();

        return writer;
    }

    async create_uni_writer(): Promise<WritableStreamDefaultWriter | null> {
        if (this.is_closed()) return null;

        const stream = await this.transport.createUnidirectionalStream();

        return stream.getWriter();
    }

    async create_bi_stream(): Promise<WebTransportBidirectionalStream> {
        return this.transport.createBidirectionalStream();
    }

    private run_setup() {
        for (const module of this.modules) {
            if (module.setup !== undefined) {
                if (this.setup_handled && module.setup.once) continue;

                module.setup.callback(this);
            }
        }

        this.setup_handled = true;
    }

    run_game_load_callbacks() {
        for (const module of this.modules) {
            if (module.on_game_load !== undefined) {
                if (this.game_load_handled && module.on_game_load.once) continue;

                module.on_game_load.callback();
            }
        }

        this.game_load_handled = true;
    }

    private run_init(data: BinaryReader) {
        const init_modules = this.modules
            .filter(module => module.init !== undefined)
            .sort((a, b) => a.init!.order - b.init!.order);

        for (const module of init_modules) {
            module.init!.callback(data);
        }
    }

    private run_cleanup() {
        for (const module of this.modules) {
            if (module.cleanup !== undefined) {
                module.cleanup();
            }
        }
    }
}

enum InitResponse {
    Ok = 0,
    InvalidName = 1,
    Error = 2,
}

export type InitError = {
    message: string
}

type StreamHandler = {
    header: string,
    callback: (data: BinaryReader) => void
}

export class NetworkModule {
    uni_handlers?: StreamHandler[] = [];
    datagram_handlers?: StreamHandler[] = [];

    setup?: {
        callback: (data: NetworkController) => void;
        once: boolean,
    };
    on_game_load?: {
        callback: () => void;
        once: boolean,
    };
    init?: {
        callback: (data: BinaryReader) => void,
        order: number
    };
    cleanup?: () => void;
}

export const network_controller = new NetworkController();
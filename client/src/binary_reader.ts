import { Rect, Vector2 } from "./types.js";

export class BinaryReader {
    private buffer: ArrayBuffer;
    private index: number;

    private decoder: TextDecoder;

    constructor(data: ArrayBuffer) {
        this.buffer = data;
        this.index = 0;

        this.decoder = new TextDecoder("utf-8");
    }

    length(): number {
        return this.buffer.byteLength;
    }

    step(length: number) {
        this.index += length;
    }

    read_u8(): number {
        const value = new Uint8Array(this.buffer.slice(this.index, this.index + 1))[0];
        this.index += 1;
        return value;
    }

    read_u16(): number {
        const value = new Uint16Array(this.buffer.slice(this.index, this.index + 2))[0];
        this.index += 2;
        return value;
    }

    read_u32(): number {
        const value = new Uint32Array(this.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_u64(): bigint {
        const value = new BigInt64Array(this.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_i8(): number {
        const value = new Int8Array(this.buffer.slice(this.index, this.index + 1))[0];
        this.index += 1;
        return value;
    }

    read_i16(): number {
        const value = new Int16Array(this.buffer.slice(this.index, this.index + 2))[0];
        this.index += 2;
        return value;
    }

    read_i32(): number {
        const value = new Int32Array(this.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_i64(): bigint {
        const value = new BigInt64Array(this.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_f32(): number {
        const value = new Float32Array(this.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_f64(): number {
        const value = new Float64Array(this.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_bytes(count: number): ArrayBuffer {
        const bytes = this.buffer.slice(this.index, this.index + count);
        this.index += count;
        return bytes;
    }

    read_bool(): boolean {
        const value = new Uint8Array(this.buffer.slice(this.index, this.index + 1))[0];
        this.index += 1;
        return value !== 0;
    }

    read_flags(): boolean[] {
        const byte = this.read_u8();
        const flags = [];
        for (let i = 0; i < 8; i++) {
            flags.push(((byte >> i) & 1) === 1);
        }
        return flags;
    }

    read_rgba(): [number, number, number, number] {
        const bytes = this.read_bytes(4);
        const [r, g, b, a] = new Uint8Array(bytes);

        return [r, g, b, a];
    }

    read_vector2(): Vector2 {
        const bytes = this.read_bytes(8);
        const [x, y] = new Float32Array(bytes);

        return { x, y };
    }

    read_rect(): Rect {
        const bytes = this.read_bytes(16);
        const [x, y, w, h] = new Float32Array(bytes);

        return { x, y, w, h };
    }

    read_string(length_bytes: number): string {
        const bytes = this.read_bytes(length_bytes);
        return this.decoder.decode(bytes);
    }

    read_length_u8_string(): string | null {
        const length = this.read_u8();

        if (length === 0) {
            return null;
        }

        return this.read_string(length);
    }

    read_length_u16_string(): string | null {
        const length = this.read_u16();

        if (length === 0) {
            return null;
        }

        return this.read_string(length);
    }
}
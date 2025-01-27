import { Rect, Vector2 } from "./types.js";

export class BinaryStream {
    private data: Uint8Array;
    private index: number;
    private text_decoder: TextDecoder;

    constructor(data: Uint8Array, text_decoder?: TextDecoder) {
        this.data = data;
        this.index = 0;

        this.text_decoder = text_decoder ?? new TextDecoder("utf-8");
    }

    set_text_decoder(decoder: TextDecoder) {
        this.text_decoder = decoder;
    }

    length(): number {
        return this.data.length;
    }

    read_u8(): number {
        const value = this.data[this.index];
        this.index += 1;
        return value;
    }

    read_u16(): number {
        const value = new Uint16Array(this.data.buffer.slice(this.index, this.index + 2))[0];
        this.index += 2;
        return value;
    }

    read_u32(): number {
        const value = new Uint32Array(this.data.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_u64(): bigint {
        const value = new BigUint64Array(this.data.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_i8(): number {
        const value = new Int8Array(this.data.buffer.slice(this.index, this.index + 1))[0];
        this.index += 1;
        return value;
    }

    read_i16(): number {
        const value = new Int16Array(this.data.buffer.slice(this.index, this.index + 2))[0];
        this.index += 2;
        return value;
    }

    read_i32(): number {
        const value = new Int32Array(this.data.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_i64(): bigint {
        const value = new BigInt64Array(this.data.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_f32(): number {
        const value = new Float32Array(this.data.buffer.slice(this.index, this.index + 4))[0];
        this.index += 4;
        return value;
    }

    read_f64(): number {
        const value = new Float64Array(this.data.buffer.slice(this.index, this.index + 8))[0];
        this.index += 8;
        return value;
    }

    read_bytes(count: number): Uint8Array {
        const bytes = this.data.slice(this.index, this.index + count);
        this.index += count;
        return bytes;
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
        const r = this.read_u8();
        const g = this.read_u8();
        const b = this.read_u8();
        const a = this.read_u8();
        return [r, g, b, a];
    }

    read_vector2(): Vector2 {
        const x = this.read_f32();
        const y = this.read_f32();
        return { x, y };
    }

    read_rect(): Rect {
        const x = this.read_f32();
        const y = this.read_f32();
        const w = this.read_f32();
        const h = this.read_f32();
        return { x, y, w, h };
    }

    read_string(length_bytes: number): string {
        const bytes = this.read_bytes(length_bytes);
        return this.text_decoder.decode(bytes);
    }

    read_length_u8_string(): string | undefined {
        const length = this.read_u8();

        if (length === 0) {
            return undefined;
        }

        return this.read_string(length);
    }

    read_length_u16_string(): string | undefined {
        const length = this.read_u16();

        if (length === 0) {
            return undefined;
        }

        return this.read_string(length);
    }
}
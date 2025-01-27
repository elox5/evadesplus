import { Vector2 } from "./types.js";

export default class Canvas {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    renderOffset: Vector2;
    tileSize: number;
    radiusScale: number;

    constructor(id: string, tileSize: number, radiusScale: number = 1) {
        this.canvas = document.getElementById(id) as HTMLCanvasElement;
        this.ctx = this.canvas.getContext("2d") as CanvasRenderingContext2D;
        this.update_dimensions();
        this.renderOffset = { x: 0, y: 0 };
        this.tileSize = tileSize;
        this.radiusScale = radiusScale;
    }

    clear() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }

    game_to_canvas_pos(x: number, y: number) {
        return {
            x: (x - this.renderOffset.x) * this.tileSize,
            y: this.canvas.height - (y + this.renderOffset.y) * this.tileSize,
        }
    }

    game_to_canvas(a: number) {
        return a * this.tileSize;
    }

    set_dimensions(w: number, h: number) {
        this.canvas.width = w;
        this.canvas.height = h;

        this.canvas.style.width = `${w}px`;
        this.canvas.style.height = `${h}px`;
    }

    update_dimensions() {
        this.canvas.width = this.canvas.clientWidth;
        this.canvas.height = this.canvas.clientHeight;
    }

    set_render_offset(x: number, y: number) {
        this.renderOffset = { x, y };
    }

    set_physical_offset(x: number, y: number) {
        const xOffset = -x * this.tileSize + this.canvas.width / 2;
        const yOffset = y * this.tileSize - this.canvas.height / 2;

        this.canvas.style.translate = `${xOffset}px ${yOffset}px`;
    }
}
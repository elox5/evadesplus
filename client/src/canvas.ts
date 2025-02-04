import { Vector2 } from "./types.js";

export default class Canvas {
    canvas: HTMLCanvasElement;
    ctx: CanvasRenderingContext2D;
    render_offset: Vector2;
    tile_size: number;
    radius_scale: number;

    constructor(id: string, tileSize: number, radiusScale: number = 1) {
        this.canvas = document.getElementById(id) as HTMLCanvasElement;
        this.ctx = this.canvas.getContext("2d") as CanvasRenderingContext2D;
        this.update_dimensions();
        this.render_offset = { x: 0, y: 0 };
        this.tile_size = tileSize;
        this.radius_scale = radiusScale;
    }

    clear() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }

    game_to_canvas_pos(x: number, y: number) {
        return {
            x: (x - this.render_offset.x) * this.tile_size,
            y: this.canvas.height - (y + this.render_offset.y) * this.tile_size,
        }
    }

    game_to_canvas(a: number) {
        return a * this.tile_size;
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
        this.render_offset = { x, y };
    }

    set_physical_offset(x: number, y: number) {
        const xOffset = -x * this.tile_size + this.canvas.width / 2;
        const yOffset = y * this.tile_size - this.canvas.height / 2;

        this.canvas.style.translate = `${xOffset}px ${yOffset}px`;
    }
}
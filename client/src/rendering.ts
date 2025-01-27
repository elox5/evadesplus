import { input, input_settings } from "./input.js";
import Canvas from "./canvas.js";
import { reportRenderEnd, report_render_start } from "./metrics.js";
import { Portal, Rect, RenderNode, Vector2 } from "./types.js";

export let render_settings = {
    tile_size: 40,
    minimap_tile_size: 6,
    minimap_hero_scale: 2,
}

const main_canvas = new Canvas("main-canvas", render_settings.tile_size);
const area_canvas = new Canvas("area-canvas", render_settings.tile_size);

const area_minimap = new Canvas("area-minimap", render_settings.minimap_tile_size);
const hero_minimap = new Canvas("hero-minimap", render_settings.minimap_tile_size, render_settings.minimap_hero_scale);

export function setup_canvas() {
    main_canvas.update_dimensions();
    main_canvas.clear();

    window.onresize = () => {
        main_canvas.update_dimensions();
    }
}

function set_draw_offset(x: number, y: number) {
    main_canvas.set_render_offset(x - main_canvas.canvas.width / 2 / render_settings.tile_size, -y + main_canvas.canvas.height / 2 / render_settings.tile_size);
    area_canvas.set_physical_offset(x, y);
}

type DrawSettings = {
    fill_color?: string,
    outline_color?: string,
    outline_width?: number,
    has_frame?: boolean,
}

function draw_circle(canvas: Canvas, _x: number, _y: number, _r: number, settings: DrawSettings) {
    const ctx = canvas.ctx;
    const { x, y } = canvas.game_to_canvas_pos(_x, _y);
    const r = _r * canvas.tileSize * canvas.radiusScale;

    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);

    if (settings.fill_color !== undefined) {
        ctx.fillStyle = settings.fill_color;
        ctx.fill();
    }

    if (settings.outline_color !== undefined) {
        ctx.strokeStyle = settings.outline_color;
        ctx.lineWidth = settings.outline_width ?? 1;

        ctx.stroke();
    }

    if (settings.has_frame === true) {
        ctx.strokeStyle = settings.outline_color ?? "black";
        ctx.lineWidth = settings.outline_width ?? 1;

        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x - r, y);
        ctx.moveTo(x, y + r);
        ctx.lineTo(x, y - r);
        ctx.stroke();
    }
}


function draw_rect(canvas: Canvas, _x: number, _y: number, _w: number, _h: number, settings: DrawSettings) {
    const ctx = canvas.ctx;
    const { x, y } = canvas.game_to_canvas_pos(_x, _y);

    const w = canvas.game_to_canvas(_w);
    const h = -canvas.game_to_canvas(_h);

    if (settings.fill_color !== undefined) {
        ctx.fillStyle = settings.fill_color;
        ctx.fillRect(x, y, w, h);
    }

    if (settings.outline_color !== undefined) {
        ctx.strokeStyle = settings.outline_color;
        ctx.lineWidth = settings.outline_width ?? 1;
        ctx.strokeRect(x, y, w, h);
    }

    if (settings.has_frame) {
        ctx.strokeStyle = settings.outline_color ?? "black";
        ctx.lineWidth = settings.outline_width ?? 1;

        ctx.beginPath();
        ctx.moveTo(x + w / 2, y);
        ctx.lineTo(x + w / 2, y + h);
        ctx.moveTo(x, y + h / 2);
        ctx.lineTo(x + w, y + h / 2);
        ctx.stroke();
    }
}

function drawLine(canvas: Canvas, _x1: number, _y1: number, _x2: number, _y2: number, color = "#000", width = 1) {
    const ctx = canvas.ctx;
    const { x: x1, y: y1 } = canvas.game_to_canvas_pos(_x1, _y1);
    const { x: x2, y: y2 } = canvas.game_to_canvas_pos(_x2, _y2);

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
}

function draw_text(canvas: Canvas, _x: number, _y: number, text: string, color = "#000", size = 16, modifiers = "") {
    const ctx = canvas.ctx;
    const { x, y } = canvas.game_to_canvas_pos(_x, _y);

    ctx.fillStyle = color;
    ctx.font = `${modifiers} ${size}px Questrial, Noto Color Emoji`;
    ctx.textAlign = "center";
    ctx.fillText(text, x, y);
}

function drawGrid(width: number, height: number) {
    const ctx = area_canvas.ctx;

    ctx.strokeStyle = "#00000033";
    ctx.lineWidth = 1;

    for (let i = 0; i < width; i++) {
        ctx.beginPath();
        ctx.moveTo(i * area_canvas.tileSize, 0);
        ctx.lineTo(i * area_canvas.tileSize, area_canvas.canvas.height);
        ctx.stroke();
    }

    for (let j = 0; j < height; j++) {
        ctx.beginPath();
        ctx.moveTo(0, j * area_canvas.tileSize);
        ctx.lineTo(area_canvas.canvas.width, j * area_canvas.tileSize);
        ctx.stroke();
    }
}

export function renderArea(width: number, height: number, color: string, walls: Rect[], safeZones: Rect[], portals: Portal[]) {
    area_canvas.set_dimensions(width * area_canvas.tileSize, height * area_canvas.tileSize);

    const ctx = area_canvas.ctx;

    ctx.fillStyle = color;
    ctx.fillRect(0, 0, area_canvas.canvas.width, area_canvas.canvas.height);

    drawGrid(width, height);

    for (const wall of walls) {
        draw_rect(area_canvas, wall.x, wall.y, wall.w, wall.h, {
            fill_color: "#222",
        });
    }

    for (const safeZone of safeZones) {
        draw_rect(area_canvas, safeZone.x, safeZone.y, safeZone.w, safeZone.h, {
            fill_color: "#00000022",
        });
    }

    for (const portal of portals) {
        draw_rect(area_canvas, portal.x, portal.y, portal.w, portal.h, {
            fill_color: portal.color,
        });
    }

    area_minimap.set_dimensions(width * render_settings.minimap_tile_size, height * render_settings.minimap_tile_size);
    hero_minimap.set_dimensions(width * render_settings.minimap_tile_size, height * render_settings.minimap_tile_size);

    area_minimap.ctx.drawImage(area_canvas.canvas, 0, 0, area_canvas.canvas.width, area_canvas.canvas.height, 0, 0, area_minimap.canvas.width, area_minimap.canvas.height);
}


export function render_frame(offset: Vector2, nodes: RenderNode[]) {
    report_render_start();

    main_canvas.clear();
    hero_minimap.clear();

    set_draw_offset(offset.x, offset.y);

    let named_nodes = [];
    let own_hero = null;

    for (const node of nodes) {
        if (node.name !== undefined) {
            named_nodes.push(node);
        }

        if (node.is_hero) {
            drawMinimapHero(node);

            if (node.downed) {
                draw_text(hero_minimap, node.x, node.y + 1, "!", "red", 16, "bold");
            }
        }

        if (node.own_hero) {
            own_hero = node;
            continue;
        }

        draw_circle(main_canvas, node.x, node.y, node.radius, {
            fill_color: node.color,
            outline_color: node.has_outline ? "black" : undefined,
            outline_width: 2
        });
    }

    if (own_hero !== null) {
        draw_circle(main_canvas, own_hero.x, own_hero.y, own_hero.radius, {
            fill_color: own_hero.color,
        });
    }

    for (const node of named_nodes) {
        const nameColor = node.downed ? "red" : "black";
        draw_text(main_canvas, node.x, node.y + 1, node.name!, nameColor, 16, "bold");
    }

    let range = input_settings.mouseInputRange;
    drawLine(main_canvas, offset.x, offset.y, offset.x + (input.x * range), offset.y + (input.y * range), "yellow", 2);
    draw_circle(main_canvas, offset.x, offset.y, range, {
        outline_color: "orange",
        outline_width: 2
    });

    reportRenderEnd();
}

function drawMinimapHero(hero: RenderNode) {
    draw_circle(hero_minimap, hero.x, hero.y, hero.radius, {
        fill_color: hero.color,
    });
}
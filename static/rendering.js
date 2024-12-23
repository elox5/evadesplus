import { input, inputSettings } from "./input.js";
import Canvas from "./canvas.js";

export let renderSettings = {
    tileSize: 32,
}

const mainCanvas = new Canvas("main-canvas");
const areaCanvas = new Canvas("area-canvas");

export function setupCanvas() {
    mainCanvas.updateDimensions();
    mainCanvas.clear();

    window.onresize = () => {
        mainCanvas.updateDimensions();
    }
}

function setDrawOffset(x, y) {
    mainCanvas.setRenderOffset(x - mainCanvas.canvas.width / 2 / renderSettings.tileSize, -y + mainCanvas.canvas.height / 2 / renderSettings.tileSize);
    areaCanvas.setPhysicalOffset(x, y);
}

function drawCircle(canvas, _x, _y, _r, settings = {
    hasFill: true,
    hasFrame: false,
    hasOutline: false,
    fillColor: "#000",
    strokeColor: "#000",
    strokeWidth: 1,
}) {
    const ctx = canvas.ctx;
    const { x, y } = canvas.gameToCanvasPos(_x, _y);
    const r = _r * renderSettings.tileSize;

    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);

    if (settings.hasFill === true) {
        ctx.fillStyle = settings.fillColor;
        ctx.fill();
    }

    if (settings.hasOutline === true) {
        ctx.strokeStyle = settings.strokeColor;
        ctx.lineWidth = settings.strokeWidth;

        ctx.stroke();
    }

    if (settings.hasFrame === true) {
        ctx.strokeStyle = settings.strokeColor;
        ctx.lineWidth = settings.strokeWidth;

        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x - r, y);
        ctx.moveTo(x, y + r);
        ctx.lineTo(x, y - r);
        ctx.stroke();
    }
}

function drawRect(canvas, _x, _y, _w, _h, settings) {
    const ctx = canvas.ctx;
    const { x, y } = canvas.gameToCanvasPos(_x, _y);

    const w = canvas.gameToCanvas(_w);
    const h = -canvas.gameToCanvas(_h);

    if (settings.hasFill) {
        ctx.fillStyle = settings.fillColor;
        ctx.fillRect(x, y, w, h);
    }

    if (settings.hasOutline) {
        ctx.strokeStyle = settings.strokeColor;
        ctx.lineWidth = settings.strokeWidth;
        ctx.strokeRect(x, y, w, h);
    }

    if (settings.hasFrame) {
        ctx.beginPath();
        ctx.moveTo(x + w / 2, y);
        ctx.lineTo(x + w / 2, y + h);
        ctx.moveTo(x, y + h / 2);
        ctx.lineTo(x + w, y + h / 2);
        ctx.stroke();
    }
}

function drawLine(canvas, _x1, _y1, _x2, _y2, color = "#000", width = 1) {
    const ctx = canvas.ctx;
    const { x: x1, y: y1 } = canvas.gameToCanvasPos(_x1, _y1);
    const { x: x2, y: y2 } = canvas.gameToCanvasPos(_x2, _y2);

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
}

function drawText(canvas, _x, _y, text, color = "#000", size = 16, modifiers = "") {
    const ctx = canvas.ctx;
    const { x, y } = canvas.gameToCanvasPos(_x, _y);

    ctx.fillStyle = color;
    ctx.font = `${modifiers} ${size}px Nunito`;
    ctx.textAlign = "center";
    ctx.fillText(text, x, y);
}

function drawGrid(width, height) {
    const ctx = areaCanvas.ctx;

    ctx.strokeStyle = "#00000033";
    ctx.lineWidth = 1;

    for (let i = 0; i < width; i++) {
        ctx.beginPath();
        ctx.moveTo(i * renderSettings.tileSize, 0);
        ctx.lineTo(i * renderSettings.tileSize, areaCanvas.canvas.height);
        ctx.stroke();
    }

    for (let j = 0; j < height; j++) {
        ctx.beginPath();
        ctx.moveTo(0, j * renderSettings.tileSize);
        ctx.lineTo(areaCanvas.canvas.width, j * renderSettings.tileSize);
        ctx.stroke();
    }
}

export function renderArea(width, height, color, walls, safeZones) {
    areaCanvas.setDimensions(width * renderSettings.tileSize, height * renderSettings.tileSize);

    const ctx = areaCanvas.ctx;

    ctx.fillStyle = color;
    ctx.fillRect(0, 0, areaCanvas.canvas.width, areaCanvas.canvas.height);

    drawGrid(width, height);

    for (const wall of walls) {
        drawRect(areaCanvas, wall.x, wall.y, wall.w, wall.h, {
            hasFill: true,
            fillColor: "#222",
        });
    }

    for (const safeZone of safeZones) {
        drawRect(areaCanvas, safeZone.x, safeZone.y, safeZone.w, safeZone.h, {
            hasFill: true,
            fillColor: "#00000022",
        });
    }
}


export function renderFrame(offset, rects, nodes) {
    mainCanvas.clear();

    setDrawOffset(offset.x, offset.y);

    for (const rect of rects) {
        drawRect(mainCanvas, rect.x, rect.y, rect.w, rect.h, {
            hasFill: true,
            fillColor: rect.color,
            hasOutline: rect.hasBorder,
            strokeColor: "black",
            strokeWidth: 2
        });
    }

    for (const node of nodes) {
        drawCircle(mainCanvas, node.x, node.y, node.radius, {
            hasFill: true,
            fillColor: node.color,
            hasOutline: node.hasBorder,
            strokeColor: "black",
            strokeWidth: 2
        });

        if (node.name !== undefined) {
            drawText(mainCanvas, node.x, node.y + 1, node.name, "black", 16, "bold");
        }
    }

    let range = inputSettings.mouseInputRange;
    drawLine(mainCanvas, offset.x, offset.y, offset.x + (input.x * range), offset.y + (input.y * range), "yellow", 2);
    drawCircle(mainCanvas, offset.x, offset.y, range, {
        hasOutline: true,
        strokeColor: "orange",
        strokeWidth: 2
    });
}
import { input, inputSettings } from "./input.js";
import Canvas from "./canvas.js";

export let renderSettings = {
    tileSize: 40,
    minimapTileSize: 6,
    minimapHeroScale: 2,
}

const mainCanvas = new Canvas("main-canvas", renderSettings.tileSize);
const areaCanvas = new Canvas("area-canvas", renderSettings.tileSize);

const areaMinimap = new Canvas("area-minimap", renderSettings.minimapTileSize);
const heroMinimap = new Canvas("hero-minimap", renderSettings.minimapTileSize, renderSettings.minimapHeroScale);

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
    const r = _r * canvas.tileSize * canvas.radiusScale;

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
    ctx.font = `${modifiers} ${size}px Questrial, Noto Color Emoji`;
    ctx.textAlign = "center";
    ctx.fillText(text, x, y);
}

function drawGrid(width, height) {
    const ctx = areaCanvas.ctx;

    ctx.strokeStyle = "#00000033";
    ctx.lineWidth = 1;

    for (let i = 0; i < width; i++) {
        ctx.beginPath();
        ctx.moveTo(i * areaCanvas.tileSize, 0);
        ctx.lineTo(i * areaCanvas.tileSize, areaCanvas.canvas.height);
        ctx.stroke();
    }

    for (let j = 0; j < height; j++) {
        ctx.beginPath();
        ctx.moveTo(0, j * areaCanvas.tileSize);
        ctx.lineTo(areaCanvas.canvas.width, j * areaCanvas.tileSize);
        ctx.stroke();
    }
}

export function renderArea(width, height, color, walls, safeZones, portals) {
    areaCanvas.setDimensions(width * areaCanvas.tileSize, height * areaCanvas.tileSize);

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

    for (const portal of portals) {
        drawRect(areaCanvas, portal.x, portal.y, portal.w, portal.h, {
            hasFill: true,
            fillColor: portal.color,
        });
    }

    areaMinimap.setDimensions(width * renderSettings.minimapTileSize, height * renderSettings.minimapTileSize);
    heroMinimap.setDimensions(width * renderSettings.minimapTileSize, height * renderSettings.minimapTileSize);

    areaMinimap.ctx.drawImage(areaCanvas.canvas, 0, 0, areaCanvas.canvas.width, areaCanvas.canvas.height, 0, 0, areaMinimap.canvas.width, areaMinimap.canvas.height);
}


export function renderFrame(offset, rects, nodes) {
    mainCanvas.clear();
    heroMinimap.clear();

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

    let namedNodes = [];
    let ownHero = null;

    for (const node of nodes) {
        if (node.name !== undefined) {
            namedNodes.push(node);
        }

        if (node.isHero) {
            drawMinimapHero(node);

            if (node.downed) {
                drawText(heroMinimap, node.x, node.y + 1, "!", "red", 16, "bold");
            }
        }

        if (node.ownHero) {
            ownHero = node;
            continue;
        }

        drawCircle(mainCanvas, node.x, node.y, node.radius, {
            hasFill: true,
            fillColor: node.color,
            hasOutline: node.hasBorder,
            strokeColor: "black",
            strokeWidth: 2
        });
    }

    if (ownHero !== null) {
        drawCircle(mainCanvas, ownHero.x, ownHero.y, ownHero.radius, {
            hasFill: true,
            fillColor: ownHero.color,
        });
    }

    for (const node of namedNodes) {
        const nameColor = node.downed ? "red" : "black";
        drawText(mainCanvas, node.x, node.y + 1, node.name, nameColor, 16, "bold");
    }

    let range = inputSettings.mouseInputRange;
    drawLine(mainCanvas, offset.x, offset.y, offset.x + (input.x * range), offset.y + (input.y * range), "yellow", 2);
    drawCircle(mainCanvas, offset.x, offset.y, range, {
        hasOutline: true,
        strokeColor: "orange",
        strokeWidth: 2
    });
}

function drawMinimapHero(hero) {
    drawCircle(heroMinimap, hero.x, hero.y, hero.radius, {
        hasFill: true,
        fillColor: hero.color,
    });
}
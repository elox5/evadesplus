const mainCanvas = document.querySelector("#main-canvas");
const areaCanvas = document.querySelector("#area-canvas");
const mainCtx = mainCanvas.getContext("2d");
const areaCtx = areaCanvas.getContext("2d");

export let renderSettings = {
    tileSize: 32,
}

let drawOffset = { x: 0, y: 0 };

export function setupCanvas() {
    mainCanvas.width = mainCanvas.clientWidth;
    mainCanvas.height = mainCanvas.clientHeight;

    window.onresize = () => {
        mainCanvas.width = mainCanvas.clientWidth;
        mainCanvas.height = mainCanvas.clientHeight;
    }

    clearCanvas();
}

export function setDrawOffset(x, y) {
    drawOffset = { x, y };
    areaCanvas.style.translate = `${-x * renderSettings.tileSize + areaCanvas.width / 2}px ${y * renderSettings.tileSize - areaCanvas.height / 2}px`;
}

export function clearCanvas() {
    mainCtx.clearRect(0, 0, mainCanvas.width, mainCanvas.height);
}

export function drawCircle(canvas, x, y, r, color = "#000", hasOutline = false) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    r *= renderSettings.tileSize;

    ctx.fillStyle = color;

    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.fill();

    if (hasOutline) {
        ctx.strokeStyle = "black";
        ctx.lineWidth = 2;
        ctx.stroke();
    }
}

export function drawCircleOutline(canvas, x, y, r, color = "#000", width = 1) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    r *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.stroke();
}

export function drawCircleFrame(canvas, x, y, r, color = "#000", width = 1) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    r *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.moveTo(x + r, y);
    ctx.lineTo(x - r, y);
    ctx.moveTo(x, y + r);
    ctx.lineTo(x, y - r);
    ctx.stroke();
}

export function drawRect(canvas, x, y, w, h, color = "#000", hasOutline = false) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    ctx.fillStyle = color;
    ctx.fillRect(x, y, w, -h);

    if (hasOutline) {
        ctx.strokeStyle = "black";
        ctx.lineWidth = 2;
        ctx.strokeRect(x, y, w, h);
    }
}

export function drawRectOutline(canvas, x, y, w, h, color = "#000", width = 1) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.strokeRect(x, y, w, h);
}

export function drawRectFrame(canvas, x, y, w, h, color = "#000", width = 1) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.strokeRect(x, y, w, h);
    ctx.moveTo(x + w / 2, y);
    ctx.lineTo(x + w / 2, y + h);
    ctx.moveTo(x, y + h / 2);
    ctx.lineTo(x + w, y + h / 2);
    ctx.stroke();
}

export function drawLine(canvas, x1, y1, x2, y2, color = "#000", width = 1) {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x1 = gameToCanvasX(canvas, x1);
    y1 = gameToCanvasY(canvas, y1);
    x2 = gameToCanvasX(canvas, x2);
    y2 = gameToCanvasY(canvas, y2);

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
}

export function drawText(canvas, x, y, text, color = "#000", size = 16, modifiers = "") {
    const ctx = canvas == "main" ? mainCtx : areaCtx;

    x = gameToCanvasX(canvas, x);
    y = gameToCanvasY(canvas, y);

    ctx.fillStyle = color;
    ctx.font = `${modifiers} ${size}px Nunito`;
    ctx.textAlign = "center";
    ctx.fillText(text, x, y);
}

function drawGrid(width, height) {
    areaCtx.strokeStyle = "#00000033";
    areaCtx.lineWidth = 1;

    for (let i = 0; i < width; i++) {
        areaCtx.beginPath();
        areaCtx.moveTo((i - drawOffset.x % 1) * renderSettings.tileSize, 0);
        areaCtx.lineTo((i - drawOffset.x % 1) * renderSettings.tileSize, areaCanvas.height);
        areaCtx.stroke();
    }

    for (let j = 0; j < height; j++) {
        areaCtx.beginPath();
        areaCtx.moveTo(0, (j + drawOffset.y % 1) * renderSettings.tileSize);
        areaCtx.lineTo(areaCanvas.width, (j + drawOffset.y % 1) * renderSettings.tileSize);
        areaCtx.stroke();
    }
}

export function renderArea(width, height, color, walls, safeZones) {
    areaCanvas.width = width * renderSettings.tileSize;
    areaCanvas.height = height * renderSettings.tileSize;

    areaCanvas.style.width = `${areaCanvas.width}px`;
    areaCanvas.style.height = `${areaCanvas.height}px`;

    areaCtx.fillStyle = color;
    areaCtx.fillRect(0, 0, areaCanvas.width, areaCanvas.height);

    drawGrid(width, height);

    for (const wall of walls) {
        drawRect("area", wall.x, wall.y, wall.w, wall.h, "#222");
    }

    for (const safeZone of safeZones) {
        drawRect("area", safeZone.x, safeZone.y, safeZone.w, safeZone.h, "#00000033");
    }
}

function gameToCanvasX(canvas, x) {
    const width = canvas == "main" ? mainCanvas.width : 0;
    const offset = canvas == "main" ? drawOffset.x : 0;

    return (x - offset) * renderSettings.tileSize + width / 2;
}
function gameToCanvasY(canvas, y) {
    const height = canvas == "main" ? mainCanvas.height : areaCanvas.height * 2;
    const offset = canvas == "main" ? drawOffset.y : 0;

    return (offset - y) * renderSettings.tileSize + height / 2;
}
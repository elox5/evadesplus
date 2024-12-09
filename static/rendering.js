const canvas = document.querySelector("#canvas");
const ctx = canvas.getContext("2d");

export let renderSettings = {
    tileSize: 32,
    canvasWidth: 16,
    canvasHeight: 12,
}

let background = "#222";

let drawOffset = { x: 0, y: 0 };

export function setupCanvas() {
    canvas.width = renderSettings.tileSize * renderSettings.canvasWidth;
    canvas.height = renderSettings.tileSize * renderSettings.canvasHeight;

    canvas.style.width = `${canvas.width}px`;
    canvas.style.height = `${canvas.height}px`;

    clearCanvas();
}

export function setDrawOffset(x, y) {
    drawOffset = { x, y };
}

export function setBackground(color) {
    background = color;
}

export function clearCanvas() {
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
}

export function drawGrid() {
    ctx.strokeStyle = "#00000033";
    ctx.lineWidth = 1;

    for (let i = 0; i < renderSettings.canvasWidth; i++) {
        ctx.beginPath();
        ctx.moveTo((i - drawOffset.x % 1) * renderSettings.tileSize, 0);
        ctx.lineTo((i - drawOffset.x % 1) * renderSettings.tileSize, canvas.height);
        ctx.stroke();
    }

    for (let j = 0; j < renderSettings.canvasHeight; j++) {
        ctx.beginPath();
        ctx.moveTo(0, (j + drawOffset.y % 1) * renderSettings.tileSize);
        ctx.lineTo(canvas.width, (j + drawOffset.y % 1) * renderSettings.tileSize);
        ctx.stroke();
    }
}

export function drawCircle(x, y, r, color = "#000", hasOutline = false) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
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

export function drawCircleOutline(x, y, r, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.stroke();
}

export function drawCircleFrame(x, y, r, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
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

export function drawRect(x, y, w, h, color = "#000", hasOutline = false) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
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

export function drawRectOutline(x, y, w, h, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.strokeRect(x, y, w, h);
}

export function drawRectFrame(x, y, w, h, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
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

export function drawLine(x1, y1, x2, y2, color = "#000", width = 1) {
    x1 = gameToCanvasX(x1);
    y1 = gameToCanvasY(y1);
    x2 = gameToCanvasX(x2);
    y2 = gameToCanvasY(y2);

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y2);
    ctx.stroke();
}

export function drawText(x, y, text, color = "#000", size = 16, modifiers = "") {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);

    ctx.fillStyle = color;
    ctx.font = `${modifiers} ${size}px Nunito`;
    ctx.textAlign = "center";
    ctx.fillText(text, x, y);
}

function gameToCanvasX(x) {
    return (x + renderSettings.canvasWidth / 2 - drawOffset.x) * renderSettings.tileSize;
}
function gameToCanvasY(y) {
    return (renderSettings.canvasHeight / 2 - (y - drawOffset.y)) * renderSettings.tileSize;
}
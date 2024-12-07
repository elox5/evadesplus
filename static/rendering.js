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
    ctx.strokeStyle = "black";
    ctx.globalAlpha = 0.2;
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

    ctx.globalAlpha = 1;
}

export function drawCircle(x, y, r, color = "#777", hasBorder = false) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    ctx.fillStyle = color;

    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.fill();

    if (hasBorder) {
        ctx.strokeStyle = "black";
        ctx.lineWidth = 2;
        ctx.stroke();
    }
}

export function drawCircleOutline(x, y, r, color = "#777", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, 2 * Math.PI);
    ctx.stroke();
}

export function drawRect(x, y, w, h, color = "#000", alpha = 0.3) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    ctx.fillStyle = color;
    ctx.globalAlpha = alpha;
    ctx.fillRect(x, y, w, h);

    ctx.globalAlpha = 1;
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

function gameToCanvasX(x) {
    return (x + renderSettings.canvasWidth / 2 - drawOffset.x) * renderSettings.tileSize;
}
function gameToCanvasY(y) {
    return (renderSettings.canvasHeight / 2 - (y - drawOffset.y)) * renderSettings.tileSize;
}
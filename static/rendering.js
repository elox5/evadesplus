const canvas = document.querySelector("#canvas");
const ctx = canvas.getContext("2d");

let tileSize;
let canvasWidth;
let canvasHeight;

let background = "#222";

let drawOffset = { x: 0, y: 0 };

export function setupCanvas(tileSizePx, width, height) {
    tileSize = tileSizePx;
    canvasWidth = width;
    canvasHeight = height;

    canvas.width = tileSizePx * width;
    canvas.height = tileSizePx * height;

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

export function drawGrid(offsetX = 0, offsetY = 0) {
    ctx.strokeStyle = "black";
    ctx.globalAlpha = 0.2;
    ctx.lineWidth = 1;

    for (let i = 0; i < canvasWidth; i++) {
        ctx.beginPath();
        ctx.moveTo((i + offsetX) * tileSize, 0);
        ctx.lineTo((i + offsetX) * tileSize, canvas.height);
        ctx.stroke();
    }

    for (let j = 0; j < canvasHeight; j++) {
        ctx.beginPath();
        ctx.moveTo(0, (j + offsetY) * tileSize);
        ctx.lineTo(canvas.width, (j + offsetY) * tileSize);
        ctx.stroke();
    }

    ctx.globalAlpha = 1;
}

export function drawCircle(x, y, r, color = "#777", hasBorder = true) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= tileSize;

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

export function drawRect(x, y, w, h, color = "#000", alpha = 0.3) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= tileSize;
    h *= tileSize;

    ctx.fillStyle = color;
    ctx.globalAlpha = alpha;
    ctx.fillRect(x, y, w, h);

    ctx.globalAlpha = 1;
}

function gameToCanvasX(x) {
    return (x + canvasWidth / 2 - drawOffset.x) * tileSize;
}
function gameToCanvasY(y) {
    return (y + canvasHeight / 2 - drawOffset.y) * tileSize;
}
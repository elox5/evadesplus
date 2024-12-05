const canvas = document.querySelector("#canvas");
const ctx = canvas.getContext("2d");

let tileSize;
let canvasWidth;
let canvasHeight;

let background = "#222";

export function setupCanvas(_tileSize, width, height) {
    tileSize = _tileSize;
    canvasWidth = width;
    canvasHeight = height;

    canvas.width = _tileSize * width;
    canvas.height = _tileSize * height;

    canvas.style.width = `${canvas.width}px`;
    canvas.style.height = `${canvas.height}px`;

    clearCanvas();
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
    ctx.strokeWidth = 1;

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

export function drawCircle(x, y, r, color = "#777", hasBorder = false) {
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

function gameToCanvasX(x) {
    return (x + canvasWidth / 2) * tileSize;
}
function gameToCanvasY(y) {
    return (y + canvasHeight / 2) * tileSize;
}
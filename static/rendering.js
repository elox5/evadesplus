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

export function drawCircle(x, y, r, color = "#000", hasOutline = false) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    mainCtx.fillStyle = color;

    mainCtx.beginPath();
    mainCtx.arc(x, y, r, 0, 2 * Math.PI);
    mainCtx.fill();

    if (hasOutline) {
        mainCtx.strokeStyle = "black";
        mainCtx.lineWidth = 2;
        mainCtx.stroke();
    }
}

export function drawCircleOutline(x, y, r, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    mainCtx.strokeStyle = color;
    mainCtx.lineWidth = width;
    mainCtx.beginPath();
    mainCtx.arc(x, y, r, 0, 2 * Math.PI);
    mainCtx.stroke();
}

export function drawCircleFrame(x, y, r, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    r *= renderSettings.tileSize;

    mainCtx.strokeStyle = color;
    mainCtx.lineWidth = width;
    mainCtx.beginPath();
    mainCtx.arc(x, y, r, 0, 2 * Math.PI);
    mainCtx.moveTo(x + r, y);
    mainCtx.lineTo(x - r, y);
    mainCtx.moveTo(x, y + r);
    mainCtx.lineTo(x, y - r);
    mainCtx.stroke();
}

export function drawRect(x, y, w, h, color = "#000", hasOutline = false) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    mainCtx.fillStyle = color;
    mainCtx.fillRect(x, y, w, -h);

    if (hasOutline) {
        mainCtx.strokeStyle = "black";
        mainCtx.lineWidth = 2;
        mainCtx.strokeRect(x, y, w, h);
    }
}

export function drawRectOutline(x, y, w, h, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    mainCtx.strokeStyle = color;
    mainCtx.lineWidth = width;
    mainCtx.beginPath();
    mainCtx.strokeRect(x, y, w, h);
}

export function drawRectFrame(x, y, w, h, color = "#000", width = 1) {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);
    w *= renderSettings.tileSize;
    h *= renderSettings.tileSize;

    mainCtx.strokeStyle = color;
    mainCtx.lineWidth = width;
    mainCtx.beginPath();
    mainCtx.strokeRect(x, y, w, h);
    mainCtx.moveTo(x + w / 2, y);
    mainCtx.lineTo(x + w / 2, y + h);
    mainCtx.moveTo(x, y + h / 2);
    mainCtx.lineTo(x + w, y + h / 2);
    mainCtx.stroke();
}

export function drawLine(x1, y1, x2, y2, color = "#000", width = 1) {
    x1 = gameToCanvasX(x1);
    y1 = gameToCanvasY(y1);
    x2 = gameToCanvasX(x2);
    y2 = gameToCanvasY(y2);

    mainCtx.strokeStyle = color;
    mainCtx.lineWidth = width;
    mainCtx.beginPath();
    mainCtx.moveTo(x1, y1);
    mainCtx.lineTo(x2, y2);
    mainCtx.stroke();
}

export function drawText(x, y, text, color = "#000", size = 16, modifiers = "") {
    x = gameToCanvasX(x);
    y = gameToCanvasY(y);

    mainCtx.fillStyle = color;
    mainCtx.font = `${modifiers} ${size}px Nunito`;
    mainCtx.textAlign = "center";
    mainCtx.fillText(text, x, y);
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

export function renderArea(width, height, color) {
    areaCanvas.width = width * renderSettings.tileSize;
    areaCanvas.height = height * renderSettings.tileSize;

    areaCanvas.style.width = `${areaCanvas.width}px`;
    areaCanvas.style.height = `${areaCanvas.height}px`;

    areaCtx.fillStyle = color;
    areaCtx.fillRect(0, 0, areaCanvas.width, areaCanvas.height);

    drawGrid(width, height);


}

function gameToCanvasX(x) {
    return (x - drawOffset.x) * renderSettings.tileSize + mainCanvas.width / 2;
}
function gameToCanvasY(y) {
    return (drawOffset.y - y) * renderSettings.tileSize + mainCanvas.height / 2;
}
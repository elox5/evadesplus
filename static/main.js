import { renderSettings, clearCanvas, drawCircle, drawGrid, drawRect, setBackground, setDrawOffset, setupCanvas } from "./rendering.js";
import { input } from "./input.js";

const player = {
    x: 50,
    y: 10,
    r: 0.5,
    color: "red",
    border: false
};

async function main() {
    renderSettings.tileSize = 40;
    renderSettings.canvasWidth = 24;
    renderSettings.canvasHeight = 14;

    setupCanvas();
    setBackground("#aaa");

    setInterval(handleFrame, 1000 / 60);
}
window.main = main;

async function handleFrame() {
    const rects = [{
        x: 45,
        y: 5,
        w: 4,
        h: 5,
        color: "gold",
    }];
    const entities = [{
        x: 45,
        y: 5,
        r: 0.5,
        border: true,
    }, {
        x: 50,
        y: 7,
        r: 3,
        color: "#00ff0044",
    }];

    renderFrame(player, rects, entities);

    const speed = 0.2;

    player.x += input.x * speed;
    player.y += input.y * speed;
}

function renderFrame(player, rects, entities) {
    clearCanvas();
    setDrawOffset(player.x, player.y);

    for (const rect of rects) {
        drawRect(rect.x, rect.y, rect.w, rect.h, rect.color, rect.alpha);
    }

    drawGrid();

    drawCircle(player.x, player.y, player.r, player.color, player.border);
    for (const entity of entities) {
        drawCircle(entity.x, entity.y, entity.r, entity.color, entity.border);
    }
}
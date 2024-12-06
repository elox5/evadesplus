import { renderSettings } from "./rendering.js";

export let input = {
    x: 0,
    y: 0,
}

export let inputSettings = {
    mouseInputRange: 4,
}

let keyboardInput = {
    up: false,
    down: false,
    left: false,
    right: false
}

let mouseInput = {
    x: 0,
    y: 0,
}

let mouseInputActive = false;

function updateInput() {
    const keyboardTotal = getKeyboardTotal();

    if (!mouseInputActive || keyboardTotal.x !== 0 || keyboardTotal.y !== 0) {
        input.x = keyboardTotal.x;
        input.y = keyboardTotal.y;
    }
    else if (mouseInputActive) {
        input.x = mouseInput.x;
        input.y = mouseInput.y;
    }
}

function getKeyboardTotal() {
    let keyboardTotal = {
        x: 0,
        y: 0,
    }

    if (keyboardInput.up) {
        keyboardTotal.y += 1;
    }
    if (keyboardInput.down) {
        keyboardTotal.y -= 1;
    }
    if (keyboardInput.right) {
        keyboardTotal.x += 1;
    }
    if (keyboardInput.left) {
        keyboardTotal.x -= 1;
    }

    return keyboardTotal;
}

addEventListener("keydown", (e) => {
    if (e.key === "ArrowLeft" || e.key === "a") {
        keyboardInput.left = true;
    }
    else if (e.key === "ArrowRight" || e.key === "d") {
        keyboardInput.right = true;
    }
    else if (e.key === "ArrowUp" || e.key === "w") {
        keyboardInput.up = true;
    }
    else if (e.key === "ArrowDown" || e.key === "s") {
        keyboardInput.down = true;
    } else {
        return;
    }

    updateInput();
});

addEventListener("keyup", (e) => {
    if (e.key === "ArrowLeft" || e.key === "a") {
        keyboardInput.left = false;
    }
    else if (e.key === "ArrowRight" || e.key === "d") {
        keyboardInput.right = false;
    }
    else if (e.key === "ArrowUp" || e.key === "w") {
        keyboardInput.up = false;
    }
    else if (e.key === "ArrowDown" || e.key === "s") {
        keyboardInput.down = false;
    } else {
        return;
    }

    updateInput();
});

addEventListener("mousemove", (e) => {
    if (!mouseInputActive) return;

    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;
    const range = inputSettings.mouseInputRange;

    let dx = (e.clientX - centerX) / range;
    let dy = (e.clientY - centerY) / range;

    let magnitude = Math.sqrt(dx * dx + dy * dy);

    if (magnitude > range) {
        dx /= magnitude;
        dy /= magnitude;
    }

    mouseInput.x = dx;
    mouseInput.y = dy;

    updateInput();
});

addEventListener("mousedown", (e) => {
    mouseInputActive = !mouseInputActive;
    mouseInput.x = 0;
    mouseInput.y = 0;
});
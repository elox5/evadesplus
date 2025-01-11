import { chat } from "./chat.js";
import { renderSettings } from "./rendering.js";

const canvasContainer = document.querySelector("#canvas-container");

export let input = {
    x: 0,
    y: 0,
}

export let inputSettings = {
    mouseInputRange: 4,
}

let keyboardPressed = {
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

let sneaking = false;

function updateInput() {
    const keyboardInput = getKeyboardInput();

    if (!mouseInputActive || keyboardInput.x !== 0 || keyboardInput.y !== 0) {
        input.x = keyboardInput.x;
        input.y = keyboardInput.y;
    }
    else if (mouseInputActive) {
        input.x = mouseInput.x;
        input.y = mouseInput.y;
    }

    if (sneaking) {
        input.x *= 0.5;
        input.y *= 0.5;
    }
}

function getKeyboardInput() {
    let total = {
        x: 0,
        y: 0,
    }

    if (keyboardPressed.up) {
        total.y += 1;
    }
    if (keyboardPressed.down) {
        total.y -= 1;
    }
    if (keyboardPressed.right) {
        total.x += 1;
    }
    if (keyboardPressed.left) {
        total.x -= 1;
    }

    return normalize(total);
}

export function setupInput() {
    window.onkeydown = (e) => {
        if (chat.focused()) return;

        if (e.key === "Shift") {
            sneaking = true;
            updateInput();
        }

        if (e.key === "ArrowLeft" || e.code === "KeyA") {
            keyboardPressed.left = true;
        }
        else if (e.key === "ArrowRight" || e.code === "KeyD") {
            keyboardPressed.right = true;
        }
        else if (e.key === "ArrowUp" || e.code === "KeyW") {
            keyboardPressed.up = true;
        }
        else if (e.key === "ArrowDown" || e.code === "KeyS") {
            keyboardPressed.down = true;
        } else if ((e.key === "Enter" || e.code === "Slash") && !chat.focused()) {
            chat.focus();
            return;
        } else {
            return;
        }

        updateInput();
    };

    window.onkeyup = (e) => {
        if (e.key === "Shift") {
            sneaking = false;
            updateInput();
        }

        if (e.key === "ArrowLeft" || e.code === "KeyA") {
            keyboardPressed.left = false;
        }
        else if (e.key === "ArrowRight" || e.code === "KeyD") {
            keyboardPressed.right = false;
        }
        else if (e.key === "ArrowUp" || e.code === "KeyW") {
            keyboardPressed.up = false;
        }
        else if (e.key === "ArrowDown" || e.code === "KeyS") {
            keyboardPressed.down = false;
        } else {
            return;
        }

        updateInput();
    };

    window.onmousemove = (e) => {
        if (!mouseInputActive) return;

        mouseInput = calculateMouseInput(e);

        updateInput();
    };

    canvasContainer.onmousedown = (e) => {
        mouseInputActive = !mouseInputActive;
        if (!mouseInputActive) {
            mouseInput.y = 0;
            mouseInput.x = 0;
        }
        else {
            mouseInput = calculateMouseInput(e);
        }
        updateInput();
    };
}

function calculateMouseInput(e) {
    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;
    const range = inputSettings.mouseInputRange;

    let delta = {
        x: (e.clientX - centerX) / renderSettings.tileSize / range,
        y: (e.clientY - centerY) / renderSettings.tileSize / range,
    }

    const magnitude = Math.sqrt(delta.x * delta.x + delta.y * delta.y);

    if (magnitude > 1) {
        delta = normalize(delta, magnitude);
    }

    delta.y = -delta.y;

    return delta;
}

function normalize(v, magnitude = null) {
    if (magnitude === null) {
        magnitude = Math.sqrt(v.x * v.x + v.y * v.y);
    }
    if (magnitude === 0) {
        return { x: 0, y: 0 };
    }

    return { x: v.x / magnitude, y: v.y / magnitude };
}
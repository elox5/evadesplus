import { chat } from "./chat.js";
import { try_execute_command } from "./commands.js";
import { register_keydown_handler, register_keyup_handler } from "./input.js";
import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";
import { render_settings } from "./rendering.js";
import { settings } from "./settings.js";
import { Vector2 } from "./types.js";

const canvas_container = document.querySelector("#canvas-container") as HTMLDivElement;
const settings_popover = document.querySelector("#settings-popover") as HTMLDivElement;

export let player_input = {
    x: 0,
    y: 0,
}

let keyboard_pressed = {
    up: false,
    down: false,
    left: false,
    right: false
}

let mouse_input: Vector2 = {
    x: 0,
    y: 0,
}

let range = 4;
let mouse_input_active = false;
let sneaking = false;

function update_input() {
    const keyboard_input = get_keyboard_input();

    if (!mouse_input_active || keyboard_input.x !== 0 || keyboard_input.y !== 0) {
        player_input.x = keyboard_input.x;
        player_input.y = keyboard_input.y;
    }
    else if (mouse_input_active) {
        player_input.x = mouse_input.x;
        player_input.y = mouse_input.y;
    }

    if (sneaking) {
        player_input.x *= 0.5;
        player_input.y *= 0.5;
    }
}

function get_keyboard_input() {
    let total = {
        x: 0,
        y: 0,
    }

    if (keyboard_pressed.up) {
        total.y += 1;
    }
    if (keyboard_pressed.down) {
        total.y -= 1;
    }
    if (keyboard_pressed.right) {
        total.x += 1;
    }
    if (keyboard_pressed.left) {
        total.x -= 1;
    }

    return normalize(total);
}

function setup_player_input() {
    range = settings.get("gameplay.mouse_input_range");
    settings.bind("gameplay.mouse_input_range", v => range = v);

    register_keydown_handler(e => {
        if (chat.focused()) return;
        if (settings_popover.matches(":popover-open")) return;

        if (e.key === "Shift") {
            sneaking = true;
            update_input();
        }

        if (e.key === "ArrowLeft" || e.code === "KeyA") {
            keyboard_pressed.left = true;
        }
        else if (e.key === "ArrowRight" || e.code === "KeyD") {
            keyboard_pressed.right = true;
        }
        else if (e.key === "ArrowUp" || e.code === "KeyW") {
            keyboard_pressed.up = true;
        }
        else if (e.key === "ArrowDown" || e.code === "KeyS") {
            keyboard_pressed.down = true;
        } else if ((e.key === "Enter" || e.code === "Slash") && !chat.focused()) {
            chat.focus();
            return;
        } else if (e.key == "Escape" && !chat.focused()) {
            try_execute_command("/reset");
        } else {
            return;
        }

        update_input();
    });

    register_keyup_handler(e => {
        if (e.key === "Shift") {
            sneaking = false;
            update_input();
        }

        if (e.key === "ArrowLeft" || e.code === "KeyA") {
            keyboard_pressed.left = false;
        }
        else if (e.key === "ArrowRight" || e.code === "KeyD") {
            keyboard_pressed.right = false;
        }
        else if (e.key === "ArrowUp" || e.code === "KeyW") {
            keyboard_pressed.up = false;
        }
        else if (e.key === "ArrowDown" || e.code === "KeyS") {
            keyboard_pressed.down = false;
        } else {
            return;
        }

        update_input();
    });

    window.onmousemove = (e) => {
        if (!mouse_input_active) return;

        mouse_input = calculate_mouse_input(e);

        update_input();
    };

    canvas_container.onmousedown = (e) => {
        if (e.button !== 0) return;
        if (settings_popover.matches(":popover-open")) return;

        mouse_input_active = !mouse_input_active;

        if (!mouse_input_active) {
            mouse_input.x = 0;
            mouse_input.y = 0;
        }
        else {
            mouse_input = calculate_mouse_input(e);
        }
        update_input();
    };
}

export function lock_mouse_input() {
    mouse_input_active = false;

    mouse_input.x = 0;
    mouse_input.y = 0;

    update_input();
}

function calculate_mouse_input(e: MouseEvent) {
    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;

    let delta = {
        x: (e.clientX - centerX) / render_settings.tile_size / range,
        y: (e.clientY - centerY) / render_settings.tile_size / range,
    }

    const magnitude = Math.sqrt(delta.x * delta.x + delta.y * delta.y);

    if (magnitude > 1) {
        delta = normalize(delta, magnitude);
    }

    delta.y = -delta.y;

    return delta;
}

function normalize(v: Vector2, magnitude: number | undefined = undefined) {
    if (magnitude === undefined) {
        magnitude = Math.sqrt(v.x * v.x + v.y * v.y);
    }
    if (magnitude === 0) {
        return { x: 0, y: 0 };
    }

    return { x: v.x / magnitude, y: v.y / magnitude };
}

class PlayerInputModule implements NetworkModule {
    private lastInput: Vector2 = {
        x: 0,
        y: 0,
    }

    private interval: number | undefined;

    setup = {
        callback: (controller: NetworkController) => {
            const input_writer = controller.create_datagram_writer();

            if (input_writer === null) {
                console.warn("Failed to register input module");
                return;
            }

            this.interval = setInterval(async () => {
                this.send_input(input_writer, player_input);
            }, 1000 / 60);
        },
        once: false,
    }

    on_game_load = {
        callback: setup_player_input,
        once: true,
    }

    cleanup() {
        clearInterval(this.interval);
        lock_mouse_input();
    }

    private async send_input(writer: WritableStreamDefaultWriter, input: Vector2) {
        if (this.lastInput.x === input.x && this.lastInput.y === input.y) {
            return;
        }

        const input_array = new Float32Array([input.x, input.y]);
        const data = new Uint8Array(input_array.buffer);

        this.lastInput.x = input.x;
        this.lastInput.y = input.y;

        await writer.write(data);
    }
}

network_controller.register_module(new PlayerInputModule());
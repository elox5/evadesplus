import { InitError, network_controller, NetworkModule } from "./network_controller.js";
import { init_cache } from "./cache.js";
import { setup_input } from "./input.js";
import { MessageHandler, ws_connector, WsConnector } from "./ws_connector.js";
import { BinaryReader } from "./binary_reader.js";

const game_container = document.querySelector("#game-container") as HTMLDivElement;
const connection_panel = document.querySelector("#connection-panel") as HTMLDivElement;
const connect_button = document.querySelector("#connect-button") as HTMLButtonElement;
const connection_message_display = document.querySelector("#connection-message") as HTMLDivElement;

async function main() {
    window.oncontextmenu = (e) => e.preventDefault();

    setup_input();

    display_connection_message("Fetching cache...", "#dddddd");

    const warning_timeout = setTimeout(() => {
        display_connection_message("Fetching cache is taking longer than expected. Check the console for any errors", "#ffbf40");
    }, 5000);

    try {
        await init_cache();

        clear_connection_message();
        connect_button.disabled = false;
        connect_button.onclick = handle_connection;
    }
    catch (err) {
        display_connection_message("Failed to fetch cache. Check the console for more info", "#ff3f3f");
        console.error("Failed to fetch cache:\n", err);
    }
    finally {
        clearTimeout(warning_timeout);
    }
}
window.onload = main;

async function handle_connection() {
    const name_input = document.querySelector("#name-input") as HTMLInputElement;
    const name = name_input.value.trim();

    if (name.length === 0) {
        display_connection_message("Please enter a name", "#ffbf3f");
        return;
    }


    await ws_connector.init();
    await ws_connector.ready();

    const handler: MessageHandler = {
        header: "INIT",
        callback: (message: BinaryReader) => {
            const response = message.read_u8();
            const user_id = message.read_u64();

            console.log(`Received INIT response: ${response}`);
            console.log(`User ID: ${user_id}`);

            post_connect("ok");
        }
    };

    ws_connector.register_handler(handler);

    const encoder = new TextEncoder();
    const name_bytes = encoder.encode(name);

    ws_connector.send("INIT", name_bytes);

    console.log("Connecting...");
    display_connection_message("Connecting...", "#bfff3f");
}

function post_connect(response: "ok" | "already_connected" | "name_invalid" | InitError) {
    try {
        if (response === "already_connected") {
            display_connection_message("Websocket connection already established", "#ffbf3f");
            return;
        }

        if (response === "name_invalid") {
            display_connection_message("Invalid name! Forbidden characters: #, @, $, ^, :, /, \\, *", "#ffbf3f");
            return;
        }

        if (typeof response === "object") {
            display_connection_message(`Error encountered during initialization:\n ${response.message}`, "#ff3f3f");
            return;
        }

        show_game();

        // network_controller.run_game_load_callbacks();

        clear_connection_message();
        connect_button.disabled = true;
    }
    catch (err) {
        display_connection_message("Failed to establish WebTransport connection. Check the console for more info", "#ff3f3f");
        console.error("Failed to establish WebTransport connection:\n", err);
    }
}

function show_game() {
    game_container.classList.remove("hidden");
    connection_panel.classList.add("hidden");
}

function display_connection_message(message: string, color: string) {
    connection_message_display.textContent = message;
    connection_message_display.style.color = color;
}

function clear_connection_message() {
    connection_message_display.textContent = "";
}

class CleanupModule implements NetworkModule {
    cleanup() {
        game_container.classList.add("hidden");
        connection_panel.classList.remove("hidden");

        connect_button.disabled = false;
    }
}

network_controller.register_module(new CleanupModule());
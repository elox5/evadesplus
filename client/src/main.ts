import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";
import { cache } from "./cache.js";

const game_container = document.querySelector("#game-container") as HTMLDivElement;
const connection_panel = document.querySelector("#connection-panel") as HTMLDivElement;
const connect_button = document.querySelector("#connect-button") as HTMLButtonElement;
const connection_message_display = document.querySelector("#connection-message") as HTMLDivElement;

async function main() {
    await cache.init();

    connect_button.disabled = false;
    connect_button.onclick = handle_connection;
}
window.onload = main;

async function handle_connection() {
    const name_input = document.querySelector("#name-input") as HTMLInputElement;
    const name = name_input.value.trim();

    console.log("Connecting...");

    if (name.length === 0) {
        display_connection_message("Please enter a name", "#ffbf40");
        return;
    }

    connect_button.disabled = true;

    await network_controller.connect(name);

    show_game();

    network_controller.run_module_pre_register();
}

function show_game() {
    game_container.classList.remove("hidden");
    connection_panel.classList.add("hidden");
}

function display_connection_message(message: string, color: string) {
    connection_message_display.textContent = message;
    connection_message_display.style.color = color;
}

class CleanupModule implements NetworkModule {
    register(_: NetworkController) { }

    cleanup() {
        this.return_to_menu();
        cache.current_players = [];
    }

    private return_to_menu() {
        game_container.classList.add("hidden");
        connection_panel.classList.remove("hidden");

        connect_button.disabled = false;
    }
}

network_controller.register_module(new CleanupModule());
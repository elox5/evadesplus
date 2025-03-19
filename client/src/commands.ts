import { chat } from "./chat.js";
import { lock_mouse_input } from "./player_input.js";
import { cache, CommandData } from "./cache.js";
import { network_controller } from "./network_controller.js";
import { player_info } from "./player_info.js";
import { reset_timer } from "./timer.js";

export function try_execute_command(message: string): boolean {
    if (cache.commands === null) {
        chat.mock_server_response("The command list cache has not been initialized yet. Please wait a few seconds...");

        return true;
    }

    message = message.substring(1).trim();

    if (message.length === 0) {
        return true;
    }

    const split = message.split(" ");
    const commandName = split[0].trim();
    const args = split.slice(1);

    if (!is_valid_command(commandName)) {
        chat.mock_server_response(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`);

        return true;
    }
    else if (matches_command_with_name(commandName, "help")) {
        const arg = args[0];

        if (arg === "" || arg === undefined) {
            display_command_list();
        }
        else {
            display_command_help(arg);
        }

        return true;
    }
    else if (matches_command_with_name(commandName, "reply")) {
        if (chat.reply_target === undefined) {
            chat.mock_server_response("There's nobody to reply to.");

            return true;
        }
        if (!player_info.player_exists(chat.reply_target)) {
            chat.mock_server_response("The target player is no longer available.");

            return true;
        }

        chat.send_message_raw(`/whisper @${chat.reply_target} ${args.join(" ")}`);

        return true;
    }
    else if (matches_command_with_name(commandName, "togglereply")) {
        chat.settings.auto_reply = !chat.settings.auto_reply;

        chat.mock_server_response(`Auto-reply is now ${chat.settings.auto_reply ? "enabled" : "disabled"}.`);

        return true;
    }
    else if (matches_command_with_name(commandName, "reset")) {
        chat.send_message_raw("/reset");

        reset_timer();
        lock_mouse_input();
    }
    else if (matches_command_with_name(commandName, "clear")) {
        chat.clear();

        chat.mock_server_response("Chat cleared.");

        return true;
    }
    else if (matches_command_with_name(commandName, "disconnect")) {
        network_controller.disconnect();

        return true;
    }
    else if (matches_command_with_name(commandName, "filter")) {
        if (args[0] === undefined) {
            const self_id = player_info.get_self_id();

            if (self_id === null) {
                chat.set_map_filter(null);
                return true;
            }

            const self = player_info.get_player(self_id);

            if (self === null) {
                chat.set_map_filter(null);
                return true;
            }

            if (self.area_info.map_id === chat.get_map_filter()) {
                chat.set_map_filter(null);
                return true;
            }

            chat.set_map_filter(self.area_info.map_id);
            return true;
        }

        if (!cache.maps.some(map => map.id === args[0])) {
            chat.mock_server_response(`Map '${args[0]}' does not exist.`);
            return true;
        }

        if (args[0] === "off" || args[0] === chat.get_map_filter()) {
            chat.set_map_filter(null);
        }
        else {
            chat.set_map_filter(args[0]);
        }

        return true;
    }

    return false;
}

function display_command_help(name: string) {
    const command = try_get_command(name);

    if (command === null) {
        display_command_list();
        return;
    }

    let msg = `*/${command.name}*\n`;
    msg += command.description;

    if (command.usage !== null) {
        msg += `\n\nUsage: /${command.name} ${command.usage}`;
    }

    if (command.aliases !== null) {
        msg += `\n\nAliases: ${command.aliases.map(alias => `/${alias}`).join(", ")}`;
    }

    chat.mock_server_response(msg);
}

function display_command_list() {
    let msg = "*Available commands:*\n";

    msg += cache.commands.map(command => `/${command.name}`).join(", ");

    msg += `\n\n Use */help <command>* for more information about a specific command.`;

    chat.mock_server_response(msg);
}

function matches_command_with_name(name: string, command_name: string) {
    name = name.toLowerCase();

    const command = cache.commands.find(command => command.name === command_name);

    if (command === undefined) {
        return false;
    }

    return matches_command(name, command);
}

function matches_command(name: string, command: CommandData) {
    if (name.charAt(0) == "/") {
        name = name.substring(1);
    }

    name = name.toLowerCase();

    return command.name === name || (command.aliases !== null && command.aliases.some(alias => alias === name));
}

function is_valid_command(name: string) {
    for (let command of cache.commands) {
        if (matches_command(name, command)) {
            return true;
        }
    }

    return false;
}

export function try_get_command(name: string): CommandData | null {
    for (let command of cache.commands) {
        if (matches_command(name, command)) {
            return command;
        }
    }

    return null;
}
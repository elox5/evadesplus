import { chat } from "./chat.js";
import { lock_mouse_input } from "./input.js";
import { cache, CommandData } from "./cache.js";
import { network_controller } from "./network_controller.js";

export function try_execute_command(message: string) {
    const response = {
        executed: false,
        message: message,
    }

    if (cache.commands === null) {
        mock_server_response("The command list cache has not been initialized yet. Please wait a few seconds...");

        response.executed = true;
        return response;
    }

    message = message.substring(1).trim();

    if (message.length === 0) {
        response.executed = true;
        return response;
    }

    const split = message.split(" ");
    const commandName = split[0].trim();
    const args = split.slice(1);

    if (!is_valid_command(commandName)) {
        mock_server_response(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`);

        response.executed = true;
    }
    else if (matches_command_with_name(commandName, "help")) {
        const arg = args[0];

        if (arg === "" || arg === undefined) {
            display_command_list();
        }
        else {
            display_command_help(arg);
        }

        response.executed = true;
    }
    else if (matches_command_with_name(commandName, "reply")) {
        if (chat.reply_target === undefined) {
            mock_server_response("There's nobody to reply to.");

            response.executed = true;
            return response;
        }
        if (!cache.current_players.some(p => p.player_id === chat.reply_target)) {
            mock_server_response("The target player is no longer available.");

            response.executed = true;
            return response;
        }

        response.message = `/whisper @${chat.reply_target} ${args.join(" ")}`;
    }
    else if (matches_command_with_name(commandName, "togglereply")) {
        chat.settings.auto_reply = !chat.settings.auto_reply;

        mock_server_response(`Auto-reply is now ${chat.settings.auto_reply ? "enabled" : "disabled"}.`);

        response.executed = true;
    }
    else if (matches_command_with_name(commandName, "reset")) {
        lock_mouse_input();
    }
    else if (matches_command_with_name(commandName, "clear")) {
        chat.clear();

        mock_server_response("Chat cleared.");

        response.executed = true;
    }
    else if (matches_command_with_name(commandName, "disconnect")) {
        network_controller.disconnect();

        response.executed = true;
    }

    return response;
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

    mock_server_response(msg);
}

function display_command_list() {
    let msg = "*Available commands:*\n";

    msg += cache.commands.map(command => `/${command.name}`).join(", ");

    msg += `\n\n Use */help <command>* for more information about a specific command.`;

    mock_server_response(msg);
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

function mock_server_response(message: string) {
    chat.receive_message(message, -1n, "", 2);
}
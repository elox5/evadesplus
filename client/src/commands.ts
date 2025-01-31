import { chat, chat_settings } from "./chat.js";
import { lock_mouse_input } from "./input.js";
import { cache, CommandData } from "./main.js";
import { disconnect } from "./networking.js";

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

    if (!isValidCommand(commandName)) {
        mock_server_response(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`);

        response.executed = true;
    }
    else if (matches_command_name(commandName, "help")) {
        const messages = [];

        for (let command of cache.commands) {
            let msg = `*/${command.name}* - ${command.description}`;

            if (command.usage !== null) {
                msg += `\nUsage: ${command.usage}`;
            }

            if (command.aliases !== null) {
                let aliases = command.aliases.map(alias => `/${alias}`);

                msg += "\nAliases: ";
                msg += aliases.join(", ");
            }

            messages.push(msg);
        }

        const helpMessage = messages.join("\n\n");

        mock_server_response(helpMessage);

        response.executed = true;
    }
    else if (matches_command_name(commandName, "reply")) {
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
    else if (matches_command_name(commandName, "togglereply")) {
        chat_settings.auto_reply = !chat_settings.auto_reply;

        mock_server_response(`Auto-reply is now ${chat_settings.auto_reply ? "enabled" : "disabled"}.`);

        response.executed = true;
    }
    else if (matches_command_name(commandName, "reset")) {
        lock_mouse_input();
    }
    else if (matches_command_name(commandName, "clear")) {
        chat.clear();

        mock_server_response("Chat cleared.");

        response.executed = true;
    }
    else if (matches_command_name(commandName, "disconnect")) {
        disconnect();

        response.executed = true;
    }

    return response;
}

function matches_command_name(name: string, command_name: string) {
    const command = cache.commands.find(command => command.name === command_name);

    if (command === undefined) {
        return false;
    }

    return matches_command(name, command);
}

function matches_command(name: string, command: CommandData) {
    return command.name === name || (command.aliases !== null && command.aliases.some(alias => alias === name));
}

function isValidCommand(name: string) {
    for (let command of cache.commands) {
        if (matches_command(name, command)) {
            return true;
        }
    }

    return false;
}

function mock_server_response(message: string) {
    chat.receive_message(message, -1n, "", 2);
}
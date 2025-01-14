import { chat } from "./chat.js";
import { lockMouseInput } from "./input.js";
import { currentPlayers } from "./leaderboard.js";

export const commandList = [];

export function tryExecuteCommand(message) {
    const response = {
        executed: false,
        message: message,
    }

    if (commandList === null) {
        chat.receiveMessage("The command list cache has not been initialized yet. Please wait a few seconds...", -1, "", 2);

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
        chat.receiveMessage(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`, -1, "", 2);

        response.executed = true;
    }
    else if (matchesCommand(commandName, "help")) {
        const messages = [];

        for (let command of commandList) {
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

        chat.receiveMessage(helpMessage, -1, "", 2);

        response.executed = true;
    }
    else if (matchesCommand(commandName, "reply")) {
        if (chat.replyTarget === null) {
            chat.receiveMessage("There's nobody to reply to.", -1, "", 2);

            response.executed = true;
            return response;
        }
        if (!currentPlayers.includes(chat.replyTarget)) {
            chat.receiveMessage("The target player is no longer available.", -1, "", 2);

            response.executed = true;
            return response;
        }

        response.message = `/whisper @${chat.replyTarget} ${args.join(" ")}`;
    }
    else if (matchesCommand(commandName, "togglereply")) {
        chat.autoReply = !chat.autoReply;

        chat.receiveMessage(`Auto-reply is now ${chat.autoReply ? "enabled" : "disabled"}.`, -1, "", 2);

        response.executed = true;
    }
    else if (matchesCommand(commandName, "reset")) {
        lockMouseInput();
    }
    else if (matchesCommand(commandName, "clear")) {
        chat.clear();

        chat.receiveMessage("Chat cleared.", -1, "", 2);

        response.executed = true;
    }

    return response;
}

function matchesCommand(name, command) {
    if (typeof (command) === "string") {
        command = commandList.find(c => c.name === command);
    }

    return command.name === name || (command.aliases !== null && command.aliases.some(alias => alias === name));
}

function isValidCommand(name) {
    for (let command of commandList) {
        if (matchesCommand(name, command)) {
            return true;
        }
    }

    return false;
}
import { chat } from "./chat.js";

export const commandList = [];

export function tryExecuteCommand(message) {
    const SERVER_ID = 1_000_000_000_000_000;

    if (commandList === null) {
        chat.receiveMessage("The command list cache has not been initialized yet. Please wait a few seconds...", SERVER_ID, "", 2);
        return true;
    }

    message = message.substring(1).trim();

    if (message.length === 0) {
        return true;
    }

    let commandName = message.split(" ")[0].trim();

    if (!isValidCommand(commandName)) {
        chat.receiveMessage(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`, SERVER_ID, "", 2);
        return true;
    }
    else if (commandName === "help") {
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

        chat.receiveMessage(helpMessage, SERVER_ID, "", 2);

        return true;
    }

    return false;
}

function isValidCommand(name) {
    for (let command of commandList) {
        if (command.name === name) return true;

        if (command.aliases !== null) {
            if (command.aliases.some(alias => alias === name)) return true;
        }
    }
}
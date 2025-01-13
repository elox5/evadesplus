import { sendChatMessage } from "./networking.js";

class Chat {
    constructor() {
        this.messages = [];
        this.messageElements = [];

        this.commandList = null;

        this.element = document.getElementById("chat");
        this.list = document.getElementById("chat-list");
        this.input = document.getElementById("chat-input");
        const sendButton = document.getElementById("chat-send-button");

        sendButton.onclick = () => {
            this.sendMessage(this.input.value);
        };

        this.input.onkeydown = (e) => {
            if (e.key === "Enter") {
                this.sendMessage(this.input.value);
                e.stopPropagation();
            }
        };
    }

    isValidCommand(name) {
        for (let command of this.commandList) {
            if (command.name === name) return true;

            if (command.aliases !== null) {
                if (command.aliases.some(alias => alias === name)) return true;
            }
        }
    }

    sendMessage(message) {
        this.input.blur();
        this.input.value = "";

        message = message.trim();
        if (message.length === 0) {
            return;
        }

        if (message.startsWith("/")) {
            if (this.tryExecuteCommand(message.slice(1))) {
                return;
            }
        }

        sendChatMessage(message);
    }

    tryExecuteCommand(command) {
        if (this.commandList === null) {
            this.receiveMessage("The command list cache has not been initialized yet. Please wait a few seconds...", "", 2);
            return true;
        }

        if (command.trim().length === 0) {
            return true;
        }

        let commandName = command.split(" ")[0].trim();

        if (!this.isValidCommand(commandName)) {
            this.receiveMessage(`Unknown command: */${commandName}*. For a list of available commands, use */help*.`, "", 2);
            return true;
        }
        else if (commandName === "help") {
            const messages = [];

            for (command of this.commandList) {
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

            this.receiveMessage(helpMessage, "", 2);

            return true;
        }

        return false;
    }

    receiveMessage(message, senderId, name, messageType) {
        const atBottom = Math.abs(this.list.scrollHeight - this.list.clientHeight - this.list.scrollTop) < 1

        const entry = document.createElement("div");
        entry.classList.add("chat-entry");

        const showUsername = messageType === 0 || messageType === 1;

        message = message.replace(/\n/g, "\r\n");

        if (showUsername) {
            message = `${name}: ${message}`;
        }

        entry.textContent = message;

        entry.innerHTML = entry.innerHTML.replace(/\*([^*]*)\*/g, "<strong>$1</strong>");

        if (messageType === 0) entry.classList.add("normal");
        if (messageType === 1) entry.classList.add("special", "whisper");
        if (messageType === 2) entry.classList.add("special", "command-response");
        if (messageType === 3) entry.classList.add("special", "server-announcement");
        if (messageType === 4) entry.classList.add("special", "server-error");

        entry.onmousedown = (e) => {
            if (e.button === 2) {
                navigator.clipboard.writeText(`@${senderId}`);
                e.preventDefault();
            }
        }
        entry.oncontextmenu = (e) => { e.preventDefault(); }

        this.list.appendChild(entry);

        this.messages.push({
            senderId: senderId,
            element: entry,
        });

        if (atBottom) {
            this.list.scrollTo(0, this.list.scrollHeight);
        }
    }

    focus() {
        this.input.focus();
    }

    focused() {
        return this.input === document.activeElement;
    }
}

export const chatSettings = {
    maxMessages: 100
}

export const chat = new Chat();
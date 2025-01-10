import { sendChatMessage } from "./networking.js";

class Chat {
    constructor() {
        this.messages = [];
        this.messageElements = [];

        this.element = document.getElementById("chat");
        this.list = document.getElementById("chat-list");
        this.input = document.getElementById("chat-input");
        const sendButton = document.getElementById("chat-send-button");

        sendButton.onclick = () => {
            this.sendMessage(this.input.value, this.username);
        };

        this.input.onkeydown = (e) => {
            if (e.key === "Enter") {
                this.sendMessage(this.input.value, this.username);
                e.stopPropagation();
            }
        };
    }

    setUsername(username) {
        this.username = username;
    }

    sendMessage(message, name) {
        this.input.blur();

        message = message.trim();
        if (message.length === 0) {
            return;
        }

        sendChatMessage(message, name);
        this.input.value = "";
    }

    receiveMessage(message, name, messageType) {
        const atBottom = Math.abs(this.list.scrollHeight - this.list.clientHeight - this.list.scrollTop) < 1

        const entry = document.createElement("div");
        entry.classList.add("chat-entry");

        const showUsername = messageType === 0 || messageType === 1;

        message = message.replace(/\n/g, "<br>");
        console.log(message);


        if (showUsername) {
            entry.innerHTML = `<b>${name}</b>: ${message}`;
        } else {
            entry.innerHTML = message;
        }

        if (messageType === 0) entry.classList.add("normal");
        if (messageType === 1) entry.classList.add("whisper");
        if (messageType === 2) entry.classList.add("command-response");
        if (messageType === 3) entry.classList.add("server-announcement");
        if (messageType === 4) entry.classList.add("server-error");

        this.list.appendChild(entry);

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
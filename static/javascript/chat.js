import { sendChatMessage } from "./networking.js";

class Chat {
    constructor() {
        this.messages = [];
        this.sentMessageTimeQueue = [];

        this.messageElements = [];

        this.replyTarget = null;
        this.autoReply = false;

        this.selfId = null;

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

    sendMessage(message) {
        if (this.sentMessageTimeQueue.length === 10) {
            const tenMessagesTime = Date.now() - this.sentMessageTimeQueue[0];

            if (tenMessagesTime < 10000) {
                this.receiveMessage(`Please wait ${(10 - tenMessagesTime / 1000).toFixed(1)} seconds before sending another message.`, -1, "", 2);
                return;
            }
        }

        this.input.blur();
        this.input.value = "";

        message = message.trim();
        if (message.length === 0) {
            return;
        }

        this.sentMessageTimeQueue.push(Date.now());
        if (this.sentMessageTimeQueue.length > 10) {
            this.sentMessageTimeQueue.shift();
        }

        sendChatMessage(message);
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

        if (messageType === 1 && senderId !== this.selfId) {
            this.replyTarget = senderId;
        }

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

    clear() {
        this.messages = [];
        this.messageElements = [];

        this.list.innerHTML = "";
    }
}

export const chatSettings = {
    maxMessages: 100
}

export const chat = new Chat();
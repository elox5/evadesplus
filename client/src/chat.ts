import { cache } from "./main.js";
import { send_chat_message } from "./networking.js";

class Chat {
    private messages: ChatMessage[];
    private message_timestamp_queue: number[];

    private list: HTMLDivElement;
    private input: HTMLInputElement;

    reply_target?: bigint;
    private self_id: bigint;

    constructor() {
        this.messages = [];
        this.message_timestamp_queue = [];

        this.list = document.getElementById("chat-list") as HTMLDivElement;
        this.input = document.getElementById("chat-input") as HTMLInputElement;
        const send_button = document.getElementById("chat-send-button") as HTMLButtonElement;

        send_button.onclick = () => {
            this.send_message(this.input.value);
        };

        this.input.onkeydown = (e) => {
            if (e.key === "Enter") {
                this.send_message(this.input.value);
                e.stopPropagation();
            }
        };
    }

    set_self_id(self_id: bigint) {
        this.self_id = self_id;
    }

    send_message(message: string) {
        if (this.message_timestamp_queue.length === 10) {
            const tenMessagesTime = Date.now() - this.message_timestamp_queue[0];

            if (tenMessagesTime < 10000) {
                this.receive_message(`Please wait ${(10 - tenMessagesTime / 1000).toFixed(1)} seconds before sending another message.`, -1n, "", 2);
                return;
            }
        }

        this.input.blur();
        this.input.value = "";

        message = message.trim();
        if (message.length === 0) {
            return;
        }

        this.message_timestamp_queue.push(Date.now());
        if (this.message_timestamp_queue.length > 10) {
            this.message_timestamp_queue.shift();
        }

        send_chat_message(message);
    }

    receive_message(message: string, sender_id: bigint, name: string, message_type: MessageType) {
        const atBottom = Math.abs(this.list.scrollHeight - this.list.clientHeight - this.list.scrollTop) < 1

        const entry = document.createElement("div");
        entry.classList.add("chat-entry");

        const show_username = message_type === MessageType.Normal || message_type === MessageType.Whisper;

        message = message.replace(/\n/g, "\r\n");

        entry.textContent = message;

        entry.innerHTML = entry.innerHTML.replace(/\*([^*]*)\*/g, "<strong>$1</strong>");

        if (show_username) {
            const mapId = cache.current_players.find(p => p.player_id === sender_id)!.map_id;
            const map = cache.maps.find(m => m.id === mapId)!;

            entry.innerHTML = `<span style="color: ${map.text_color};">${name}</span>: ${entry.innerHTML}`;
        }

        if (message_type === MessageType.Normal) entry.classList.add("normal");
        if (message_type === MessageType.Whisper) entry.classList.add("special", "whisper");
        if (message_type === MessageType.CommandResponse) entry.classList.add("special", "command-response");
        if (message_type === MessageType.ServerAnnouncement) entry.classList.add("special", "server-announcement");
        if (message_type === MessageType.ServerError) entry.classList.add("special", "server-error");

        if (message_type === MessageType.Whisper && sender_id !== this.self_id) {
            this.reply_target = sender_id;
        }

        entry.onmousedown = (e) => {
            if (e.button === 2) {
                navigator.clipboard.writeText(`@${sender_id}`);
                e.preventDefault();
            }
        }
        entry.oncontextmenu = (e) => { e.preventDefault(); }

        this.list.appendChild(entry);

        this.messages.push({
            sender_id: sender_id,
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

        this.list.innerHTML = "";
    }
}

type ChatMessage = {
    sender_id: bigint;
    element: HTMLElement;
}

export enum MessageType {
    Normal = 0,
    Whisper = 1,
    CommandResponse = 2,
    ServerAnnouncement = 3,
    ServerError = 4
}

export const chat_settings = {
    max_messages: 100,
    auto_reply: false,
}

export const chat = new Chat();
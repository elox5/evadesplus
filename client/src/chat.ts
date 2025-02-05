import { get_autocomplete } from "./autocomplete.js";
import { BinaryReader } from "./binary_reader.js";
import { cache } from "./cache.js";
import { try_execute_command } from "./commands.js";
import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";

class Chat {
    private messages: ChatMessage[];
    private message_timestamp_queue: number[];

    private list: HTMLDivElement;
    private input: HTMLInputElement;
    private autocomplete_display: HTMLDivElement;

    settings = {
        max_messages: 100,
        auto_reply: false,
    }

    reply_target?: bigint;
    private self_id: bigint;

    private autocomplete_entries: string[] | null = null;
    private autocomplete_index: number = 0;

    constructor() {
        this.messages = [];
        this.message_timestamp_queue = [];

        this.list = document.getElementById("chat-list") as HTMLDivElement;
        this.input = document.getElementById("chat-input") as HTMLInputElement;
        const send_button = document.getElementById("chat-send-button") as HTMLButtonElement;

        this.autocomplete_display = document.getElementById("chat-autocomplete") as HTMLDivElement;

        send_button.onclick = () => {
            this.send_message(this.input.value);
        };

        this.input.onkeydown = (e) => {
            if (e.key === "Enter") {
                if (this.autocomplete_entries !== null) {
                    const should_send = this.fill_autocomplete(this.autocomplete_entries[this.autocomplete_index]);
                    if (!should_send) return;
                }

                this.send_message(this.input.value);
                e.stopPropagation();
            }
            else if (e.key === "Tab" && this.autocomplete_entries !== null) {
                this.cycle_autocomplete_down();

                e.preventDefault();
                e.stopPropagation();
            }
            else if (e.key === "ArrowDown" && this.autocomplete_entries !== null) {
                this.cycle_autocomplete_down();

                e.stopPropagation();
            }
            else if (e.key === "ArrowUp" && this.autocomplete_entries !== null) {
                this.cycle_autocomplete_up();

                e.stopPropagation();
            }
        };

        this.input.onkeyup = (_) => {
            if (this.input.value.charAt(0) === "/") {

                const entries = get_autocomplete(this.input.value);

                if (entries === null || entries.length === 0) {
                    this.hide_autocomplete();
                }
                else {
                    this.show_autocomplete(entries);
                }
            } else {
                this.hide_autocomplete();
            }
        }
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

        module.send_chat_message(message);
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

    private hide_autocomplete() {
        this.autocomplete_entries = null;
        this.autocomplete_index = 0;

        this.autocomplete_display.classList.add("hidden");
    }

    private show_autocomplete(entries: string[]) {
        this.autocomplete_entries = entries;

        this.autocomplete_display.classList.remove("hidden");

        this.autocomplete_display.innerHTML = "";
        for (const entry of entries) {
            const button = document.createElement("button");
            button.classList.add("autocomplete-entry");

            button.textContent = entry;
            button.onclick = () => {
                this.fill_autocomplete(entry);
            };

            this.autocomplete_display.appendChild(button);
        }

        if (this.autocomplete_index >= this.autocomplete_entries.length) {
            this.autocomplete_index = this.autocomplete_entries.length - 1;
        }

        this.focus_autocomplete();
    }

    private focus_autocomplete() {
        const entry_element = this.autocomplete_display.children[this.autocomplete_index] as HTMLButtonElement;

        for (let i = 0; i < this.autocomplete_display.children.length; i++) {
            this.autocomplete_display.children[i].classList.remove("selected");
        }

        entry_element.classList.add("selected");

        this.autocomplete_display.scrollTop = entry_element.offsetTop - this.autocomplete_display.offsetTop;
    }

    private cycle_autocomplete_up() {
        if (this.autocomplete_entries === null) {
            return;
        }

        this.autocomplete_index = (this.autocomplete_index - 1 + this.autocomplete_entries.length) % this.autocomplete_entries.length;

        this.focus_autocomplete();
    }

    private cycle_autocomplete_down() {
        if (this.autocomplete_entries === null) {
            return;
        }

        this.autocomplete_index = (this.autocomplete_index + 1) % this.autocomplete_entries.length;

        this.focus_autocomplete();
    }

    private fill_autocomplete(entry: string): boolean {
        if (entry === "") {
            this.hide_autocomplete();
            return true;
        }

        const words = this.input.value.split(" ");
        const last_word = words[words.length - 1];

        if (last_word.replace("/", "") === entry) {
            this.hide_autocomplete();
            return true;
        }

        const string_without_last_word = words.slice(0, words.length - 1).join(" ");

        if (string_without_last_word === "") {
            this.input.value = `/${entry} `;
        }
        else {
            this.input.value = `${string_without_last_word} ${entry} `;
        }

        this.hide_autocomplete();
        return false;
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

export const chat = new Chat();

export class ChatModule implements NetworkModule {
    async register(controller: NetworkController) {
        controller.register_uni_handler("CHBR", this.handle_broadcast.bind(this));
    }

    async cleanup() {
        chat.clear();
    }

    private handle_broadcast(data: BinaryReader) {
        const message_type = data.read_u8() as MessageType;
        const sender_id = data.read_u64();

        const name = data.read_length_u8_string()!;
        const message = data.read_length_u8_string()!;

        chat.receive_message(message, sender_id, name, message_type);
    }

    async send_chat_message(msg: string) {
        if (chat.settings.auto_reply
            && chat.reply_target !== undefined
            && cache.current_players.some(p => p.player_id === chat.reply_target)
            && !msg.startsWith("/")
        ) {
            msg = `/reply ${msg}`;
        }

        if (msg.startsWith("/")) {
            let { executed, message } = try_execute_command(msg);

            if (executed) return;

            msg = message;
        }

        const encoder = new TextEncoder();
        const writer = await network_controller.create_uni_writer();

        if (writer === null) {
            console.error("Failed to send chat message");
            return;
        }

        await writer.write(encoder.encode(`CHAT${msg}`));
    }
}

const module = new ChatModule();
network_controller.register_module(module);
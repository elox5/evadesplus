import { AutocompleteMatch, get_autocomplete } from "./autocomplete.js";
import { BinaryReader } from "./binary_reader.js";
import { try_execute_command, try_get_command } from "./commands.js";
import { network_controller, NetworkModule } from "./network_controller.js";
import { player_info, PlayerData } from "./player_info.js";

class Chat {
    private messages: ChatMessage[];
    private message_timestamp_queue: number[];

    private list: HTMLDivElement;
    private input: HTMLInputElement;
    private autocomplete_display: HTMLDivElement;

    private context_menu: HTMLDivElement;

    settings = {
        max_messages: 100,
        auto_reply: false,
    }

    reply_target?: bigint;

    private autocomplete_entries: AutocompleteMatch[] | null = null;
    private autocomplete_index: number = 0;

    constructor() {
        this.messages = [];
        this.message_timestamp_queue = [];

        this.list = document.getElementById("chat-list") as HTMLDivElement;
        this.input = document.getElementById("chat-input") as HTMLInputElement;
        const send_button = document.getElementById("chat-send-button") as HTMLButtonElement;

        this.autocomplete_display = document.getElementById("chat-autocomplete") as HTMLDivElement;
        this.context_menu = document.getElementById("context-menu") as HTMLDivElement;

        send_button.onclick = () => {
            this.send_message(this.input.value);
        };

        this.input.onkeydown = (e) => {
            if (e.key === "Enter") {
                if (this.autocomplete_entries !== null) {
                    const command = this.input.value.substring(1).split(" ")[0];
                    const command_arg_count = try_get_command(command)?.usage?.length ?? 0;

                    const should_send = this.fill_autocomplete(this.autocomplete_entries[this.autocomplete_index].value, command_arg_count);
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
                const command = this.input.value.substring(1).split(" ")[0];
                const command_arg_count = try_get_command(command)?.usage?.length ?? 0;

                const entries = get_autocomplete(this.input.value);

                if (entries === null || entries.length === 0) {
                    this.hide_autocomplete();
                }
                else {
                    this.show_autocomplete(entries, command_arg_count);
                }
            } else {
                this.hide_autocomplete();
            }
        }
    }

    send_message(message: string) {
        if (this.message_timestamp_queue.length === 10) {
            const tenMessagesTime = Date.now() - this.message_timestamp_queue[0];

            if (tenMessagesTime < 10000) {
                this.receive_message(`Please wait ${(10 - tenMessagesTime / 1000).toFixed(1)} seconds before sending another message.`, null, 2);
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

    send_message_raw(message: string) {
        module.send_message_raw(message);
    }

    receive_message(message: string, sender_id: bigint | null, message_type: MessageType, properties?: ChatMessageProperties) {
        const atBottom = Math.abs(this.list.scrollHeight - this.list.clientHeight - this.list.scrollTop) < 1

        const entry = document.createElement("div");
        entry.classList.add("chat-entry");

        if (message_type === MessageType.Normal && sender_id !== null) {
            entry.appendChild(player_info.get_player_name_span(sender_id));
            entry.appendChild(document.createTextNode(": "));
        }
        if (message_type === MessageType.Whisper && sender_id !== null && properties !== undefined && properties.target_id !== undefined) {
            entry.appendChild(player_info.get_player_name_span(sender_id));
            entry.appendChild(document.createTextNode(" -> "));

            entry.appendChild(player_info.get_player_name_span(properties.target_id));
            entry.appendChild(document.createTextNode(": "));
        }

        message = message.replace(/\n/g, "\r\n");
        message = message.replace(/\*([^*]*)\*/g, "<strong>$1</strong>");

        entry.appendChild(document.createTextNode(message));

        if (message_type === MessageType.Normal) entry.classList.add("normal");
        if (message_type === MessageType.Whisper) entry.classList.add("special", "whisper");
        if (message_type === MessageType.CommandResponse) entry.classList.add("special", "command-response");
        if (message_type === MessageType.ServerAnnouncement) entry.classList.add("special", "server-announcement");
        if (message_type === MessageType.ServerError) entry.classList.add("special", "server-error");

        if (message_type === MessageType.Whisper && sender_id !== null && sender_id !== player_info.get_self_id()) {
            this.reply_target = sender_id;
        }

        this.list.appendChild(entry);

        this.messages.push({
            sender_id,
            element: entry,
        });

        if (this.messages.length > this.settings.max_messages) {
            const removed_message = this.messages.shift();
            this.list.removeChild(removed_message!.element);
        }

        if (atBottom) {
            this.list.scrollTo(0, this.list.scrollHeight);
        }
    }

    mock_server_announcement(message: string) {
        this.receive_message(message, null, MessageType.ServerAnnouncement);
    }

    mock_server_response(message: string) {
        this.receive_message(message, null, MessageType.CommandResponse);
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

    private show_autocomplete(entries: AutocompleteMatch[], arg_count: number) {
        this.autocomplete_entries = entries;

        this.autocomplete_display.classList.remove("hidden");

        this.autocomplete_display.innerHTML = "";
        for (const entry of entries) {
            const button = document.createElement("button");
            button.classList.add("autocomplete-entry");

            button.textContent = entry.name;
            button.onclick = () => {
                this.fill_autocomplete(entry.value, arg_count);
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

    private fill_autocomplete(entry: string, arg_count: number): boolean {
        if (entry === "") {
            this.hide_autocomplete();
            return true;
        }

        const words = this.input.value.split(" ");
        const last_word = words[words.length - 1];

        if (last_word.replace("/", "") === entry && words.length >= arg_count) {
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


    private show_name_context_menu(player_id: bigint, e: MouseEvent) {
        const player = player_info.get_player(player_id);

        if (player === null) {
            return;
        }

        this.context_menu.innerHTML = "";

        const header = document.createElement("h3");
        header.classList.add("context-menu-header");
        header.textContent = player.name;
        this.context_menu.appendChild(header);

        const copy_name_button = document.createElement("button");
        copy_name_button.classList.add("context-menu-button");
        copy_name_button.textContent = "Copy Name";
        copy_name_button.onclick = () => {
            navigator.clipboard.writeText(player.name);
            this.hide_context_menu();
        }
        this.context_menu.appendChild(copy_name_button);

        const copy_id_button = document.createElement("button");
        copy_id_button.classList.add("context-menu-button");
        copy_id_button.textContent = "Copy Player ID";
        copy_id_button.onclick = () => {
            navigator.clipboard.writeText(`@${player_id}`);
            this.hide_context_menu();
        }
        this.context_menu.appendChild(copy_id_button);

        this.context_menu.style.left = `${e.clientX}px`;
        this.context_menu.style.top = `${e.clientY}px`;
        this.context_menu.classList.remove("hidden");
    }

    private hide_context_menu() {
        this.context_menu.classList.add("hidden");
    }
}

type ChatMessage = {
    sender_id: bigint | null;
    element: HTMLElement;
}

type ChatMessageProperties = {
    target_id?: bigint
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
    uni_handlers = [
        { header: "CHBR", callback: this.handle_broadcast.bind(this) }
    ]

    async cleanup() {
        chat.clear();
    }

    private handle_broadcast(data: BinaryReader) {
        const message_type = data.read_u8() as MessageType;
        const sender_id = data.read_u64();
        const message = data.read_length_u8_string()!;

        let properties: ChatMessageProperties | undefined = undefined;
        if (message_type === MessageType.Whisper) {
            properties = { target_id: data.read_u64() };
        }

        chat.receive_message(message, sender_id, message_type, properties);
    }

    async send_chat_message(msg: string) {
        if (chat.settings.auto_reply
            && chat.reply_target !== undefined
            && player_info.player_exists(chat.reply_target)
            && !msg.startsWith("/")
        ) {
            msg = `/reply ${msg}`;
        }

        if (msg.startsWith("/")) {
            const executed = try_execute_command(msg);
            if (executed) return;
        }

        await this.send_message_raw(msg);
    }

    async send_message_raw(msg: string) {
        const encoder = new TextEncoder();
        const message = encoder.encode(`CHAT${msg}`);

        const writer = await network_controller.create_uni_writer();

        if (writer === null) {
            console.error("Failed to create network writer");
            return;
        }

        await writer.write(message);
    }
}

const module = new ChatModule();
network_controller.register_module(module);
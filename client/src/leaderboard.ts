import { BinaryReader } from "./binary_reader.js";
import { cache } from "./cache.js";
import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";

class Leaderboard {
    private maps: Map<string, LeaderboardMap>;
    private element: HTMLElement;

    constructor() {
        this.element = document.getElementById("leaderboard") as HTMLElement;

        this.maps = new Map();
    }

    add(player_id: bigint, player_name: string, area_order: number, area_name: string, map_id: string, downed: boolean) {
        if (!this.maps.get(map_id)) {
            const map = new LeaderboardMap(map_id);

            this.maps.set(map_id, map);
            this.element.appendChild(map.element);
        }

        this.maps.get(map_id)!.add(player_id, player_name, area_order, area_name, downed);

        cache.current_players.push({
            player_id,
            player_name,
            map_id
        });
    }

    remove(player_id: bigint) {
        for (const map of this.maps.values()) {
            map.remove(player_id);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                this.maps.delete(map.id);
            }
        }

        cache.current_players.splice(cache.current_players.findIndex(entry => entry.player_id === player_id), 1);
    }

    transfer(player_id: bigint, area_order: number, area_name: string, map_id: string) {
        for (const map of this.maps.values()) {
            let old_entry_index = map.entries.findIndex(entry => entry.player_id === player_id);

            if (old_entry_index === -1) {
                continue;
            }

            let old_entry = map.entries[old_entry_index];
            map.remove(player_id);

            this.add(player_id, old_entry.player_name, area_order, area_name, map_id, old_entry.downed);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                this.maps.delete(map.id);
            }
        }

        cache.current_players.find(entry => entry.player_id === player_id)!.map_id = map_id;
    }

    set_downed(player_id: bigint, downed: boolean) {
        for (const map of this.maps.values()) {
            map.set_downed(player_id, downed);
        }
    }

    clear() {
        this.maps.clear();
        this.element.innerHTML = "";
    }
}

class LeaderboardMap {
    id: string;
    entries: LeaderboardEntry[];

    element: HTMLElement;
    private list: HTMLElement;

    constructor(id: string) {
        const name = cache.maps.find(map => map.id === id)!.name;

        this.id = id;

        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-map");

        this.list = document.createElement("div");
        this.list.classList.add("leaderboard-map-list");

        const header = document.createElement("h3");
        header.classList.add("leaderboard-map-name");
        header.textContent = name;

        this.element.appendChild(header);
        this.element.appendChild(this.list);

        this.entries = [];
    }

    add(player_id: bigint, player_name: string, area_order: number, area_name: string, downed: boolean) {
        this.entries.push(new LeaderboardEntry(player_id, player_name, area_order, area_name, downed));
        this.entries.sort((a, b) => b.area_order - a.area_order);

        this.update_list();
    }

    remove(player_id: bigint) {
        this.entries = this.entries.filter(entry => entry.player_id !== player_id);

        this.update_list();
    }

    set_downed(player_id: bigint, downed: boolean) {
        for (const entry of this.entries) {
            if (entry.player_id === player_id) {
                entry.element.classList.toggle("downed", downed);
                break;
            }
        }
    }

    update_list() {
        this.list.textContent = "";
        for (const entry of this.entries) {
            this.list.appendChild(entry.element);
        }
    }
}

class LeaderboardEntry {
    player_id: bigint;
    player_name: string;
    downed: boolean;

    area_order: number;

    element: HTMLElement;

    constructor(player_id: bigint, player_name: string, area_order: number, area_name: string, downed: boolean) {
        this.player_id = player_id;
        this.player_name = player_name;
        this.area_order = area_order;
        this.downed = downed;

        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-entry");

        if (downed) {
            this.element.classList.add("downed");
        }

        const nameDiv = document.createElement("div");
        nameDiv.classList.add("leaderboard-entry-name");
        nameDiv.textContent = player_name;
        this.element.appendChild(nameDiv);

        const areaDiv = document.createElement("div");
        areaDiv.classList.add("leaderboard-entry-area");
        areaDiv.textContent = `${area_name}`;
        this.element.appendChild(areaDiv);
    }
}

const leaderboard = new Leaderboard();

export class LeaderboadModule implements NetworkModule {

    uni_handlers = [
        { header: "LBAD", callback: this.handle_add.bind(this) },
        { header: "LBRM", callback: this.handle_remove.bind(this) },
        { header: "LBTR", callback: this.handle_transfer.bind(this) },
        { header: "LBSD", callback: this.handle_set_downed.bind(this) },
    ];

    init = {
        callback: (data: BinaryReader) => {
            const entry_count = data.read_u8();

            for (let i = 0; i < entry_count; i++) {
                const player_id = data.read_u64();
                const area_order = data.read_u16();
                const [downed] = data.read_flags();

                const player_name = data.read_length_u8_string()!;
                const area_name = data.read_length_u8_string()!;
                const map_id = data.read_length_u8_string()!;

                leaderboard.add(player_id, player_name, area_order, area_name, map_id, downed);
            }
        },
        order: 0,
    };

    cleanup() {
        leaderboard.clear();
    }

    private handle_add(data: BinaryReader) {
        const player_id = data.read_u64();
        const area_order = data.read_u16();
        const [downed] = data.read_flags();

        const player_name = data.read_length_u8_string()!;
        const area_name = data.read_length_u8_string()!;
        const map_id = data.read_length_u8_string()!;

        leaderboard.add(player_id, player_name, area_order, area_name, map_id, downed);
    }

    private handle_remove(data: BinaryReader) {
        const player_id = data.read_u64();
        leaderboard.remove(player_id);
    }

    private handle_transfer(data: BinaryReader) {
        const player_id = data.read_u64();
        const area_order = data.read_u16();

        const area_name = data.read_length_u8_string()!;
        const map_id = data.read_length_u8_string()!;

        leaderboard.transfer(player_id, area_order, area_name, map_id);
    }

    private handle_set_downed(data: BinaryReader) {
        const player_id = data.read_u64();
        const [downed] = data.read_flags();

        leaderboard.set_downed(player_id, downed);
    }
}

network_controller.register_module(new LeaderboadModule());
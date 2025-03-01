import { cache } from "./cache.js";
import { network_controller, NetworkModule } from "./network_controller.js";
import { player_info, PlayerData } from "./player_info.js";

class Leaderboard implements NetworkModule {
    private maps: Map<string, LeaderboardMap>;
    private element: HTMLElement;

    constructor() {
        this.element = document.getElementById("leaderboard") as HTMLElement;

        this.maps = new Map();

        player_info.on_player_add.push(this.add.bind(this));
        player_info.on_player_remove.push(this.remove.bind(this));
        player_info.on_player_transfer.push(this.transfer.bind(this));
        player_info.on_player_set_downed.push(this.set_downed.bind(this));
    }

    private add(player: PlayerData) {
        if (!this.maps.get(player.location_info.map_id)) {
            const map = new LeaderboardMap(player.location_info.map_id);

            this.maps.set(player.location_info.map_id, map);
            this.element.appendChild(map.element);
        }

        this.maps.get(player.location_info.map_id)!.add(player);
    }

    private remove(player: PlayerData) {
        for (const map of this.maps.values()) {
            map.remove(player.id);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                this.maps.delete(map.id);
            }
        }
    }

    private transfer(player: PlayerData) {
        for (const map of this.maps.values()) {
            let old_entry_index = map.entries.findIndex(entry => entry.id === player.id);

            if (old_entry_index === -1) {
                continue;
            }

            map.remove(player.id);
            this.add(player);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                this.maps.delete(map.id);
            }
        }
    }

    private set_downed(player: PlayerData) {
        for (const map of this.maps.values()) {
            map.set_downed(player.id, player.downed);
        }
    }

    // NetworkModule implementation

    cleanup() {
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
        const map = cache.maps.find(map => map.id === id);

        if (map === undefined) return;

        this.id = id;

        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-map");

        this.list = document.createElement("div");
        this.list.classList.add("leaderboard-map-list");

        const header = document.createElement("h3");
        header.classList.add("leaderboard-map-name");
        header.textContent = map.name;
        header.style.color = map.text_color;

        this.element.appendChild(header);
        this.element.appendChild(this.list);

        this.entries = [];
    }

    add(player: PlayerData) {
        this.entries.push(new LeaderboardEntry(player));
        this.entries.sort((a, b) => b.order - a.order);

        this.update_list();
    }

    remove(player_id: bigint) {
        this.entries = this.entries.filter(entry => entry.id !== player_id);

        this.update_list();
    }

    set_downed(player_id: bigint, downed: boolean) {
        for (const entry of this.entries) {
            if (entry.id === player_id) {
                entry.element.classList.toggle("downed", downed);
                break;
            }
        }
    }

    private update_list() {
        this.list.textContent = "";
        for (const entry of this.entries) {
            this.list.appendChild(entry.element);
        }
    }
}

class LeaderboardEntry {
    id: bigint;
    order: number;
    element: HTMLDivElement;

    constructor(player: PlayerData) {
        this.id = player.id;
        this.order = player.location_info.area_order;

        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-entry");

        if (player.downed) {
            this.element.classList.add("downed");
        }

        const nameDiv = document.createElement("div");
        nameDiv.classList.add("leaderboard-entry-name");
        nameDiv.appendChild(player_info.get_player_name_span(player.id));
        this.element.appendChild(nameDiv);

        const areaDiv = document.createElement("div");
        areaDiv.classList.add("leaderboard-entry-area");
        areaDiv.textContent = player?.location_info.area_name ?? "Unknown Area";
        areaDiv.style.color = player.location_info.area_color ?? cache.maps.find(m => m.id === player.location_info.map_id)!.text_color;

        this.element.appendChild(areaDiv);
    }
}

const leaderboard = new Leaderboard();

network_controller.register_module(leaderboard);
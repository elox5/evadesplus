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

    // Leaderboard action handlers

    private add(player: PlayerData) {
        if (!this.maps.get(player.area_info.map_id)) {
            const map = new LeaderboardMap(player.area_info.map_id);

            this.add_map(map);

            this.redraw();
        }

        this.maps.get(player.area_info.map_id)!.add(player);
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

    // Helpers

    private add_map(map: LeaderboardMap) {
        this.maps.set(map.id, map);
        this.order_maps();
    }

    private order_maps() {
        this.maps = new Map(
            [...this.maps.entries()].sort((a, b) => this.compare_map_names(a[1], b[1]))
        )
    }

    private compare_map_names(a: LeaderboardMap, b: LeaderboardMap): number {
        const a_name = cache.maps.find(m => m.id === a.id)!.name;
        const b_name = cache.maps.find(m => m.id === b.id)!.name;

        return a_name.localeCompare(b_name);
    }

    private redraw() {
        this.element.innerHTML = "";

        for (const map of this.maps.values()) {
            this.element.append(map.element);
        }

        const self_id = player_info.get_self_id();

        if (self_id !== null) {
            const self = player_info.get_player(self_id);

            if (self !== null) {
                const current_map = this.maps.get(self.area_info.map_id);

                if (current_map) {
                    this.element.insertBefore(current_map.element, this.element.firstChild);
                }
            }
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

    element: HTMLDivElement;
    private list: HTMLDivElement;

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
        this.order = player.area_info.area_order;

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

        let area_name = player.area_info.area_name ?? "Unknown Area";
        if (player.area_info.victory) {
            area_name = `Victory! ${area_name}`;
        }

        areaDiv.textContent = area_name;
        areaDiv.style.color = player.area_info.area_color ?? cache.maps.find(m => m.id === player.area_info.map_id)!.text_color;

        this.element.appendChild(areaDiv);
    }
}

const leaderboard = new Leaderboard();

network_controller.register_module(leaderboard);
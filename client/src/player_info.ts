import { BinaryReader } from "./binary_reader.js";
import { cache } from "./cache.js";
import { network_controller, NetworkModule } from "./network_controller.js";

export const player_info = {
    players: new Array<PlayerData>(),
    self_id: null as bigint | null,
}

type PlayerData = {
    id: bigint,
    name: string,
    map_id: string,
};

export function get_player_name_span(id: bigint): HTMLSpanElement {
    const player = player_info.players.find(p => p.id === id)!;
    const map = cache.maps.find(m => m.id === player.map_id)!;
    const map_color = map.text_color;

    const span = document.createElement("span");
    span.style.color = map_color;
    span.textContent = player.name;

    return span;
}

class PlayerInfoModule implements NetworkModule {
    cleanup() {
        player_info.players = [];
        player_info.self_id = null;
    }

    init = {
        order: 0,
        callback: (data: BinaryReader) => {
            player_info.self_id = data.read_u64();
        }
    }
}

network_controller.register_module(new PlayerInfoModule());
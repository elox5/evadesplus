import { BinaryReader } from "./binary_reader.js";
import { cache } from "./cache.js";
import { network_controller, NetworkModule } from "./network_controller.js";

class PlayerInfo implements NetworkModule {
    private players: PlayerData[];
    private self_id: bigint | null;

    public on_player_add: ((player: PlayerData) => void)[] = [];
    public on_player_remove: ((player: PlayerData) => void)[] = [];
    public on_player_transfer: ((player: PlayerData) => void)[] = [];
    public on_player_set_downed: ((player: PlayerData) => void)[] = [];

    constructor() {
        this.players = [];
        this.self_id = null;
    }

    get_self_id(): bigint | null {
        return player_info.self_id;
    }

    get_player(id: bigint): PlayerData | null {
        return player_info.players.find(p => p.id === id) || null;
    }

    player_exists(id: bigint): boolean {
        return player_info.players.some(p => p.id === id);
    }

    all_players(): PlayerData[] {
        return player_info.players;
    }

    get_player_name_span(id: bigint): HTMLSpanElement {
        const player = player_info.players.find(p => p.id === id)!;
        const map = cache.maps.find(m => m.id === player.area_info.map_id)!;
        const map_color = map.text_color;

        const span = document.createElement("span");
        span.style.color = map_color;
        span.textContent = player.name;

        return span;
    }

    // Handlers

    private handle_add(data: BinaryReader) {
        const player_id = data.read_u64();
        const player_name = data.read_length_u8_string()!;
        const [downed] = data.read_flags();

        const area_info = this.parse_area_info(data);

        player_info.players.push({
            id: player_id,
            name: player_name,
            area_info,
            downed,
        });

        for (const handler of this.on_player_add) {
            handler(player_info.players[player_info.players.length - 1]);
        }
    }

    private handle_remove(data: BinaryReader) {
        const player_id = data.read_u64();

        const [removed_player] = player_info.players.splice(
            player_info.players.findIndex(p => p.id === player_id), 1
        );

        for (const handler of this.on_player_remove) {
            handler(removed_player);
        }
    }

    private handle_transfer(data: BinaryReader) {
        const player_id = data.read_u64();
        const player = this.get_player(player_id)!;

        player.area_info = this.parse_area_info(data);

        for (const handler of this.on_player_transfer) {
            handler(player);
        }
    }

    private handle_set_downed(data: BinaryReader) {
        const player_id = data.read_u64();
        const [downed] = data.read_flags();

        const player = this.get_player(player_id)!;
        player.downed = downed;

        for (const handler of this.on_player_set_downed) {
            handler(player);
        }
    }

    // Private helpers

    private parse_area_info(data: BinaryReader): AreaInfo {
        const map_id = data.read_length_u8_string()!;
        const area_name = data.read_length_u8_string()!;
        const area_order = data.read_u16();
        const victory = data.read_bool();

        const has_color = data.read_bool();
        const area_color = has_color ? data.read_string(7) : null;

        return {
            map_id,
            area_name,
            area_order,
            area_color,
            victory,
        };
    }

    // NetworkModule implementation

    uni_handlers = [
        { header: "PADD", callback: this.handle_add.bind(this) },
        { header: "PRMV", callback: this.handle_remove.bind(this) },
        { header: "PTRF", callback: this.handle_transfer.bind(this) },
        { header: "PSDN", callback: this.handle_set_downed.bind(this) },
    ];

    init = {
        callback: (data: BinaryReader) => {
            player_info.self_id = data.read_u64();

            const entry_count = data.read_u8();

            for (let i = 0; i < entry_count; i++) {
                this.handle_add(data);
            }
        },
        order: 0,
    }

    cleanup() {
        player_info.players = [];
        player_info.self_id = null;
    }
}

export type PlayerData = {
    id: bigint,
    name: string,
    area_info: AreaInfo,
    downed: boolean,
};

type AreaInfo = {
    map_id: string,
    area_name: string,
    area_order: number,
    area_color: string | null,
    victory: boolean,
};

export const player_info = new PlayerInfo();

network_controller.register_module(player_info);
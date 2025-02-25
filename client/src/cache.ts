
export const cache = {
    maps: new Array<MapData>(),
    commands: new Array<CommandData>(),
    current_players: new Array<PlayerData>(),
    self_id: null as bigint | null,
}

export async function init_cache() {
    const json = await fetch("/cache").then((response) => response.json());

    cache.maps = json["maps"];
    cache.commands = json["commands"];

    console.log("Cache loaded: ", cache);
}

type MapData = {
    id: string,
    name: string,
    text_color: string,
}

export type CommandData = {
    name: string,
    description: string,
    usage: string,
    aliases: string[],
}

type PlayerData = {
    player_id: bigint,
    player_name: string,
    map_id: string,
};
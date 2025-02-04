
class Cache {
    maps: MapData[] = [];
    commands: CommandData[] = [];
    current_players: PlayerData[] = [];

    async init() {
        const json = await fetch("/cache").then((response) => response.json());

        this.maps = json["maps"];
        this.commands = json["commands"];

        console.log("Cache loaded: ", this);
    }
}

export const cache = new Cache();

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
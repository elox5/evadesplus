
export const cache = {
    maps: new Array<MapData>(),
    commands: new Array<CommandData>(),
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


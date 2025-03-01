
export const cache = {
    maps: new Array<MapData>(),
    commands: new Array<CommandData>(),
}

export async function init_cache() {
    const local_cache_hash = window.localStorage.getItem("ep_cache_hash");
    const current_cache_hash = await fetch("/cache_hash").then((response) => response.text());

    if (local_cache_hash !== current_cache_hash) {
        await fetch_cache();
        window.localStorage.setItem("ep_cache_hash", current_cache_hash);
    }
    else {
        const local_cache = window.localStorage.getItem("ep_cache");

        if (local_cache === null) {
            await fetch_cache();
        }
        else {
            console.log("Cache loaded from local storage.");

            cache.maps = JSON.parse(local_cache)["maps"];
            cache.commands = JSON.parse(local_cache)["commands"];
        }
    }
}

async function fetch_cache() {
    console.log("Cache missing or outdated. Fetching up-to-date cache...");

    const json = await fetch("/cache").then((response) => response.json());

    cache.maps = json["maps"];
    cache.commands = json["commands"];

    console.log("Cache loaded.");
    console.log(cache);

    window.localStorage.setItem("ep_cache", JSON.stringify(cache));
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


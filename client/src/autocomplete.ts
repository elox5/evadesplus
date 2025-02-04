import { try_get_command } from "./commands.js";
import { cache } from "./cache.js";

export type CommandAutocompleteToken = {
    name: string,
    optional: boolean
}

function resolve_token(token: CommandAutocompleteToken, input: string): string[] {
    const matches: string[] = [];

    if (token.optional && input === "") {
        matches.push("");
    }

    if (token.name === "command") {
        for (let command of cache.commands) {
            if (input === undefined || input === "" || (command.name.startsWith(input) && !matches.includes(command.name))) {
                matches.push(command.name);
            }
        }
    } else if (token.name === "player") {
        for (let player of cache.current_players) {
            if (player.player_name.startsWith(input) && !matches.includes(player.player_name)) {
                matches.push(player.player_name);
            }
        }
    } else if (token.name === "map") {
        for (let map of cache.maps) {
            if (map.name.startsWith(input) && !matches.includes(map.name)) {
                matches.push(map.name);
            }
        }
    }

    return matches;
}


export function get_autocomplete(input: string): string[] | null {
    const words = input.substring(1).split(" ");
    const [command, ...args] = words;

    if (args.length === 0) {
        return null;
        // return resolve_token({ name: "command", optional: false }, command);
    }

    const usage = try_get_command(command)?.usage;

    if (usage === "" || usage === undefined || usage === null) {
        return null;
    }

    const tokens: CommandAutocompleteToken[] = usage.split(" ")
        .map(token => token.replace("<", "").replace(">", ""))
        .map(token => {
            const optional = token.endsWith("?");

            if (optional) {
                token = token.substring(0, token.length - 1);
            }

            return {
                name: token,
                optional
            };
        });

    const arg_index = args.length - 1;

    if (arg_index >= tokens.length) {
        return null;
    }

    return resolve_token(tokens[arg_index], args[arg_index]);
}
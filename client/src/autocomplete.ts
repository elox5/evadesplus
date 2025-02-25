import { try_get_command } from "./commands.js";
import { cache } from "./cache.js";

export type CommandAutocompleteToken = {
    name: string,
    optional: boolean
}

export type AutocompleteMatch = {
    name: string,
    value: string
}

function resolve_token(token: CommandAutocompleteToken, input: string): AutocompleteMatch[] {
    const matches: AutocompleteMatch[] = [];

    if (token.optional && input === "") {
        matches.push({ name: "", value: "" });
    }

    if (token.name === "command") {
        for (let command of cache.commands) {
            if (input === undefined || input === "" || (command.name.startsWith(input) && !matches.some(m => m.name == command.name))) {
                matches.push({ name: command.name, value: command.name });
            }
        }
    } else if (token.name === "player") {
        for (let player of cache.current_players) {
            if (player.player_name.startsWith(input) && !matches.some(m => m.name == player.player_name)) {
                matches.push({ name: player.player_name, value: `@${player.player_id}` });
            }
        }
    } else if (token.name === "map") {
        for (let map of cache.maps) {
            if (map.name.startsWith(input) && !matches.some(m => m.value == map.id)) {
                matches.push({ name: map.name, value: map.id });
            }
        }
    }

    return matches;
}


export function get_autocomplete(input: string): AutocompleteMatch[] | null {
    const words = input.substring(1).split(" ");
    const [command, ...args] = words;

    if (args.length === 0) {
        return resolve_token({ name: "command", optional: false }, command);
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
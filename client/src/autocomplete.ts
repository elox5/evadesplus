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

    input = input.toLowerCase();

    if (token.name === "command") {
        let exact_match: AutocompleteMatch | null = null;

        for (const command of cache.commands) {
            if (input === undefined || input === "" || (command.name.toLowerCase().startsWith(input) && !matches.some(m => m.name == command.name))) {
                matches.push({ name: command.name, value: command.name });
            }
            if (command.aliases !== null) {
                for (let alias of command.aliases) {
                    if (alias.toLowerCase() === input && !matches.some(m => m.name == alias)) {
                        exact_match = { name: alias, value: alias };
                    }
                }
            }
        }

        if (exact_match !== null) {
            matches.unshift(exact_match);
        }
    } else if (token.name === "player") {
        for (const player of cache.current_players) {
            if (player.player_name.toLowerCase().startsWith(input) && !matches.some(m => m.name == player.player_name)) {
                matches.push({ name: player.player_name, value: `@${player.player_id}` });
            }
        }
    } else if (token.name === "map") {
        let exact_match: AutocompleteMatch | null = null;

        for (const map of cache.maps) {
            if (map.name.toLowerCase().startsWith(input) && !matches.some(m => m.value == map.id)) {
                matches.push({ name: map.name, value: map.id });
            }
            if (map.id.toLowerCase() === input && !matches.some(m => m.value == map.id)) {
                exact_match = { name: map.name, value: map.id };
            }
        }

        if (exact_match !== null) {
            matches.unshift(exact_match);
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
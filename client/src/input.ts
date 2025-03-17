
const keydown_handlers: ((e: KeyboardEvent) => void)[] = [];
const keyup_handlers: ((e: KeyboardEvent) => void)[] = [];

export function setup_input() {
    window.onkeydown = (e) => {
        for (const handler of keydown_handlers) {
            handler(e);
        }
    }

    window.onkeyup = (e) => {
        for (const handler of keyup_handlers) {
            handler(e);
        }
    }
}

export function register_keydown_handler(handler: (e: KeyboardEvent) => void) {
    keydown_handlers.push(handler);
}

export function register_keyup_handler(handler: (e: KeyboardEvent) => void) {
    keyup_handlers.push(handler);
}
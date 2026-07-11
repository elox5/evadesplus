import { BinaryReader } from "./binary_reader.js";
import { ws_connector, WsModule } from "./ws_connector.js";

const timer_display = document.getElementById("speedrun-timer") as HTMLDivElement;

const timer = {
    start: null as bigint | null,
    interval: null as number | null,
}

function start_timer(start: bigint) {
    timer.start = start;

    update_timer();

    if (timer.interval === null) {
        timer.interval = setInterval(() => {
            update_timer();
        }, 1000);
    }
}

function now(): bigint {
    return BigInt(Math.floor(Date.now() / 1000));
}

function elapsed_seconds(): number | null {
    if (timer.start === null) {
        return null;
    }

    return Number(now() - timer.start);
}

function update_timer() {
    const time = elapsed_seconds();

    if (time === null) return;

    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);

    timer_display.textContent = `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

class TimerModule implements WsModule {
    handlers = [
        {
            header: "TIME",
            callback: (data: BinaryReader) => {
                console.log("TEST");

                const start = data.read_u64();
                start_timer(start);
            }
        },
    ]

    cleanup = () => {
        timer.start = null;
        if (timer.interval !== null) clearInterval(timer.interval);
    }
}

ws_connector.register_module(new TimerModule());
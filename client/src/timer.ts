import { BinaryReader } from "./binary_reader.js";
import { network_controller, NetworkModule } from "./network_controller.js";

const timer_display = document.getElementById("speedrun-timer") as HTMLDivElement;

const timer = {
    time: 0,
    interval: null as number | null,
}

function start_timer() {
    timer.time = 0;
    update_timer();

    timer.interval = setInterval(() => {
        timer.time += 1;
        update_timer();
    }, 1000);
}

export function reset_timer() {
    if (timer.interval !== null) clearInterval(timer.interval);

    start_timer();
}

function update_timer() {
    const minutes = Math.floor(timer.time / 60);
    const seconds = Math.floor(timer.time % 60);

    timer_display.textContent = `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

class TimerModule implements NetworkModule {
    uni_handlers = [
        {
            header: "TIME",
            callback: (data: BinaryReader) => {
                timer.time = data.read_f32();
                update_timer();
            }
        },
    ]

    on_game_load = {
        callback: start_timer,
        once: false
    }

    cleanup = () => {
        timer.time = 0;
        if (timer.interval !== null) clearInterval(timer.interval);
    }
}

network_controller.register_module(new TimerModule());
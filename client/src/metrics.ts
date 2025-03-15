import { BinaryReader } from "./binary_reader.js";
import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";
import { settings } from "./settings.js";

const fps_container = document.getElementById("fps-container") as HTMLDivElement;
const ping_container = document.getElementById("ping-container") as HTMLDivElement;
const bandwidth_container = document.getElementById("bandwidth-container") as HTMLDivElement;
const render_time_container = document.getElementById("render-time-container") as HTMLDivElement;

const fps_meter = document.getElementById("fps-meter") as HTMLSpanElement;
const ping_meter = document.getElementById("ping-meter") as HTMLSpanElement;
const bandwidth_meter = document.getElementById("bandwidth-meter") as HTMLSpanElement;
const render_time_meter = document.getElementById("render-time-meter") as HTMLSpanElement;

type ColorLevel = {
    threshold: number,
    color: string
}

type ColorMap = {
    levels: ColorLevel[], // from worst to best
    lower_is_better: boolean
}

type MetricSettings = {
    render_report_frequency: number;
    fps_report_window: number;
    fps_color_map: ColorMap;
    ping_frequency: number;
    ping_color_levels: ColorMap;
    bandwidth_report_window: number;
}

export const metric_settings: MetricSettings = {
    render_report_frequency: 10,
    fps_report_window: 1000,
    fps_color_map: {
        levels: [
            { threshold: 15, color: "#ff0000" },
            { threshold: 30, color: "#ff7700" },
            { threshold: 45, color: "#ffff00" },
            { threshold: 55, color: "#77ff00" },
        ],
        lower_is_better: false
    },
    ping_frequency: 500,
    ping_color_levels: {
        levels: [
            { threshold: 300, color: "#ff0000" },
            { threshold: 200, color: "#ff7700" },
            { threshold: 100, color: "#ffff00" },
            { threshold: 50, color: "#77ff00" },
        ],
        lower_is_better: true
    },
    bandwidth_report_window: 2000,
}

let render_report_counter: number = 0;
let render_start_time: number;

let frame_time_queue: number[] = [];

let ping_start_time: number;

type BandwidthEntry = {
    time: number,
    bytes: number,
}

let bandwidth_queue: BandwidthEntry[] = [];

const enabled_flags = {
    fps: true,
    ping: true,
    bandwidth: true,
    render_time: true,
}

export function report_render_start() {
    if (!enabled_flags.render_time) return;

    render_report_counter++;

    if (render_report_counter % metric_settings.render_report_frequency === 0) {
        render_start_time = performance.now();
    }
}

export function report_render_end() {
    if (!enabled_flags.render_time) return;

    if (render_report_counter % metric_settings.render_report_frequency === 0) {
        const renderTime = performance.now() - render_start_time;
        render_time_meter.textContent = renderTime.toFixed(2);
    }
}

export function report_frame_start() {
    if (!enabled_flags.fps) return;

    const frameTime = performance.now();

    frame_time_queue.push(frameTime);

    if (frameTime - frame_time_queue[0] > metric_settings.fps_report_window) {
        frame_time_queue.shift();
    }

    let fps = frame_time_queue.length / (performance.now() - frame_time_queue[0]) * 1000;

    fps_meter.textContent = fps.toFixed(0);

    set_meter_color(fps_meter, fps, metric_settings.fps_color_map);
}

export function start_ping() {
    ping_start_time = performance.now();
}

function set_meter_color(meter: HTMLSpanElement, value: number, color_map: ColorMap) {
    for (const { threshold, color } of color_map.levels) {
        if (color_map.lower_is_better ? value >= threshold : value <= threshold) {
            meter.style.color = color;
            break;
        }

        meter.style.color = "#00ff00";
    }
}

export function report_bandwidth(bytes: number) {
    if (!enabled_flags.bandwidth) return;

    const entry = {
        time: performance.now(),
        bytes
    };

    bandwidth_queue.push(entry);

    const time_delta = performance.now() - bandwidth_queue[0].time;

    if (time_delta > metric_settings.bandwidth_report_window) {
        bandwidth_queue.shift();
    }

    let sum_bytes = 0;

    for (const { bytes } of bandwidth_queue) {
        sum_bytes += bytes;
    }

    const sum_bits = sum_bytes * 8;

    bandwidth_meter.textContent = (sum_bits / time_delta).toFixed(0); // (sum_bits / 1000 [kb]) / (time_delta / 1000 [s]) = sum_bits / time_delta
}

export class PingModule implements NetworkModule {
    private interval: number | undefined;

    setup = {
        callback: async (controller: NetworkController) => {

            this.interval = setInterval(async () => {
                if (controller.is_closed()) return;
                if (!enabled_flags.ping) return;

                start_ping();

                const ping_stream = await controller.create_bi_stream();

                const readable = ping_stream.readable;
                const writer = ping_stream.writable.getWriter();

                await writer.write(new TextEncoder().encode("PING"));
                await writer.close();

                const { value } = await readable.getReader().read();
                const stream = new BinaryReader(value);

                if (stream.read_string(4) !== "PONG") {
                    console.error("Invalid ping response");
                    return;
                }

                this.report_ping();
            }, metric_settings.ping_frequency);
        },
        once: false,
    }

    on_game_load = {
        callback: () => {
            settings.bind("metrics.fps_enabled", v => {
                enabled_flags.fps = v;
                fps_container.classList.toggle("hidden", !v);
            });

            settings.bind("metrics.ping_enabled", v => {
                enabled_flags.ping = v;
                ping_container.classList.toggle("hidden", !v);
            });

            settings.bind("metrics.bandwidth_enabled", v => {
                enabled_flags.bandwidth = v;
                bandwidth_container.classList.toggle("hidden", !v);
            });

            settings.bind("metrics.render_time_enabled", v => {
                enabled_flags.render_time = v;
                render_time_container.classList.toggle("hidden", !v);
            });
        },
        once: true,
    };

    cleanup() {
        clearInterval(this.interval);
    }

    private report_ping() {
        const pingTime = performance.now() - ping_start_time;
        ping_meter.textContent = pingTime.toFixed(2);

        set_meter_color(ping_meter, pingTime, metric_settings.ping_color_levels);
    }
}

setTimeout(() => {
    network_controller.register_module(new PingModule());
})
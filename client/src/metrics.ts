import { BinaryReader } from "./binary_reader.js";
import { network_controller, NetworkController, NetworkModule } from "./network_controller.js";

const pingMeter = document.getElementById("ping-meter") as HTMLSpanElement;
const bandwidthMeter = document.getElementById("bandwidth-meter") as HTMLSpanElement;
const fpsMeter = document.getElementById("fps-meter") as HTMLSpanElement;
const renderMsMeter = document.getElementById("render-ms-meter") as HTMLSpanElement;

type ColorLevel = [number, string];

type MetricSettings = {
    render_report_frequency: number;
    fps_report_window: number;
    fps_color_levels: ColorLevel[];
    ping_frequency: number;
    ping_color_levels: ColorLevel[];
    bandwidth_report_window: number;
}

export const metric_settings: MetricSettings = {
    render_report_frequency: 10,
    fps_report_window: 1000,
    fps_color_levels: [
        [15, "#ff0000"],
        [30, "#ff7700"],
        [45, "#ffff00"],
        [55, "#77ff00"],
    ],
    ping_frequency: 500,
    ping_color_levels: [
        [300, "#ff0000"],
        [200, "#ff7700"],
        [100, "#ffff00"],
        [50, "#77ff00"],
    ],
    bandwidth_report_window: 2000,
}

let render_report_counter: number = 0;
let render_start_time: number;

let frame_time_queue: number[] = [];

let ping_start_time: number;

let bandwidth_queue: number[][] = [];

export function report_render_start() {
    render_report_counter++;

    if (render_report_counter % metric_settings.render_report_frequency === 0) {
        render_start_time = performance.now();
    }
}

export function report_render_end() {
    if (render_report_counter % metric_settings.render_report_frequency === 0) {
        const renderTime = performance.now() - render_start_time;
        renderMsMeter.textContent = renderTime.toFixed(2);
    }
}

export function report_frame_start() {
    const frameTime = performance.now();

    frame_time_queue.push(frameTime);

    if (frameTime - frame_time_queue[0] > metric_settings.fps_report_window) {
        frame_time_queue.shift();
    }

    let fps = frame_time_queue.length / (performance.now() - frame_time_queue[0]) * 1000;

    fpsMeter.textContent = fps.toFixed(0);

    set_meter_color(fpsMeter, fps, metric_settings.fps_color_levels, true);
}

export function start_ping() {
    ping_start_time = performance.now();
}

function set_meter_color(meter: HTMLSpanElement, value: number, colorLevels: ColorLevel[], lower: boolean) {
    for (const [threshold, color] of colorLevels) {
        if ((lower && value < threshold) || (!lower && value > threshold)) {
            meter.style.color = color;
            break;
        }

        meter.style.color = "#00ff00";
    }
}

export function report_bandwidth(bandwidth: number) {
    const entry = [performance.now(), bandwidth];
    bandwidth_queue.push(entry);

    const timeDelta = performance.now() - bandwidth_queue[0][0];

    if (timeDelta > metric_settings.bandwidth_report_window) {
        bandwidth_queue.shift();
    }

    let sum = 0;

    for (const [_, bandwidth] of bandwidth_queue) {
        sum += bandwidth;
    }

    bandwidthMeter.textContent = (sum / timeDelta).toFixed(0);
}

export class PingModule implements NetworkModule {
    private interval: number | undefined;

    async register(controller: NetworkController) {

        this.interval = setInterval(async () => {
            if (!controller.is_connected()) return;

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
    }

    cleanup() {
        clearInterval(this.interval);
    }

    private report_ping() {
        const pingTime = performance.now() - ping_start_time;
        pingMeter.textContent = pingTime.toFixed(2);

        set_meter_color(pingMeter, pingTime, metric_settings.ping_color_levels, false);
    }
}

setTimeout(() => {
    network_controller.register_module(new PingModule());
})
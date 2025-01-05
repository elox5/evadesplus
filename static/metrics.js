
const pingMeter = document.getElementById("ping-meter");
const bandwidthMeter = document.getElementById("bandwidth-meter");
const fpsMeter = document.getElementById("fps-meter");
const renderMsMeter = document.getElementById("render-ms-meter");

export const metricSettings = {
    renderReportFrequency: 10,
    fpsReportWindow: 1000,
    fpsColorLevels: [
        [15, "#ff0000"],
        [30, "#ff7700"],
        [45, "#ffff00"],
        [55, "#77ff00"],
    ],
    pingFrequency: 500,
    pingColorLevels: [
        [300, "#ff0000"],
        [200, "#ff7700"],
        [100, "#ffff00"],
        [50, "#77ff00"],
    ],
    bandwidthReportWindow: 2000,
}

let renderReportCounter = 0;
let renderStartTime;

let frameTimeQueue = [];

let pingStartTime;

let bandwidthQueue = [];

export function reportRenderStart() {
    renderReportCounter++;

    if (renderReportCounter % metricSettings.renderReportFrequency === 0) {
        renderStartTime = performance.now();
    }
}

export function reportRenderEnd() {
    if (renderReportCounter % metricSettings.renderReportFrequency === 0) {
        const renderTime = performance.now() - renderStartTime;
        renderMsMeter.textContent = renderTime.toFixed(2);
    }
}

export function reportFrameStart() {
    const frameTime = performance.now();

    frameTimeQueue.push(frameTime);

    if (frameTime - frameTimeQueue[0] > metricSettings.fpsReportWindow) {
        frameTimeQueue.shift();
    }

    let fps = frameTimeQueue.length / (performance.now() - frameTimeQueue[0]) * metricSettings.fpsReportWindow;

    fpsMeter.textContent = fps.toFixed(0);

    setMeterColor(fpsMeter, metricSettings.fpsColorLevels, true);
}

export function startPing() {
    pingStartTime = performance.now();
}

export function reportPing() {
    const pingTime = performance.now() - pingStartTime;
    pingMeter.textContent = pingTime.toFixed(2);

    setMeterColor(pingMeter, metricSettings.pingColorLevels, false);
}

function setMeterColor(meter, colorLevels, lower) {
    for (const [threshold, color] of colorLevels) {
        if ((lower && meter < threshold) || (!lower && meter > threshold)) {
            meter.style.color = color;
            break;
        }

        meter.style.color = "#00ff00";
    }
}

export function reportBandwidth(bandwidth) {
    const entry = [performance.now(), bandwidth];
    bandwidthQueue.push(entry);

    const timeDelta = performance.now() - bandwidthQueue[0][0];

    if (timeDelta > metricSettings.bandwidthReportWindow) {
        bandwidthQueue.shift();
    }

    let sum = 0;

    for (const [_, bandwidth] of bandwidthQueue) {
        sum += bandwidth;
    }

    bandwidthMeter.textContent = (sum / timeDelta * metricSettings.bandwidthReportWindow / 1000).toFixed(0);
}
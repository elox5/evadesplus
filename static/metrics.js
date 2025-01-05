
const pingMeter = document.getElementById("ping-meter");
const fpsMeter = document.getElementById("fps-meter");
const renderMsMeter = document.getElementById("render-ms-meter");

const settings = {
    renderReportFrequency: 10,
    fpsReportWindow: 1000,
    fpsColorLevels: [
        [15, "#ff0000"],
        [30, "#ff7700"],
        [45, "#ffff00"],
        [55, "#77ff00"],
    ],
}

let renderReportCounter = 0;
let renderStartTime;

let frameTimeQueue = [];

export function reportRenderStart() {
    renderReportCounter++;

    if (renderReportCounter % settings.renderReportFrequency === 0) {
        renderStartTime = performance.now();
    }
}

export function reportRenderEnd() {
    if (renderReportCounter % settings.renderReportFrequency === 0) {
        const renderTime = performance.now() - renderStartTime;
        renderMsMeter.textContent = renderTime.toFixed(2);
    }
}

export function reportFrameStart() {
    const frameTime = performance.now();

    frameTimeQueue.push(frameTime);

    if (frameTime - frameTimeQueue[0] > settings.fpsReportWindow) {
        frameTimeQueue.shift();
    }

    let fps = frameTimeQueue.length / (performance.now() - frameTimeQueue[0]) * settings.fpsReportWindow;

    fpsMeter.textContent = fps.toFixed(0);

    for (const [threshold, color] of settings.fpsColorLevels) {
        if (fps < threshold) {
            fpsMeter.style.color = color;
            break;
        }

        fpsMeter.style.color = "#00ff00";
    }
}

export default class Canvas {
    constructor(id) {
        this.canvas = document.getElementById(id);
        this.ctx = this.canvas.getContext("2d");
        this.updateDimensions();
        this.renderOffset = { x: 0, y: 0 };
        this.tileSize = 40;
    }

    clear() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    }

    gameToCanvasPos(x, y) {
        return {
            x: (x - this.renderOffset.x) * this.tileSize,
            y: this.canvas.height - (y + this.renderOffset.y) * this.tileSize,
        }
    }

    gameToCanvas(a) {
        return a * this.tileSize;
    }

    setDimensions(w, h) {
        this.canvas.width = w;
        this.canvas.height = h;

        this.canvas.style.width = `${w}px`;
        this.canvas.style.height = `${h}px`;
    }

    updateDimensions() {
        this.canvas.width = this.canvas.clientWidth;
        this.canvas.height = this.canvas.clientHeight;
    }

    setRenderOffset(x, y) {
        this.renderOffset = { x, y };
    }

    setPhysicalOffset(x, y) {
        const xOffset = -x * this.tileSize + this.canvas.width / 2;
        const yOffset = y * this.tileSize - this.canvas.height / 2;

        this.canvas.style.translate = `${xOffset}px ${yOffset}px`;
    }
}
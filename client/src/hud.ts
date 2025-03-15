import { settings } from "./settings.js";

const minimap = document.querySelector("#minimap-container") as HTMLDivElement;
const leaderboard = document.querySelector("#leaderboard") as HTMLDivElement;
const chat = document.querySelector("#chat") as HTMLDivElement;
const metrics = document.querySelector("#metrics-container") as HTMLDivElement;

settings.bind("hud.minimap", v => minimap.classList.toggle("hidden", !v));
settings.bind("hud.leaderboard", v => leaderboard.classList.toggle("hidden", !v));
settings.bind("hud.chat", v => chat.classList.toggle("hidden", !v));
settings.bind("hud.metrics", v => metrics.classList.toggle("hidden", !v));
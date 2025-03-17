import { settings } from "./settings.js";

const hud = document.querySelector("#hud") as HTMLDivElement;
const minimap = document.querySelector("#minimap-container") as HTMLDivElement;
const leaderboard = document.querySelector("#leaderboard") as HTMLDivElement;
const chat = document.querySelector("#chat") as HTMLDivElement;
const metrics = document.querySelector("#metrics-container") as HTMLDivElement;

settings.bind("hud.show_minimap", v => minimap.classList.toggle("hidden", !v));
settings.bind("hud.show_leaderboard", v => leaderboard.classList.toggle("hidden", !v));
settings.bind("hud.show_chat", v => chat.classList.toggle("hidden", !v));
settings.bind("hud.show_metrics", v => metrics.classList.toggle("hidden", !v));
settings.bind("hud.panel_blur", v => hud.classList.toggle("panel-blur", v));
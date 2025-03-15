

class Settings {
    private settings: Setting[] = [];
    private panel: HTMLDivElement;

    constructor(sections: Section[]) {
        for (const section of sections) {
            for (const setting of section.settings) {
                setting.id = `${section.id}.${setting.id}`;
            }
        }

        this.settings = sections.flatMap(s => s.settings);

        this.panel = document.querySelector("#settings-panel") as HTMLDivElement;

        const header = document.createElement("h1");
        header.textContent = "Settings";
        this.panel.appendChild(header);

        for (const section of sections) {
            const section_element = document.createElement("section");
            section_element.classList.add("settings-section");

            const header = document.createElement("h2");
            header.textContent = section.name;

            section_element.appendChild(header);

            for (const setting of section.settings) {
                const element = document.createElement("div");
                element.classList.add("settings-entry");

                const name = document.createElement("span");
                name.textContent = setting.name;
                element.appendChild(name);

                if (typeof setting.value === "boolean") {
                    const checkbox = document.createElement("input");
                    checkbox.type = "checkbox";
                    checkbox.checked = setting.value;

                    checkbox.onchange = () => {
                        setting.value = checkbox.checked;

                        if (setting.handlers !== undefined) {
                            for (const handler of setting.handlers) {
                                handler(setting.value);
                            }
                        }
                    }

                    element.appendChild(checkbox);
                }
                else if (typeof setting.value === "number") {
                    const slider = document.createElement("input");
                    slider.type = "range";
                    slider.min = setting.options?.min?.toString() ?? "0";
                    slider.max = setting.options?.max?.toString() ?? "100";
                    slider.step = setting.options?.slider_step?.toString() ?? "1";
                    slider.value = setting.value.toString();

                    slider.oninput = () => {
                        setting.value = parseFloat(slider.value);

                        if (setting.handlers !== undefined) {
                            for (const handler of setting.handlers) {
                                handler(setting.value);
                            }
                        }
                    }

                    element.appendChild(slider);
                }
                else {
                    const value = document.createElement("span");
                    value.textContent = setting.value.toString();
                    element.appendChild(value);
                }

                section_element.appendChild(element);
            }

            this.panel.appendChild(section_element);
        }
    }

    private get_setting(id: string): Setting {
        const setting = this.settings.find(s => s.id === id);

        if (setting === undefined) {
            throw new Error(`Setting ${id} not found`);
        }

        return setting;
    }

    get<T>(id: string): T {
        return this.get_setting(id).value as T;
    }

    bind(id: string, handler: (v: any) => void) {
        const setting = this.get_setting(id);

        if (setting.handlers === undefined) {
            setting.handlers = [handler];
        } else {
            setting.handlers.push(handler);
        }

        handler(setting.value);
    }
}

type Section = {
    id: string,
    name: string,
    settings: Setting[],
}

type Setting = {
    id: string,
    name: string,
    type: "boolean" | "number",
    value: any,
    hotkey?: string,
    handlers?: ((v: any) => void)[],
    options?: {
        min?: number,
        max?: number,
        slider_step?: number
    }
}

export const settings = new Settings([
    {
        id: "gameplay",
        name: "Gameplay",
        settings: [
            {
                id: "mouse_input_range",
                name: "Mouse Input Range",
                type: "number",
                value: 4,
                options: {
                    min: 1, max: 8,
                    slider_step: 0.1
                }
            },
        ],
    },
    {
        id: "visual",
        name: "Visual",
        settings: [
            {
                id: "input_overlay",
                name: "Show Input Overlay",
                type: "boolean",
                value: false,
                hotkey: "i",
            },
            {
                id: "downed_radar.enabled",
                name: "Show Downed Radar",
                type: "boolean",
                value: true,
            },
            {
                id: "downed_radar.distance",
                name: "Downed Radar Distance",
                type: "number",
                value: 7,
                options: {
                    min: 1, max: 10,
                    slider_step: 0.1
                }
            },
            {
                id: "downed_radar.cutoff_distance",
                name: "Downed Radar Cutoff Distance",
                type: "number",
                value: 10,
                options: {
                    min: 5, max: 20,
                    slider_step: 0.5
                }
            },
            {
                id: "downed_radar.opacity",
                name: "Downed Radar Opacity",
                type: "number",
                value: 0.5,
                options: {
                    min: 0.2, max: 1,
                    slider_step: 0.01
                }
            },
            {
                id: "downed_radar.size",
                name: "Downed Radar Size",
                type: "number",
                value: 0.7,
                options: {
                    min: 0.2, max: 2,
                    slider_step: 0.1
                }
            },
        ],
    },
    {
        id: "hud",
        name: "HUD",
        settings: [
            { id: "minimap", name: "Show Minimap", type: "boolean", value: true, hotkey: "m" },
            { id: "leaderboard", name: "Show Leaderboard", type: "boolean", value: true, hotkey: "b" },
            { id: "chat", name: "Show Chat", type: "boolean", value: true, hotkey: "v" },
            { id: "metrics", name: "Show Metrics", type: "boolean", value: true, hotkey: "n" },
        ],
    },
    {
        id: "metrics",
        name: "Metrics",
        settings: [
            { id: "fps_enabled", name: "Show FPS", type: "boolean", value: true },
            { id: "ping_enabled", name: "Show Ping", type: "boolean", value: true },
            { id: "bandwidth_enabled", name: "Show Bandwidth", type: "boolean", value: true },
            { id: "render_time_enabled", name: "Show Render Time", type: "boolean", value: false },
        ],
    },
]);


class Settings {
    private settings: Setting[] = [];
    private panel: HTMLDivElement;

    constructor(sections: Section[]) {
        for (const section of sections) {
            for (const setting of section.settings) {
                setting.id = `${section.id}.${setting.id}`;
                setting.value = setting.default_value;
            }
        }

        this.settings = sections.flatMap(s => s.settings);

        const saved_settings = this.load();

        if (saved_settings !== null) {
            this.settings = this.settings.map(setting => {
                const saved = saved_settings.find(s => s.id === setting.id);

                if (saved !== undefined) {
                    setting.value = saved.value;
                }

                return setting;
            })
        }

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
                name.textContent = `${setting.name}:`;
                element.appendChild(name);

                if (typeof setting.value === "boolean") {
                    const checkbox = document.createElement("input");
                    checkbox.type = "checkbox";
                    checkbox.defaultChecked = setting.default_value;
                    checkbox.checked = setting.value;

                    checkbox.onchange = () => {
                        this.update_setting(setting, checkbox.checked);
                    }

                    element.appendChild(checkbox);
                }
                else if (typeof setting.value === "number") {
                    const value_display = document.createElement("span");
                    value_display.classList.add("settings-entry-value");
                    value_display.textContent = setting.value.toFixed(2);
                    element.appendChild(value_display);

                    const slider = document.createElement("input");
                    slider.type = "range";
                    slider.min = setting.options?.min?.toString() ?? "0";
                    slider.max = setting.options?.max?.toString() ?? "100";
                    slider.step = setting.options?.slider_step?.toString() ?? "1";
                    slider.defaultValue = setting.default_value.toFixed(2);
                    slider.value = setting.value.toFixed(2);

                    slider.oninput = () => {
                        const value = parseFloat(slider.value);

                        this.update_setting(setting, value);
                        value_display.textContent = value.toFixed(2);
                    }

                    element.appendChild(slider);

                    const reset_button = document.createElement("button");
                    reset_button.classList.add("settings-entry-reset-button");
                    reset_button.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--!Font Awesome Free 6.7.2 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free Copyright 2025 Fonticons, Inc.--><path fill="#dddddd" d="M125.7 160l50.3 0c17.7 0 32 14.3 32 32s-14.3 32-32 32L48 224c-17.7 0-32-14.3-32-32L16 64c0-17.7 14.3-32 32-32s32 14.3 32 32l0 51.2L97.6 97.6c87.5-87.5 229.3-87.5 316.8 0s87.5 229.3 0 316.8s-229.3 87.5-316.8 0c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0c62.5 62.5 163.8 62.5 226.3 0s62.5-163.8 0-226.3s-163.8-62.5-226.3 0L125.7 160z"/></svg>';
                    reset_button.title = "Reset to default";

                    reset_button.onclick = () => {
                        this.update_setting(setting, setting.default_value);
                        value_display.textContent = setting.default_value.toFixed(2);
                        slider.value = slider.defaultValue;
                    }

                    element.appendChild(reset_button);
                }
                else {
                    const value = document.createElement("span");
                    value.classList.add("settings-entry-value");
                    value.textContent = setting.value.toFixed(2);
                    element.appendChild(value);
                }


                section_element.appendChild(element);
            }

            this.panel.appendChild(section_element);
        }
    }

    private save() {
        localStorage.setItem("settings", JSON.stringify(this.settings));
    }

    private load(): Setting[] | null {
        const settings = localStorage.getItem("settings");

        if (settings !== null) {
            return JSON.parse(settings);
        }

        return null;
    }

    private update_setting(setting: Setting, value: any) {
        setting.value = value;

        if (setting.handlers !== undefined) {
            for (const handler of setting.handlers) {
                handler(setting.value);
            }
        }

        this.save();
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
    default_value: any,
    value?: any,
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
                default_value: 4,
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
                default_value: false,
                hotkey: "i",
            },
            {
                id: "downed_radar.enabled",
                name: "Show Downed Radar",
                type: "boolean",
                default_value: true,
            },
            {
                id: "downed_radar.distance",
                name: "Downed Radar Distance",
                type: "number",
                default_value: 7,
                options: {
                    min: 1, max: 10,
                    slider_step: 0.1
                }
            },
            {
                id: "downed_radar.cutoff_distance",
                name: "Downed Radar Cutoff Distance",
                type: "number",
                default_value: 10,
                options: {
                    min: 5, max: 20,
                    slider_step: 0.5
                }
            },
            {
                id: "downed_radar.opacity",
                name: "Downed Radar Opacity",
                type: "number",
                default_value: 0.5,
                options: {
                    min: 0.2, max: 1,
                    slider_step: 0.01
                }
            },
            {
                id: "downed_radar.size",
                name: "Downed Radar Size",
                type: "number",
                default_value: 0.7,
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
            { id: "minimap", name: "Show Minimap", type: "boolean", default_value: true, hotkey: "m" },
            { id: "leaderboard", name: "Show Leaderboard", type: "boolean", default_value: true, hotkey: "b" },
            { id: "chat", name: "Show Chat", type: "boolean", default_value: true, hotkey: "v" },
            { id: "metrics", name: "Show Metrics", type: "boolean", default_value: true, hotkey: "n" },
        ],
    },
    {
        id: "metrics",
        name: "Metrics",
        settings: [
            { id: "fps_enabled", name: "Show FPS", type: "boolean", default_value: true },
            { id: "ping_enabled", name: "Show Ping", type: "boolean", default_value: true },
            { id: "bandwidth_enabled", name: "Show Bandwidth", type: "boolean", default_value: true },
            { id: "render_time_enabled", name: "Show Render Time", type: "boolean", default_value: false },
        ],
    },
]);
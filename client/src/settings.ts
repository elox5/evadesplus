

class Settings {
    private sections: SettingsSection[] = [];
    private panel: HTMLDivElement;

    constructor(sections: SettingsSection[]) {
        this.sections = sections;

        this.panel = document.querySelector("#settings-panel") as HTMLDivElement;

        const header = document.createElement("h1");
        header.textContent = "Settings";
        this.panel.appendChild(header);

        for (const section of this.sections) {
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
                        setting.onchange?.(setting.value);
                    }

                    element.appendChild(checkbox);
                } else {
                    const value = document.createElement("span");
                    value.textContent = setting.value.toString();
                    element.appendChild(value);
                }

                section_element.appendChild(element);
            }

            this.panel.appendChild(section_element);
        }
    }

    get_setting(name: string) {
        return this.sections.flatMap(section => section.settings).find(setting => setting.name === name);
    }
}

type SettingsSection = {
    name: string,
    settings: Setting[],
}

type Setting = {
    name: string,
    value: any,
    hotkey?: string,
    onchange?: (v: any) => void
}

const minimap = document.querySelector("#minimap-container") as HTMLDivElement;
const leaderboard = document.querySelector("#leaderboard") as HTMLDivElement;
const chat = document.querySelector("#chat") as HTMLDivElement;
const metrics = document.querySelector("#metrics-container") as HTMLDivElement;

export const settings = new Settings([
    {
        name: "HUD",
        settings: [
            { name: "Show Minimap", value: true, hotkey: "m", onchange: (v) => minimap.classList.toggle("hidden", !v) },
            { name: "Show Leaderboard", value: true, hotkey: "b", onchange: (v) => leaderboard.classList.toggle("hidden", !v) },
            { name: "Show Chat", value: true, hotkey: "v", onchange: (v) => chat.classList.toggle("hidden", !v) },
            { name: "Show Metrics", value: true, hotkey: "n", onchange: (v) => metrics.classList.toggle("hidden", !v) },
        ],
    },
]);
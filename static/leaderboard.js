
class Leaderboard {
    constructor() {
        this.element = document.getElementById("leaderboard");

        this.maps = {};
    }


    add(hash, areaOrder, playerName, areaName, mapName, downed) {
        if (!this.maps[mapName]) {
            this.maps[mapName] = new Map(mapName);
            this.element.appendChild(this.maps[mapName].element);
        }

        this.maps[mapName].add(hash, areaOrder, playerName, areaName, downed);
    }

    remove(hash) {
        for (const map of Object.values(this.maps)) {
            map.remove(hash);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                delete this.maps[map.name];
            }
        }
    }

    setDowned(hash, downed) {
        for (const map of Object.values(this.maps)) {
            map.setDowned(hash, downed);
        }
    }
}

class Map {
    constructor(name) {
        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-map");

        this.list = document.createElement("div");
        this.list.classList.add("leaderboard-map-list");

        const header = document.createElement("h3");
        header.classList.add("leaderboard-map-name");
        header.textContent = name;

        this.element.appendChild(header);
        this.element.appendChild(this.list);

        this.name = name;
        this.entries = [];
    }

    add(hash, areaOrder, playerName, areaName, downed) {
        this.entries.push(new Entry(hash, areaOrder, playerName, areaName, downed));
        this.entries.sort((a, b) => b.area - a.area);

        this.updateList();
    }

    remove(hash) {
        this.entries = this.entries.filter(entry => entry.hash !== hash);

        this.updateList();
    }

    setDowned(hash, downed) {
        for (const entry of this.entries) {
            if (entry.hash === hash) {
                entry.element.classList.toggle("downed", downed);
            }
        }
    }

    updateList() {
        this.list.textContent = "";
        for (const entry of this.entries) {
            this.list.appendChild(entry.element);
        }
    }
}

class Entry {
    constructor(hash, areaOrder, playerName, areaName, downed) {
        this.hash = hash;
        this.areaOrder = areaOrder;
        this.playerName = playerName;
        this.areaName = areaName;

        this.element = document.createElement("div");
        this.element.classList.add("leaderboard-entry");

        if (downed) {
            this.element.classList.add("downed");
        }

        const nameDiv = document.createElement("div");
        nameDiv.classList.add("leaderboard-entry-name");
        nameDiv.textContent = playerName;
        this.element.appendChild(nameDiv);

        const areaDiv = document.createElement("div");
        areaDiv.classList.add("leaderboard-entry-area");
        areaDiv.textContent = `${areaName}`;
        this.element.appendChild(areaDiv);
    }
}

export const leaderboard = new Leaderboard();
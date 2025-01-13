
class Leaderboard {
    constructor() {
        this.element = document.getElementById("leaderboard");

        this.maps = {};
    }


    add(playerId, playerName, areaOrder, areaName, mapName, downed) {
        console.log("add", playerId);


        if (!this.maps[mapName]) {
            this.maps[mapName] = new Map(mapName);
            this.element.appendChild(this.maps[mapName].element);
        }

        this.maps[mapName].add(playerId, playerName, areaOrder, areaName, downed);
    }

    remove(playerId) {
        for (const map of Object.values(this.maps)) {
            map.remove(playerId);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                delete this.maps[map.name];
            }
        }
    }

    transfer(playerId, areaOrder, areaName, mapName) {
        console.log("transfer", playerId);

        for (const map of Object.values(this.maps)) {
            let oldEntryIndex = map.entries.findIndex(entry => entry.playerId === playerId);

            if (oldEntryIndex === -1) {
                continue;
            }

            let oldEntry = map.entries[oldEntryIndex];
            map.remove(playerId);

            this.add(playerId, oldEntry.playerName, areaOrder, areaName, mapName, oldEntry.downed);

            if (map.entries.length === 0) {
                this.element.removeChild(map.element);
                delete this.maps[map.name];
            }
        }
    }

    setDowned(playerId, downed) {
        for (const map of Object.values(this.maps)) {
            map.setDowned(playerId, downed);
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

    add(playerId, playerName, areaOrder, areaName, downed) {
        this.entries.push(new Entry(playerId, playerName, areaOrder, areaName, downed));
        this.entries.sort((a, b) => b.area - a.area);

        this.updateList();
    }

    remove(playerId) {
        this.entries = this.entries.filter(entry => entry.playerId !== playerId);

        this.updateList();
    }

    setDowned(playerId, downed) {
        for (const entry of this.entries) {
            if (entry.playerId === playerId) {
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
    constructor(playerId, playerName, areaOrder, areaName, downed) {
        this.playerId = playerId;
        this.areaOrder = areaOrder;
        this.playerName = playerName;
        this.areaName = areaName;
        this.downed = downed;

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
// @treehouse(non-production)

import { listen } from "treehouse/common/event.js";

export async function requestKey() {
    let ws = new WebSocket("/radio/station/1801812339");

    ws.addEventListener("error", (event) => {
        throw event.data;
    });

    await listen(ws, "open");

    let protocolHeader = await listen(ws, "message");
    if (protocolHeader.data != "/treehouse/protocol/key-alpha/v1") {
        throw new Error("incompatible protocol");
    }

    ws.send("key");
    let mazeEvent = await listen(ws, "message");

    ws.send(solveMaze(new Maze(mazeEvent.data)));
    let resultEvent = await listen(ws, "message");

    let [status, data] = resultEvent.data.split(": ");
    if (status == "ok") {
        return data;
    } else {
        throw new Error(`server reported an error: ${data}`);
    }
}

function reconstructPath(cameFrom, current) {
    let totalPath = [current];
    while (cameFrom.has(current)) {
        current = cameFrom.get(current);
        totalPath.push(current);
    }
    totalPath.reverse();
    return totalPath;
}

function astar(start, goal, { heuristic, getNeighbors }) {
    let openSet = new Set([start]);

    let cameFrom = new Map();

    let gScore = new Map();
    gScore.set(start, 0);

    let fScore = new Map();
    fScore.set(start, heuristic(start));

    while (openSet.size > 0) {
        let current = null;
        let minFScore = Infinity;
        openSet.forEach((node) => {
            let nodeFScore = fScore.get(node) ?? Infinity;
            if (nodeFScore < minFScore) {
                current = node;
                minFScore = nodeFScore;
            }
        });
        if (current == goal) {
            return reconstructPath(cameFrom, current);
        }

        openSet.delete(current);
        for (let neighbor of getNeighbors(current)) {
            let tentativeGScore = gScore.get(current) ?? Infinity;
            if (tentativeGScore < (gScore.get(neighbor) ?? Infinity)) {
                cameFrom.set(neighbor, current);
                gScore.set(neighbor, tentativeGScore);
                fScore.set(neighbor, tentativeGScore + heuristic(neighbor));
                if (!openSet.has(neighbor)) {
                    openSet.add(neighbor);
                }
            }
        }
    }

    return null;
}

class Maze {
    static WALL = 0;
    static PATH = 1;

    constructor(ascii) {
        let lines = ascii.split("\n");
        lines.pop();

        this.width = lines[0].length / 2;
        this.height = lines.length;
        this.cells = new Uint8Array(this.width * this.height);

        for (let y = 0; y < this.height; ++y) {
            for (let x = 0; x < this.width; ++x) {
                let cell = lines[y].substring(x * 2, x * 2 + 2);
                if (cell == "  ") {
                    this.cells[x + y * this.width] = Maze.PATH;
                }
            }
        }
    }

    get(x, y) {
        if (x >= 0 && y >= 0 && x < this.width && y < this.height) {
            return this.cells[x + y * this.width];
        } else {
            return Maze.WALL;
        }
    }
}

function solveMaze(maze) {
    let start = [1, 1];
    let goal = [maze.width - 2, maze.height - 2];

    let path = astar(JSON.stringify(start), JSON.stringify(goal), {
        heuristic(node) {
            node = JSON.parse(node);
            let [nodeX, nodeY] = node;
            let [goalX, goalY] = goal;
            let deltaX = goalX - nodeX;
            let deltaY = goalY - nodeY;
            return Math.abs(deltaX + deltaY);
        },
        getNeighbors(node) {
            node = JSON.parse(node);
            let [nodeX, nodeY] = node;
            let neighbors = [];
            if (maze.get(nodeX + 1, nodeY) == Maze.PATH)
                neighbors.push(JSON.stringify([nodeX + 2, nodeY]));
            if (maze.get(nodeX, nodeY + 1) == Maze.PATH)
                neighbors.push(JSON.stringify([nodeX, nodeY + 2]));
            if (maze.get(nodeX - 1, nodeY) == Maze.PATH)
                neighbors.push(JSON.stringify([nodeX - 2, nodeY]));
            if (maze.get(nodeX, nodeY - 1) == Maze.PATH)
                neighbors.push(JSON.stringify([nodeX, nodeY - 2]));
            return neighbors;
        },
    });

    let pathStr = "";
    let hamsterX = 1;
    let hamsterY = 1;
    for (let i = 1; i < path.length; ++i) {
        let [nodeX, nodeY] = JSON.parse(path[i]);
        let deltaX = nodeX - hamsterX;
        let deltaY = nodeY - hamsterY;

        if (deltaX == 2 && deltaY == 0) pathStr += "E";
        if (deltaX == 0 && deltaY == 2) pathStr += "S";
        if (deltaX == -2 && deltaY == 0) pathStr += "W";
        if (deltaX == 0 && deltaY == -2) pathStr += "N";

        hamsterX = nodeX;
        hamsterY = nodeY;
    }

    return pathStr;
}

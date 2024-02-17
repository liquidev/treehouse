import { Frame, defineFrame } from "./framework.js";
import tilemapRegistry from "./tilemap-registry.js";

export function canConnect(tile) {
    return tile == 1;
}

export function shouldConnect(a, b) {
    return a == b;
}

export class TileEditor extends Frame {
    constructor() {
        super();
        this.tileCursor = { x: 0, y: 0 };

        this.colorScheme = {
            background: "#F7F7F7",
            grid: "#00000011",
            tileCursor: "#222222",
            tiles: [
                "transparent",
                "#eb134a",
            ],
        };

        this.tileColorPalette = [
            "transparent",
            "#eb134a",
        ];
    }

    connectedCallback() {
        super.connectedCallback();

        this.tileSize = parseInt(this.getAttribute("data-tile-size"));

        let tilemapId = this.getAttribute("data-tilemap-id");
        if (tilemapId != null) {
            this.tilemap = tilemapRegistry[this.getAttribute("data-tilemap-id")];
        } else {
            throw new ReferenceError(`tilemap '${tilemapId}' does not exist`);
        }

        // 0st element is explicitly null because it represents the empty tile.
        this.tilesets = [null];

        let attachedImages = this.getElementsByTagName("img");
        for (let image of attachedImages) {
            if (image.hasAttribute("data-tairu-tileset")) {
                let tilesetIndex = parseInt(image.getAttribute("data-tairu-tileset"));
                this.tilesets[tilesetIndex] = image;
            }
        }

        this.width = this.tilemap.width * this.tileSize;
        this.height = this.tilemap.height * this.tileSize;

        this.hasFocus = false;
        this.paintingTile = null;

        this.addEventListener("mousemove", event => this.mouseMoved(event));
        this.addEventListener("mousedown", event => this.mousePressed(event));
        this.addEventListener("mouseup", event => this.mouseReleased(event));

        this.addEventListener("mouseenter", _ => this.hasFocus = true);
        this.addEventListener("mouseleave", _ => this.hasFocus = false);

        this.addEventListener("contextmenu", event => event.preventDefault());

        // TODO: This should also work on mobile.
    }

    draw() {
        this.ctx.fillStyle = this.colorScheme.background;
        this.ctx.fillRect(0, 0, this.width, this.height);

        this.drawTiles();
        this.drawGrid();
        if (this.hasFocus) {
            this.drawTileCursor();
        }
    }

    drawGrid() {
        this.ctx.beginPath();
        for (let x = 0; x < this.tilemap.width; ++x) {
            this.ctx.moveTo(x * this.tileSize, 0);
            this.ctx.lineTo(x * this.tileSize, this.height);
        }
        for (let y = 0; y < this.tilemap.width; ++y) {
            this.ctx.moveTo(0, y * this.tileSize);
            this.ctx.lineTo(this.width, y * this.tileSize);
        }
        this.ctx.strokeStyle = this.colorScheme.grid;
        this.ctx.lineWidth = 1;
        this.ctx.stroke();
    }

    drawTileCursor() {
        this.ctx.strokeStyle = this.colorScheme.tileCursor;
        this.ctx.lineWidth = 5;
        this.ctx.strokeRect(this.tileCursor.x * this.tileSize, this.tileCursor.y * this.tileSize, this.tileSize, this.tileSize);
    }

    get hasTilesets() {
        // Remember that tile 0 represents emptiness.
        return this.tilesets.length > 1;
    }

    drawTiles() {
        if (this.hasTilesets) {
            this.drawTexturedTiles();
        } else {
            this.drawColoredTiles();
        }
    }

    drawColoredTiles() {
        for (let y = 0; y < this.tilemap.height; ++y) {
            for (let x = 0; x < this.tilemap.width; ++x) {
                let tile = this.tilemap.at(x, y);
                if (tile != 0) {
                    this.ctx.fillStyle = this.colorScheme.tiles[tile];
                    this.ctx.fillRect(x * this.tileSize, y * this.tileSize, this.tileSize, this.tileSize);
                }
            }
        }
    }

    drawTexturedTiles() {
        this.ctx.imageSmoothingEnabled = false;

        for (let y = 0; y < this.tilemap.height; ++y) {
            for (let x = 0; x < this.tilemap.width; ++x) {
                let tile = this.tilemap.at(x, y);
                if (tile != 0) {
                    let tileset = this.tilesets[tile];

                    let connectedWithEast = shouldConnect(tile, this.tilemap.at(x + 1, y)) ? 0b0001 : 0;
                    let connectedWithSouth = shouldConnect(tile, this.tilemap.at(x, y + 1)) ? 0b0010 : 0;
                    let connectedWithWest = shouldConnect(tile, this.tilemap.at(x - 1, y)) ? 0b0100 : 0;
                    let connectedWithNorth = shouldConnect(tile, this.tilemap.at(x, y - 1)) ? 0b1000 : 0;
                    let tileIndex = connectedWithNorth
                        | connectedWithWest
                        | connectedWithSouth
                        | connectedWithEast;

                    let tilesetTileSize = tileset.height;
                    let tilesetX = tileIndex * tilesetTileSize;
                    let tilesetY = 0;
                    this.ctx.drawImage(
                        this.tilesets[tile],
                        tilesetX, tilesetY, tilesetTileSize, tilesetTileSize,
                        x * this.tileSize, y * this.tileSize, this.tileSize, this.tileSize,
                    );
                }
            }
        }
    }

    mouseMoved(event) {
        let mouse = this.getMousePositionFromEvent(event);
        this.tileCursor.x = Math.floor(mouse.x / this.tileSize);
        this.tileCursor.y = Math.floor(mouse.y / this.tileSize);
        this.paintTileUnderCursor();
    }

    mousePressed(event) {
        event.preventDefault();
        if (event.button == 0) {
            this.paintingTile = 1;
        } else if (event.button == 2) {
            this.paintingTile = 0;
        }
        this.paintTileUnderCursor();
    }

    mouseReleased() {
        this.paintingTile = null;
    }

    paintTileUnderCursor() {
        if (this.paintingTile != null) {
            this.tilemap.setAt(this.tileCursor.x, this.tileCursor.y, this.paintingTile);
        }
    }
}
defineFrame("tairu-editor", TileEditor);

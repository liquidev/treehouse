import { Sketch } from "treehouse/sandbox.js";

export class TileEditor extends Sketch {
    constructor({ tilemap, tileSize }) {
        super(tilemap.width * tileSize, tilemap.height * tileSize);

        this.colorScheme = {
            background: "#F7F7F7",
            grid: "#00000011",
            tileCursor: "#222222",
            tiles: [
                "transparent", // never actually drawn to the screen with the default renderer!
                "#eb134a",
            ],
        };

        this.tilemap = tilemap;
        this.tileSize = tileSize;

        this.hasFocus = false;
        this.paintingTile = null;
        this.tileCursor = { x: 0, y: 0 };

        this.canvas.addEventListener("mousemove", event => this.mouseMoved(event));
        this.canvas.addEventListener("mousedown", event => this.mousePressed(event));
        this.canvas.addEventListener("mouseup", event => this.mouseReleased(event));

        this.canvas.addEventListener("mouseenter", _ => this.mouseEnter());
        this.canvas.addEventListener("mouseleave", _ => this.mouseLeave());

        this.canvas.addEventListener("contextmenu", event => event.preventDefault());

        // Only draw first frame after the constructor already runs.
        // That way we can modify the color scheme however much we want without causing additional
        // redraws.
        requestAnimationFrame(() => this.draw());
    }

    draw() {
        this.drawBackground();
        this.drawTilemap();
        this.drawGrid();
        if (this.hasFocus) {
            this.drawTileCursor();
        }
    }

    drawBackground() {
        this.ctx.fillStyle = this.colorScheme.background;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    }

    drawTilemap() {
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

    drawGrid() {
        this.ctx.beginPath();
        for (let x = 0; x < this.tilemap.width; ++x) {
            this.ctx.moveTo(x * this.tileSize, 0);
            this.ctx.lineTo(x * this.tileSize, this.canvas.height);
        }
        for (let y = 0; y < this.tilemap.width; ++y) {
            this.ctx.moveTo(0, y * this.tileSize);
            this.ctx.lineTo(this.canvas.width, y * this.tileSize);
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


    mouseMoved(event) {
        this.tileCursor.x = Math.floor(event.offsetX / this.tileSize);
        this.tileCursor.y = Math.floor(event.offsetY / this.tileSize);
        this.paintTileUnderCursor();

        this.draw();
    }

    mousePressed(event) {
        event.preventDefault();

        if (event.button == 0) {
            this.paintingTile = 1;
        } else if (event.button == 2) {
            this.paintingTile = 0;
        }

        this.paintTileUnderCursor();

        this.draw();
    }

    mouseReleased(_event) {
        this.stopPainting();
        this.draw();
    }

    mouseEnter() {
        this.hasFocus = true;
        this.draw();
    }

    mouseLeave() {
        this.hasFocus = false;
        this.stopPainting();
        this.draw();
    }

    paintTileUnderCursor() {
        if (this.paintingTile != null) {
            this.tilemap.setAt(this.tileCursor.x, this.tileCursor.y, this.paintingTile);
        }
    }

    stopPainting() {
        this.paintingTile = null;
    }
}

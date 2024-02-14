export class Tilemap {
    constructor(width, height) {
        this.width = width;
        this.height = height;
        this.tiles = new Uint8Array(width * height);
        this.default = 0;
    }

    tileIndex(x, y) {
        return x + y * this.width;
    }

    inBounds(x, y) {
        return x >= 0 && y >= 0 && x < this.width && y < this.height;
    }

    at(x, y) {
        if (this.inBounds(x, y)) {
            return this.tiles[this.tileIndex(x, y)];
        } else {
            return this.default;
        }
    }

    setAt(x, y, tile) {
        if (this.inBounds(x, y)) {
            this.tiles[this.tileIndex(x, y)] = tile;
        }
    }
}

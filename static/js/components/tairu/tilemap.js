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

    static parse(alphabet, lineArray) {
        let tilemap = new Tilemap(lineArray[0].length, lineArray.length);
        for (let y in lineArray) {
            let line = lineArray[y];
            for (let x = 0; x < line.length; ++x) {
                let char = line.charAt(x);
                tilemap.setAt(x, y, alphabet.indexOf(char));
            }
        }
        return tilemap;
    }
}

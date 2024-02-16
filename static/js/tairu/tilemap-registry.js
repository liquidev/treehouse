import { Tilemap } from './tilemap.js';

const alphabet = " x";

function parseTilemap(lineArray) {
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

export default {
    bitwiseAutotiling: parseTilemap([
        "         ",
        "   xxx   ",
        "   xxx   ",
        "   xxx   ",
        "         ",
    ]),
    bitwiseAutotilingChapter2: parseTilemap([
        "         ",
        "   x     ",
        "   x     ",
        "   xxx   ",
        "         ",
    ]),
    bitwiseAutotilingCorners: parseTilemap([
        "     ",
        " x x ",
        "  x  ",
        " x x ",
        "     ",
    ]),
};


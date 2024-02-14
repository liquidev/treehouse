import { defineFrame, Frame } from './framework.js';
import { TileEditor, canConnect, shouldConnect } from './tairu.js';

class CardinalDirectionsEditor extends TileEditor {
    constructor() {
        super();
        this.colorScheme.tiles[1] = "#f96565";
    }

    drawConnectionText(text, enabled, tileX, tileY, hAlign, vAlign) {
        this.ctx.beginPath();
        this.ctx.fillStyle = enabled ? "#6c023e" : "#d84161";
        this.ctx.font = `800 14px ${Frame.monoFontFace}`;
        const padding = 2;
        let topLeftX = tileX * this.tileSize + padding;
        let topLeftY = tileY * this.tileSize + padding;
        let rectSize = this.tileSize - padding * 2;
        let { leftX, baselineY } = this.getTextPositionInBox(text, topLeftX, topLeftY, rectSize, rectSize, hAlign, vAlign);
        this.ctx.fillText(text, leftX, baselineY);
    }

    drawTiles() {
        super.drawTiles();
        for (let y = 0; y < this.tilemap.height; ++y) {
            for (let x = 0; x < this.tilemap.width; ++x) {
                let tile = this.tilemap.at(x, y);
                if (canConnect(tile)) {
                    let connectedWithEast = shouldConnect(tile, this.tilemap.at(x + 1, y));
                    let connectedWithSouth = shouldConnect(tile, this.tilemap.at(x, y + 1));
                    let connectedWithNorth = shouldConnect(tile, this.tilemap.at(x, y - 1));
                    let connectedWithWest = shouldConnect(tile, this.tilemap.at(x - 1, y));
                    this.drawConnectionText("E", connectedWithEast, x, y, "right", "center");
                    this.drawConnectionText("S", connectedWithSouth, x, y, "center", "bottom");
                    this.drawConnectionText("N", connectedWithNorth, x, y, "center", "top");
                    this.drawConnectionText("W", connectedWithWest, x, y, "left", "center");
                }
            }
        }
    }
}
defineFrame("tairu-editor-cardinal-directions", CardinalDirectionsEditor);

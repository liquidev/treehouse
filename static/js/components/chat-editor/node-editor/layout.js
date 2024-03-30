import { NodeBase } from "./node-base.js";

export function getPosition(node) {
    if (node instanceof NodeBase) {
        return node.modelNode.position;
    } else {
        return [node.offsetLeft, node.offsetTop];
    }
}

export function getPositionRelativeToAncestor(ancestor, element) {
    let x = 0;
    let y = 0;
    while (element != ancestor && element != null) {
        let [elementX, elementY] = getPosition(element);
        x += elementX;
        y += elementY;
        element = element.offsetParent;
    }
    if (x != x) x = 0;
    if (y != y) y = 0;
    return [x, y];
}

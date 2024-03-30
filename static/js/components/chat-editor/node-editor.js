import * as nodes from "./node-editor/nodes.js";
import { contextMenus } from "./context-menu.js";
import { AddNode } from "./node-editor/add-node.js";
import { getPositionRelativeToAncestor } from "./node-editor/layout.js";

const identityMatrix = new DOMMatrixReadOnly();

export class NodeEditor extends HTMLElement {
    nodes = new Map();
    connectionGroups = new Map();
    dependencies = new Map();

    transformMatrix = new DOMMatrix();
    transformMatrixInverted = new DOMMatrix();
    mouseX = 0.0;
    mouseY = 0.0;

    panning = { x: 0.0, y: 0.0 };
    isPanning = false;
    zoomLevel = 0.0;

    isShiftDown = false;

    selectedNodes = new Set();
    isDraggingSelected = false;

    selectionAnchor = null;
    rubberbandSelectedNodes = new Set();

    ongoingConnection = null;
    ongoingConnectionLine = null;
    hoveredPin = null;

    constructor(model) {
        super();
        this.model = model;
    }

    connectedCallback() {
        this.svg = this.appendChild(createSvg("svg"));
        this.svgGroup = this.svg.appendChild(createSvg("g"));

        this.canvasDiv = this.appendChild(document.createElement("div"));
        this.canvasDiv.classList.add("canvas");

        this.nodesDiv = this.canvasDiv.appendChild(document.createElement("div"));
        this.nodesDiv.classList.add("nodes");

        this.selectionRubberband = this.canvasDiv.appendChild(document.createElement("div"));
        this.selectionRubberband.classList.add("selection-rubberband");

        this.addEventListener("mousedown", (event) => {
            let targetIsThis =
                event.target == this.canvasDiv ||
                event.target == this.nodesDiv ||
                event.target == this.svg;
            if (targetIsThis && event.button == 0) {
                event.preventDefault();

                document.activeElement.blur();
                this.focus();

                this.beginRubberbandSelection();
            }
            if (event.button == 1) {
                event.preventDefault();
                this.isPanning = true;
            }
            if (event.button == 2) {
                let bounds = this.getBoundingClientRect();
                let worldPosition = new DOMPoint(
                    event.clientX - bounds.x,
                    event.clientY - bounds.y
                );
                worldPosition = this.transformMatrixInverted.transformPoint(worldPosition);

                let menu = contextMenus.open(new AddNode(event));
                menu.addEventListener(".addNode", (event) => {
                    let name = nodes.generateUniqueName();
                    event.modelNode.position = [worldPosition.x, worldPosition.y];
                    this.model.nodes[name] = event.modelNode;
                    this.nodesDiv.appendChild(this.createNode(name));
                    this.sendModelUpdate();
                });
            }
        });

        this.addEventListener("wheel", (event) => {
            this.zoomLevel += Math.sign(-event.deltaY);
            this.updateTransform();
        });

        window.addEventListener("mouseup", (event) => {
            if (event.button == 0) {
                this.isDraggingSelected = false;
                if (this.ongoingConnection != null) {
                    this.dropPin();
                }
                this.endRubberbandSelection();
            }
            if (event.button == 1) {
                this.isPanning = false;
            }
        });

        this.addEventListener("mousemove", (event) => {
            let bounds = this.getBoundingClientRect();
            let x = event.clientX - bounds.left;
            let y = event.clientY - bounds.top;
            let point = this.transformMatrixInverted.transformPoint(new DOMPoint(x, y));
            this.mouseX = point.x;
            this.mouseY = point.y;
        });

        window.addEventListener("mousemove", (event) => {
            if (this.isDraggingSelected) {
                for (let name of this.selectedNodes) {
                    let node = this.nodes.get(name);
                    let zoom = this.zoom;
                    node.move(event.movementX / zoom, event.movementY / zoom);
                }
            }
            if (this.isPanning) {
                this.panBy(event.movementX, event.movementY);
            }
            if (this.ongoingConnection != null) {
                this.updateOngoingConnection();
            }
            if (this.selectionAnchor != null) {
                this.updateRubberbandSelection();
            }
        });

        this.addEventListener("contextmenu", (event) => {
            event.preventDefault();
        });

        document.addEventListener("keydown", (event) => {
            if (event.target == document.body) {
                if (event.key == "Shift") {
                    this.isShiftDown = true;
                }
                if (event.code == "KeyX" || event.code == "Delete") {
                    this.deleteSelectedNodes();
                }
            }
        });

        document.addEventListener("keyup", (event) => {
            if (event.key == "Shift") {
                this.isShiftDown = false;
            }
        });

        // Need to rebuild connections once fonts get all loaded in, since that may change the
        // size of nodes.
        document.fonts.addEventListener("loadingdone", () => {
            for (let [_, node] of this.nodes) {
                node.updateRenderingCache();
            }
            this.rebuildAllConnections();
        });

        this.rebuildNodes();
        this.updateTransform();
    }

    updateFromModel() {
        this.rebuildNodes();
        this.rebuildAllDependencies();
        this.rebuildAllConnections();
    }

    sendModelUpdate() {
        this.dispatchEvent(new Event(".modelUpdate"));
    }

    get zoom() {
        return Math.pow(2.0, this.zoomLevel * 0.25);
    }

    panBy(x, y) {
        let zoom = this.zoom;
        this.panning.x += x / zoom;
        this.panning.y += y / zoom;
        this.updateTransform();
    }

    updateTransform() {
        let width = this.svg.clientWidth;
        let height = this.svg.clientHeight;

        // TODO: This calculation could probably be used in the svgGroup and nodesDiv's transforms,
        // but I was too lazy to figure it out right now, so it's done twice without much
        // good reason.

        this.transformMatrix.setMatrixValue(identityMatrix);

        this.transformMatrix.translateSelf(width / 2, height / 2);
        this.transformMatrix.scaleSelf(this.zoom, this.zoom);
        this.transformMatrix.translateSelf(this.panning.x, this.panning.y);
        this.transformMatrix.translateSelf(-width / 2, -height / 2);

        this.transformMatrixInverted.setMatrixValue(this.transformMatrix);
        this.transformMatrixInverted.invertSelf();

        this.svgGroup.style.transform = this.transformMatrix;

        // NOTE: this.nodesDiv uses a different transform matrix because it's already
        // sized appropriately.
        this.canvasDiv.style.transform = `
            scale(${this.zoom})
            translate(${this.panning.x}px, ${this.panning.y}px)
        `;
    }

    rebuildNodes() {
        this.nodes.clear();
        this.nodesDiv.replaceChildren();

        for (let name in this.model.nodes) {
            this.nodesDiv.appendChild(this.createNode(name));
        }
    }

    updateNodeSelectionState() {
        for (let node of this.nodesDiv.childNodes) {
            node.classList.remove("selected");
        }
        for (let name of this.selectedNodes) {
            let node = this.nodes.get(name);
            node.classList.add("selected");
        }
        for (let name of this.rubberbandSelectedNodes) {
            let node = this.nodes.get(name);
            node.classList.add("selected");
        }
    }

    createNode(name) {
        let node = new nodes.types[this.model.nodes[name].kind](this.model, name);

        node.addEventListener(".modelUpdate", () => {
            this.sendModelUpdate();
            this.rebuildDependenciesForNode(name);
            this.rebuildConnectionsForNodeOneDeep(name);
        });

        node.addEventListener(".select", () => {
            if (!this.isShiftDown && !this.selectedNodes.has(name)) {
                this.selectedNodes.clear();
            }
            this.selectedNodes.add(name);
            this.updateNodeSelectionState();

            this.isDraggingSelected = true;
        });

        node.addEventListener(".pinDrag", (event) => {
            if (event.pin.direction == "output") {
                this.ongoingConnection = { name, pin: event.pin };
                event.pin.beginConnecting();
            } else {
            }
        });

        node.addEventListener(".pinHover", (event) => {
            this.hoveredPin = { name, pin: event.pin };
        });

        node.addEventListener(".pinEndHover", (event) => {
            if (this.hoveredPin != null && this.hoveredPin.pin == event.pin) {
                this.hoveredPin = null;
            }
        });

        node.addEventListener(".pinDisconnect", (event) => {
            this.disconnectPin(name, event.pin);
        });

        this.nodes.set(name, node);

        return node;
    }

    deleteNode(name) {
        let node = this.nodes.get(name);
        this.nodes.delete(name);
        node.parentNode.removeChild(node);

        let connectionGroup = this.connectionGroups.get(name);
        if (connectionGroup != null) {
            for (let line of connectionGroup.childNodes) {
                poolPath(line);
            }
            connectionGroup.parentNode.removeChild(connectionGroup);
            this.connectionGroups.delete(name);
        }

        let dependencies = this.dependencies.get(name);
        if (dependencies != null) {
            for (let dependency of dependencies) {
                for (let pin of node.outputPins) {
                    if (pin.value.get() == name) {
                        pin.value.set(null);
                    }
                }
                this.rebuildConnectionsForSingleNode(dependency);
            }
        }

        for (let pin of node.outputPins) {
            let connectedToName = pin.value.get();
            let dependencies = this.dependencies.get(connectedToName);
            if (dependencies != null) {
                dependencies.delete(name);
            }
        }
        this.dependencies.delete(name);

        delete this.model.nodes[name];

        this.sendModelUpdate();
    }

    deleteSelectedNodes() {
        for (let name of this.selectedNodes) {
            this.deleteNode(name);
        }
        this.selectedNodes.clear();
    }

    rebuildAllDependencies() {
        this.dependencies.clear();
        for (let [name, _node] of this.nodes) {
            this.rebuildDependenciesForNode(name);
        }
    }

    rebuildDependenciesForNode(name) {
        let node = this.nodes.get(name);

        for (let pin of node.outputPins) {
            let connectedToName = pin.value.get();
            let connectedToNode = this.nodes.get(connectedToName);
            if (connectedToNode != null) {
                let set = this.dependencies.get(connectedToName) ?? new Set();
                set.add(name);
                this.dependencies.set(connectedToName, set);
            }
        }
    }

    rebuildAllConnections() {
        let paths = this.svgGroup.getElementsByClassName("path");
        for (let path of paths) {
            poolPath(path);
        }
        this.svgGroup.replaceChildren();

        this.connectionGroups.clear();
        for (let [name, _node] of this.nodes) {
            this.rebuildConnectionsForSingleNode(name);
        }
    }

    rebuildConnectionsForNodeOneDeep(name) {
        this.rebuildConnectionsForSingleNode(name);

        let dependencySet = this.dependencies.get(name);
        if (dependencySet != null) {
            for (let dependency of dependencySet) {
                this.rebuildConnectionsForSingleNode(dependency);
            }
        }
    }

    rebuildConnectionsForNodeRecursive(name, rebuiltSet) {
        if (rebuiltSet.has(name)) {
            return;
        }
        rebuiltSet.add(name);

        this.rebuildConnectionsForSingleNode(name);

        let dependencySet = this.dependencies.get(name);
        if (dependencySet != null) {
            for (let dependency of dependencySet) {
                this.rebuildConnectionsForNodeRecursive(dependency, rebuiltSet);
            }
        }
    }

    rebuildConnectionsForSingleNode(name) {
        let node = this.nodes.get(name);

        let svgGroup = this.connectionGroups.get(name);
        if (svgGroup == null) {
            svgGroup = this.svgGroup.appendChild(createSvg("g"));
            this.connectionGroups.set(name, svgGroup);
        }

        for (let i = svgGroup.childNodes.length; i-- > 0; ) {
            let path = svgGroup.childNodes[i];
            poolPath(path);
        }

        for (let outputPin of node.outputPins) {
            let connectedToName = outputPin.value.get();
            let connectedToNode = this.nodes.get(connectedToName);
            if (connectedToNode != null) {
                let outputPinRect = node.getPinRect(outputPin);
                let fromX =
                    node.modelNode.position[0] +
                    outputPinRect.x +
                    outputPin.getConnectionX(outputPinRect);
                let fromY =
                    node.modelNode.position[1] +
                    outputPinRect.y +
                    outputPin.getConnectionY(outputPinRect);

                let inputPin = connectedToNode.inputPin;
                let inputPinRect = connectedToNode.getPinRect(inputPin);
                let toX =
                    connectedToNode.modelNode.position[0] +
                    inputPinRect.x +
                    inputPin.getConnectionX(inputPinRect);
                let toY =
                    connectedToNode.modelNode.position[1] +
                    inputPinRect.y +
                    inputPin.getConnectionY(inputPinRect);

                let line = createNodeGraphConnectionLine(fromX, fromY, toX, toY);
                line.setAttribute(
                    "data-debug",
                    `from:${name}.${outputPin.id} to:${connectedToName}`
                );
                svgGroup.appendChild(line);
            }
        }
    }

    updateOngoingConnection() {
        if (this.ongoingConnection != null) {
            poolPath(this.ongoingConnectionLine);

            let outputPin = this.ongoingConnection.pin;
            let [fromX, fromY] = getPositionRelativeToAncestor(this.nodesDiv, outputPin);
            // fromX += outputPin.connectionX;
            // fromY += outputPin.connectionY;
            let toX = this.mouseX;
            let toY = this.mouseY;

            this.ongoingConnectionLine = this.svgGroup.appendChild(
                createNodeGraphConnectionLine(fromX, fromY, toX, toY)
            );
        } else {
            poolPath(this.ongoingConnectionLine);
            this.ongoingConnectionLine = null;
        }
    }

    disconnectPin(name, pin) {
        if (pin.direction == "output") {
            pin.value.set(null);
            this.rebuildConnectionsForNodeRecursive(name, new Set());
            this.sendModelUpdate();
        } else {
            // Find all output pins connected to this one and disconnect them.
            let dependencies = this.dependencies.get(name);
            if (dependencies != null) {
                for (let dependencyName of dependencies) {
                    let node = this.nodes.get(dependencyName);
                    if (node != null) {
                        for (let outputPin of node.outputPins) {
                            if (outputPin.value.get() == name) {
                                this.disconnectPin(dependencyName, outputPin);
                            }
                        }
                    }
                }
            }
        }
    }

    dropPin() {
        let { name, pin } = this.ongoingConnection;

        if (this.hoveredPin != null) {
            if (pin.direction == "output" && this.hoveredPin.pin.direction == "input") {
                pin.value.set(this.hoveredPin.name);
                this.rebuildDependenciesForNode(name);
                this.rebuildConnectionsForSingleNode(name);
                this.sendModelUpdate();
            }
        }

        this.ongoingConnection.pin.endConnecting();
        this.ongoingConnection = null;
        this.updateOngoingConnection();
    }

    beginRubberbandSelection() {
        this.selectionAnchor = { x: this.mouseX, y: this.mouseY };
        this.selectionRubberband.classList.add("selecting");
        if (!this.isShiftDown) {
            this.selectedNodes.clear();
        }
        this.updateRubberbandSelection();
    }

    updateRubberbandSelection() {
        let x = Math.min(this.selectionAnchor.x, this.mouseX);
        let y = Math.min(this.selectionAnchor.y, this.mouseY);
        let width = Math.abs(this.mouseX - this.selectionAnchor.x);
        let height = Math.abs(this.mouseY - this.selectionAnchor.y);
        this.selectionRubberband.style.transform = `translate(${x}px, ${y}px)`;
        this.selectionRubberband.style.width = `${width}px`;
        this.selectionRubberband.style.height = `${height}px`;

        let selectionRect = this.selectionRubberband.getBoundingClientRect();
        this.rubberbandSelectedNodes.clear();
        for (let [name, node] of this.nodes) {
            let nodeRect = node.getBoundingClientRect();
            if (rectsIntersect(selectionRect, nodeRect)) {
                this.rubberbandSelectedNodes.add(name);
            }
        }
        this.updateNodeSelectionState();
    }

    endRubberbandSelection() {
        this.selectionRubberband.classList.remove("selecting");
        this.selectionAnchor = null;
        this.rubberbandSelectedNodes.forEach((v) => this.selectedNodes.add(v));
        this.rubberbandSelectedNodes.clear();
        this.updateNodeSelectionState();
    }
}

customElements.define("th-chat-node-editor", NodeEditor);

function createSvg(element) {
    return document.createElementNS("http://www.w3.org/2000/svg", element);
}

let pathPool = [];

function unpoolPath() {
    return pathPool.pop() ?? createSvg("path");
}

function poolPath(path) {
    if (path != null) {
        path.parentNode.removeChild(path);
        pathPool.push(path);
    }
}

function createNodeGraphConnectionLine(fromX, fromY, toX, toY) {
    let line = unpoolPath();

    let deltaX = toX - fromX;
    let deltaY = toY - fromY;
    let distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
    let pokinessX = Math.max(48, Math.abs(deltaX) * 0.75) * (Math.min(48, distance) / 48);
    let pokinessY = 0;
    if (deltaX < 0) {
        pokinessY += -deltaX * 0.5;
    }
    if (pokinessY > Math.abs(deltaY)) {
        pokinessY = Math.abs(deltaY);
    }

    line.setAttribute(
        "d",
        `
            M ${fromX} ${fromY}
            C ${fromX + pokinessX} ${fromY + pokinessY},
            ${toX - pokinessX} ${toY + pokinessY},
            ${toX} ${toY}
        `
    );

    line.setAttribute("stroke", "var(--border-2)");
    line.setAttribute("fill", "none");
    return line;
}

function rectsIntersect(a, b) {
    return (
        a.x <= b.x + b.width &&
        b.x <= a.x + a.width &&
        a.y <= b.y + b.height &&
        b.y <= a.y + a.height
    );
}

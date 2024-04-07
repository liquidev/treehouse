import { NodeAsk } from "./node-ask.js";
import { NodeSay } from "./node-say.js";
import { NodeSet } from "./node-set.js";
import { NodeEnd } from "./node-end.js";
import { NodeReroute } from "./node-reroute.js";
import { NodeComment } from "./node-comment.js";
import { NodeStart } from "./node-start.js";
import { NodeCheck } from "./node-check.js";
import * as lens from "treehouse/common/lens.js";

function getNoFactReferences() {
    return [];
}

function getThenReference(node) {
    return [lens.field(node, "then")];
}

export const schema = {
    say: {
        editorClass: NodeSay,
        getNodeReferences: getThenReference,
        getFactReferences: getNoFactReferences,
    },
    ask: {
        editorClass: NodeAsk,
        getNodeReferences: (node) => node.questions.map((q) => lens.field(q, "then")),
        getFactReferences: getNoFactReferences,
    },
    set: {
        editorClass: NodeSet,
        getNodeReferences: getThenReference,
        getFactReferences: (node) => [node.fact],
    },
    check: {
        editorClass: NodeCheck,
        getNodeReferences: (node) => [
            lens.field(node, "ifSetThen"),
            lens.field(node, "ifNotSetThen"),
        ],
        getFactReferences: (node) => [node.fact],
    },
    start: {
        editorClass: NodeStart,
        getNodeReferences: getThenReference,
        getFactReferences: getNoFactReferences,
    },
    end: {
        editorClass: NodeEnd,
        getNodeReferences: () => [],
        getFactReferences: getNoFactReferences,
    },
    reroute: {
        editorClass: NodeReroute,
        getNodeReferences: getThenReference,
        getFactReferences: getNoFactReferences,
    },
    comment: {
        editorClass: NodeComment,
        getNodeReferences: () => [],
        getFactReferences: getNoFactReferences,
    },
};

const nameCharset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
export function generateUniqueName() {
    // Change this prefix in case we ever come up with a better algorithm that might conflict.
    let name = "v1.";
    for (let i = 0; i < 16; ++i) {
        let indexInCharset = Math.floor(Math.random() * nameCharset.length);
        name += nameCharset.substring(indexInCharset, indexInCharset + 1);
    }
    return name;
}

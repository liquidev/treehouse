import { NodeAsk } from "./node-ask.js";
import { NodeSay } from "./node-say.js";
import { NodeSet } from "./node-set.js";
import { NodeEnd } from "./node-end.js";
import { NodeReroute } from "./node-reroute.js";
import { NodeComment } from "./node-comment.js";
import { NodeStart } from "./node-start.js";
import * as lens from "treehouse/common/lens.js";

export const types = {
    say: NodeSay,
    ask: NodeAsk,
    set: NodeSet,
    start: NodeStart,
    end: NodeEnd,
    reroute: NodeReroute,
    comment: NodeComment,
};

function getThenReference(node) {
    return [lens.field(node, "then")];
}

export const schema = {
    say: {
        getNodeReferences: getThenReference,
    },
    ask: {
        getNodeReferences(node) {
            return node.questions.map((q) => lens.field(q, "then"));
        },
    },
    set: {
        getNodeReferences: getThenReference,
    },
    start: {
        getNodeReferences: getThenReference,
    },
    end: {
        getNodeReferences: () => [],
    },
    reroute: {
        getNodeReferences: getThenReference,
    },
    comment: {
        getNodeReferences: () => [],
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

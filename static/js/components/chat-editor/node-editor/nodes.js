import { NodeAsk } from "./node-ask.js";
import { NodeSay } from "./node-say.js";
import { NodeSet } from "./node-set.js";

export const types = {
    say: NodeSay,
    ask: NodeAsk,
    set: NodeSet,
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

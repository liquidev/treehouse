import { NodeAsk } from "./node-ask.js";
import { NodeSay } from "./node-say.js";
import { NodeSet } from "./node-set.js";

export const nodeTypes = {
    say: NodeSay,
    ask: NodeAsk,
    set: NodeSet,
};

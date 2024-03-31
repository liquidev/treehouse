// Spells are a simplistic, composition-based replacement for the is="" attribute, which is not
// supported on Webkit due to Apple's engineering being a bunch of obstinate idiots who explicitly
// choose not to follow web standards.

// Limitations:
//   - The data-cast attribute cannot be added dynamically.
//   - There is no disconnectedCallback. Only a constructor which initializes a spell.

let spells = new Map();
let elementsWithUnknownSpells = new Map();

const sSpells = Symbol("spells");

function castSpellOnElement(spellName, element) {
    element[sSpells] ??= new Map();
    if (!elementsWithUnknownSpells.has(spellName)) {
        elementsWithUnknownSpells.set(spellName, new Set());
    }

    let Spell = spells.get(spellName);
    if (Spell != null && !element[sSpells].has(Spell)) {
        element[sSpells].set(Spell, new Spell(element));
        elementsWithUnknownSpells.get(spellName).delete(element);
    } else {
        elementsWithUnknownSpells.get(spellName).add(element);
    }
}

function applySpells(elements) {
    for (let element of elements) {
        if (element instanceof Element) {
            let spellListString = element.getAttribute("data-cast");
            if (spellListString != null) {
                let spellList = spellListString.split(' ');
                for (let spellName of spellList) {
                    castSpellOnElement(spellName, element);
                }
            }
        }
    }
}

export function addSpell(name, spell) {
    spells.set(name, spell);
    let elementsWithThisSpell = elementsWithUnknownSpells.get(name);
    if (elementsWithThisSpell != null) {
        for (let element of elementsWithThisSpell) {
            castSpellOnElement(name, element);
        }
    }
}

// Returns a spell's data. Gotchas: the spell needs to already be on the element.
// Therefore, if this is used from within a spell, the requested spell must have already been
// applied by this point.
// Someday I may change this to an async function that resumes whenever the spell is available to
// iron over this limitation. But today is not that day.
export function spell(element, spell) {
    return element[sSpells].get(spell);
}

// Apply spells to elements which have them and have been loaded so far.
let loadedSoFar = document.querySelectorAll("[data-cast]");
applySpells(loadedSoFar);

// For all other elements, add a mutation observer that will handle them.
let mutationObserver = new MutationObserver(records => {
    for (let record of records) {
        let mutatedNodes = new Set();
        // NOTE: Added nodes may contain children which also need to be processed.
        // Collect those that have [data-cast] on them and apply spells to them.
        for (let addedNode of record.addedNodes) {
            if (!(addedNode instanceof Text)) {
                if (addedNode.getAttribute("data-cast") != null) {
                    mutatedNodes.add(addedNode);
                }
                addedNode.querySelectorAll("[data-cast]").forEach(element => mutatedNodes.add(element));
            }
        }
        applySpells(mutatedNodes);
    }
});
mutationObserver.observe(document, { subtree: true, childList: true });

// ------------ Common spells ------------

// js makes things visible only when JavaScript is enabled.
addSpell("js", function (element) {
    element.style.display = "block";
});

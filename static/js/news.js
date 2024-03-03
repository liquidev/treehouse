// news.js because new.js makes the TypeScript language server flip out.
// Likely because `new` is a keyword, but also, what the fuck.

import { addSpell, spell } from "treehouse/spells.js";
import { getSettingValue } from "treehouse/settings.js";
import { Branch } from "treehouse/tree.js";

const seenStatesKey = "treehouse.news.seenBranches";
const seenStates = new Set(JSON.parse(localStorage.getItem(seenStatesKey)) || []);

let seenCount = seenStates.size;
let unseenCount = TREEHOUSE_NEWS_COUNT - seenCount;

function saveSeenStates() {
    localStorage.setItem(seenStatesKey, JSON.stringify(Array.from(seenStates)));
}

function markAsRead(branch) {
    let branchData = spell(branch, Branch);

    if (!seenStates.has(branchData.namedID) && seenCount > 0) {
        let badge = document.createElement("span");
        badge.classList.add("badge", "red", "before-content");
        badge.textContent = "new";

        branchData.branchContent.firstChild.insertBefore(badge, branchData.branchContent.firstChild.firstChild);
    }

    seenStates.add(branchData.namedID);
}

export function initNewsPage() {
    for (let [_, branch] of Branch.branchesByNamedID) {
        markAsRead(branch);
    }
    saveSeenStates();

    // If any branches are added past the initial load, add them to the seen set too.
    Branch.onAdded.push(branch => {
        markAsRead(branch);
        saveSeenStates();
    })
}

export function markAllAsUnread() {
    localStorage.removeItem(seenStatesKey);
}

addSpell("new", class New {
    constructor(element) {
        // Do not show the badge to people who have never seen any news.
        // It's just annoying in that case.
        // In case you do not wish to see the badge anymore, go to the news page and uncheck the
        // checkbox at the bottom.
        let userSawNews = seenCount > 0;
        let userWantsToSeeNews = getSettingValue("showNewPostIndicator");
        if (userSawNews && userWantsToSeeNews && unseenCount > 0) {
            this.newText = document.createElement("span");
            this.newText.classList.add("new-text");
            this.newText.textContent = element.textContent;
            element.textContent = "";
            element.appendChild(this.newText);

            this.badge = document.createElement("span");
            this.badge.classList.add("badge", "red");
            this.badge.textContent = unseenCount.toString();
            element.appendChild(this.badge);
            element.classList.add("has-news");
        }
    }
});

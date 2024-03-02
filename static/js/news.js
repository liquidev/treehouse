// news.js because new.js makes the TypeScript language server flip out.
// Likely because `new` is a keyword, but also, what the fuck.

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
    if (!seenStates.has(branch.namedID) && seenCount > 0) {
        let badge = document.createElement("span");
        badge.classList.add("badge", "red", "before-content");
        badge.textContent = "new";

        branch.branchContent.firstChild.insertBefore(badge, branch.branchContent.firstChild.firstChild);
    }

    seenStates.add(branch.namedID);
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

class New extends HTMLAnchorElement {
    connectedCallback() {
        // Do not show the badge to people who have never seen any news.
        // It's just annoying in that case.
        // In case you do not wish to see the badge anymore, go to the news page and uncheck the
        // checkbox at the bottom.
        let userSawNews = seenCount > 0;
        let userWantsToSeeNews = getSettingValue("showNewPostIndicator");
        if (userSawNews && userWantsToSeeNews && unseenCount > 0) {
            this.newText = document.createElement("span");
            this.newText.classList.add("new-text");
            this.newText.textContent = this.textContent;
            this.textContent = "";
            this.appendChild(this.newText);

            this.badge = document.createElement("span");
            this.badge.classList.add("badge", "red");
            this.badge.textContent = unseenCount.toString();
            this.appendChild(this.badge);
            this.classList.add("has-news");
        }
    }
}

customElements.define("th-new", New, { extends: "a" });

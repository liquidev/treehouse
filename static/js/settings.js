import { addSpell } from "treehouse/spells.js";

const settingsKey = "treehouse.settings";
const settings = JSON.parse(localStorage.getItem(settingsKey)) || {};

const defaultSettingValues = {
    showNewPostIndicator: true,
};

function saveSettings() {
    localStorage.setItem(settingsKey, JSON.stringify(settings));
}

export function getSettingValue(setting) {
    return settings[setting] ?? defaultSettingValues[setting];
}

class SettingCheckbox {
    constructor(element) {
        element.checked = getSettingValue(element.id);

        element.addEventListener("change", () => {
            settings[element.id] = element.checked;
            saveSettings();
        });
    }
}

addSpell("setting-checkbox", SettingCheckbox);

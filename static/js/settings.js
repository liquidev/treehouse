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

class SettingCheckbox extends HTMLInputElement {
    connectedCallback() {
        this.checked = getSettingValue(this.id);

        this.addEventListener("change", () => {
            settings[this.id] = this.checked;
            saveSettings();
        });
    }
}

customElements.define("th-setting-checkbox", SettingCheckbox, { extends: "input" });

class Settings extends HTMLElement {
    connectedCallback() {
        this.style.display = "block";
    }
}

customElements.define("th-settings", Settings, { extends: "section" });

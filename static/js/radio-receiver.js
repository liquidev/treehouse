function tuneIntoRadio() {
    let events = new EventSource(`${TREEHOUSE_SITE}/radio`);
    events.addEventListener("message", (event) => {
        // TODO: Interpret event.
        console.log("[radio index]", event.data);
    });
    return events;
}

// NOTE: The following mechanism's goal is to tune out of the radio if the tab is not open.
// Browsers prevent webpages from having too many connections open, therefore we only open one
// radio index connection per visible tab.
let radio = null;

function updateRadioToVisibility() {
    if (document.visibilityState == "visible") {
        if (radio != null) {
            radio.close();
        }
        console.log("Tuning into radio index...");
        radio = tuneIntoRadio();
    } else if (document.visibilityState == "hidden") {
        if (radio != null) {
            radio.close();
        }
        console.warn(
            "NOTE: Tuning out of radio indexing to save power. Keep the tab open to listen for radio stations.",
        );
        radio = null;
    }
}

updateRadioToVisibility();
addEventListener("visibilitychange", updateRadioToVisibility);

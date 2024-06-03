const DOOR_STATUS_FREQUENCY = `${TREEHOUSE_SITE}/radio/station/1395024484`;

let doorStatusSpan = document.getElementById("sandbox/maintenance:door-status");

// Poll for status every so often.
async function updateDoorStatus() {
    let doorStatus = await (await fetch(DOOR_STATUS_FREQUENCY)).text();
    doorStatusSpan.innerText = doorStatus;
    doorStatusSpan.classList.remove("green", "red");
    if (doorStatus == "open") {
        doorStatusSpan.classList.add("green");
    } else {
        doorStatusSpan.classList.add("red");
    }
}
updateDoorStatus();
setTimeout(updateDoorStatus, 5000);

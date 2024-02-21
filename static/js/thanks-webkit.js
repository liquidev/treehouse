// Detect if we can have crucial functionality (ie. custom elements call constructors).
// This doesn't seem to happen in Epiphany, and also other Webkit-based browsers.
let works = false;
class WebkitMoment extends HTMLLIElement {
    constructor() {
        super();
        works = true;
    }
}

customElements.define("th-webkit-moment", WebkitMoment, { extends: "li" });

let willItWorkOrWillItNot = document.createElement("div");
willItWorkOrWillItNot.innerHTML = `<li is="th-webkit-moment"></li>`;

// If my takeoff fails
// tell my mother I'm sorry
let box = document.getElementById("webkit-makes-me-go-insane");
if (!works) {
    box.style = "display: block";
}

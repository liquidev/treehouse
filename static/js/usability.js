// Bits and pieces to make vanilla HTML just a bit more usable.

// We want to let the user have a selection on collapsible blocks without collapsing them when
// the user finishes marking their selection.
document.addEventListener("click", event => {
    console.log(getSelection());
    if (getSelection().type == "Range") {
        event.preventDefault();
    }
})

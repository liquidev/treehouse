// Bits and pieces to make the treehouse just a bit more easy to explore.

// We want to let the user have a selection on collapsible blocks without collapsing them when
// the user finishes marking their selection.
document.addEventListener("click", event => {
    if (getSelection().type == "Range") {
        event.preventDefault();
    }
})

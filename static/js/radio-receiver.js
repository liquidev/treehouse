let events = new EventSource(`${TREEHOUSE_SITE}/radio`);

events.addEventListener("message", (event) => {
    // TODO: Interpret event.
    console.log(event);
});

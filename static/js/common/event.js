// Asyncifies an EventTarget.
//
// This allows for `await`ing a specific event's occurrence rather than needing to work with
// `addEventListener`, which can be finnicky in sequential-style code.
export function listen(target, ...events) {
    return new Promise((resolve) => {
        let removeAllEventListeners;

        let listeners = events.map((eventName) => {
            let listener = (event) => {
                removeAllEventListeners();
                resolve(event);
            };
            target.addEventListener(eventName, listener);
            return { eventName, func: listener };
        });

        removeAllEventListeners = () => {
            for (let listener of listeners) {
                target.removeEventListener(listener.eventName, listener.func);
            }
        };
    });
}

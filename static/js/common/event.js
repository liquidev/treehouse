export function createEvent(type, props) {
    let event = new Event(type);
    return Object.assign(event, props);
}

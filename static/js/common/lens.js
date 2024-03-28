export function constant(value) {
    return {
        get() {
            return value;
        },
        set(value) {},
    };
}

export function field(object, key) {
    return {
        get() {
            return object[key];
        },
        set(value) {
            object[key] = value;
        },
    };
}

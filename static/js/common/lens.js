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

export function listen(lens, callback) {
    return {
        get() {
            return lens.get();
        },

        set(value) {
            callback(lens, value);
            lens.set(value);
        },
    };
}

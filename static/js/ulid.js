let canonicalBase32 = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
let canonicalBase32Rev = {};
for (let i = 0; i < canonicalBase32.length; ++i) {
    canonicalBase32Rev[canonicalBase32.charAt(i)] = i;
}

// Works on BigInt to support numbers larger than (1 << 31).
function decodeCanonicalBase32(string) {
    let result = 0n;
    let cursor = 0b1n;
    for (let i = string.length - 1; i >= 0; --i) {
        result += cursor * BigInt(canonicalBase32Rev[string[i]]);
        cursor <<= 5n;
    }
    return result;
}

export function getTimestamp(ulid) {
    return new Date(Number(decodeCanonicalBase32(ulid.substring(0, 10))));
}

export function isCanonicalUlid(id) {
    if (id.length != 26) {
        return false;
    }
    for (let i = 0; i < id.length; ++i) {
        if (!canonicalBase32.includes(id.charAt(i))) {
            return false;
        }
    }
    return true;
}

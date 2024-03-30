export const characters = {
    coco: {
        name: "Coco",
    },
};

export function getCharacterPictureSrc(character, expression) {
    return `${TREEHOUSE_SITE}/static/character/${character}/${expression}.svg`;
}

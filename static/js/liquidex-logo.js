// Progressive enhancement: animated logo is only available if you have JS enabled, but the SVG
// fallback is used if you don't.

import { DpiIndependentCanvas } from "./components/dpi-independent-canvas.js";

let liquidexLDots = [
    [2, 5],
    [4, 5],
    [5, 5],
    [6, 5],
    [7, 5],
    [8, 5],
    [8, 4],
    [8, 3],
    [7, 3],
    [6, 3],
    [6, 4],
    [6, 6],
    [6, 7],
    [6, 8],
    [6, 9],
    [7, 9],
    [8, 9],
    [9, 9],
    [10, 9],
    [11, 9],
    [12, 9],
    [12, 8],
    [12, 7],
    [11, 7],
    [10, 7],
    [10, 8],
    [10, 10],
    [10, 11],
    [10, 13],
];

const sketches = [
    {
        animation: "hover",
        animationLength: 500,

        init: () => ({}),

        draw(state, dpic, ctx, t, h) {
            ctx.fillStyle = "currentColor";
            for (let i = 0; i < liquidexLDots.length; ++i) {
                let p = 1 - 2 * Math.abs(h - 0.5);
                let x = liquidexLDots[i][0],
                    y = liquidexLDots[i][1];
                let dy = Math.sin(t * 0.005 + x) * p * 2 * i;
                ctx.fillRect(x * 3, y * 3 + dy, 3 + 0.1, 3 + 0.1);
            }
        },
    },
];
let sketch = sketches[Math.floor(Math.random() * sketches.length)];
let sketchState = sketch.init();

document.addEventListener("DOMContentLoaded", () => {
    let logoSvg = document.getElementById("liquidex-logo");

    let state;
    let stateStartTime;
    let setState = (name) => {
        state = name;
        stateStartTime = performance.now();
    };
    setState("idle");

    let logo = new DpiIndependentCanvas({
        width: 48,
        height: 48,
        context: "2d",
        draw(dpic, canvas, ctx) {
            let t = performance.now();
            let h = 0;
            if (state == "hover") {
                h = Math.min(
                    1,
                    (performance.now() - stateStartTime) /
                        sketch.animationLength
                );
            } else if (state == "unhover") {
                h = Math.min(
                    1,
                    (performance.now() - stateStartTime) /
                        sketch.animationLength
                );
                h = 1 - h;
                if (h <= 0.0001) {
                    dpic.stopAnimating();
                }
            }

            ctx.clearRect(0, 0, dpic.width, dpic.height);
            sketch.draw(sketchState, dpic, ctx, t, h);
        },
    });
    logo.canvas.classList.add("logo");
    logoSvg.replaceWith(logo.canvas);

    logo.canvas.addEventListener("mouseenter", () => {
        setState("hover");
        logo.startAnimating();
    });

    logo.canvas.addEventListener("mouseleave", () => {
        setState("unhover");
    });
});

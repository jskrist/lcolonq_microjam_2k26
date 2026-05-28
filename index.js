import init, { greet, Universe } from "./pkg/lcolonq_codejam.js";

const pre = document.getElementById("game-of-life-canvas");
let universe;

const renderLoop = () => {
    pre.textContent = universe.render();
    universe.tick();
    requestAnimationFrame(renderLoop);
};

// Initialize the wasm module (explicit wasm path) before using any exported
// functions or types that rely on the `wasm` variable. Create the Universe
// after initialization and start the render loop.
init("./pkg/lcolonq_codejam_bg.wasm").then(() => {
    universe = Universe.new();
    requestAnimationFrame(renderLoop);
}).catch(console.error);

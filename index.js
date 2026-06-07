import init, { Universe, Pendulum, main } from "./pkg/lcolonq_codejam.js";

// Initialize the wasm module (explicit wasm path) before using any exported
// functions or types that rely on the `wasm` variable.
const wasm = await init("./pkg/lcolonq_codejam_bg.wasm");
const memory = wasm.memory;

const CELL_SIZE = 5; // px
const GRID_COLOR = "#0cd89b";
const DEAD_COLOR = "#0cd89b";
const ALIVE_COLOR = "#b61d7b";

// let memory = wasm.memory;
// Construct the universe, and get its width and height.
const pendulum = Pendulum.new(1, 1);
const universe = Universe.new();
const width = universe.width();
const height = universe.height();

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');
let im_data = ctx.getImageData(0, 0, width, height);

let count = 0;
let animationId = null;

let frame_num = 0;

document.addEventListener('mousemove', function(event) {
    var rect = canvas.getBoundingClientRect();
    pendulum.set_pivot_position(event.clientX - rect.left, event.clientY - rect.top);
});

const renderLoop = () => {
  // fps.render();
  if(count >= 1E0) {
    universe.tick();
    frame_num = frame_num + 1;
    // console.log("Frame: " + frame_num);
    pendulum.tick(0.001);
    // drawGrid();
    // drawCells();
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    drawPendulum();
    count = 0;
  }
  count += 1;

  animationId = requestAnimationFrame(renderLoop);
};

const isPaused = () => {
  return animationId === null;
};

const playPauseButton = document.getElementById("play-pause");

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
  console.log(main());
};

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
};

const bitIsSet = (n, arr) => {
  const byte = Math.floor(n / 8);
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
};

const drawPendulum = () => {
  const angle = pendulum.angle();
  const length = pendulum.length();

  let pivot_pos = pendulum.pivot_position();
  let o_x = pivot_pos[0]; 
  let o_y = pivot_pos[1]; 

  let LP = length * 200;

  let x = LP * Math.cos(angle) + o_x;
  let y = o_y - LP * Math.sin(angle);

  let pend_radius = Math.pow(1 / 1000, 1 / 3) * 200

  // draw the pin
  ctx.beginPath();
  ctx.arc(o_x, o_y, 1, 0, 2 * Math.PI);
  ctx.fillStyle = "black";
  ctx.fill();
  // draw the rod
  ctx.beginPath();
  ctx.moveTo(o_x, o_y);
  ctx.lineTo(x, y);
  ctx.stroke();
  // draw the mass at the end of the pendulum
  ctx.beginPath();
  ctx.arc(x, y, pend_radius, 0, 2 * Math.PI);
  ctx.fillStyle = "green";
  ctx.fill();
};

const drawCells = () => {
  const cellsPtr = universe.cells();
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

  ctx.fillStyle = DEAD_COLOR;
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.fillStyle = ALIVE_COLOR;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      if(!bitIsSet(idx, cells)) {
        continue
      }
      ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
      );
    }
  }
};

canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  universe.toggle_cell(row, col);

  drawGrid();
  drawCells();
});

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
    this.lastFPSRender = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    const fps_render_duration = now - this.lastFPSRender;
    this.lastFrameTimeStamp = now;
    const fps = 1 / delta * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;

    if (fps_render_duration > 160) {
      // Render the statistics.
      this.fps.textContent = `
  Frames per Second:
          latest = ${Math.round(fps)}
  avg of last 100 = ${Math.round(mean)}
  min of last 100 = ${Math.round(min)}
  max of last 100 = ${Math.round(max)}
  `.trim();
      this.lastFPSRender = now;
    }
  }
};



play();
import init, { Pendulum, Environment, add_bodies } from "./pkg/lcolonq_codejam.js";

// Initialize the wasm module (explicit wasm path) before using any exported
// functions or types that rely on the `wasm` variable.
const wasm = await init("./pkg/lcolonq_codejam_bg.wasm");
// const memory = wasm.memory;

// Construct the environment and create a pendulum.
const env = Environment.new();
add_bodies(env);
const width = 800;
const height = 500;
const y_divs = (env.pendulum.get_length() * 2) + 1;
const grid_size = height / y_divs;

// Give the canvas room for all the elements
const canvas = document.getElementById("canvas");
canvas.height = height;
canvas.width = width;

const ctx = canvas.getContext('2d');

let count = 0;
let animationId = null;

let mouse_pos = [0, 0];
let rect = canvas.getBoundingClientRect();

document.addEventListener('mousemove', function(event) {
    mouse_pos = [event.clientX - rect.left - canvas.width/2,
      event.clientY - rect.top - canvas.height/2];
    mouse_pos[0] = mouse_pos[0] / grid_size;
    mouse_pos[1] = mouse_pos[1] / grid_size;
});

const renderLoop = () => {
  if(count >= 1E0) {
    env.step(mouse_pos[0]);

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

const play = () => {
  renderLoop();
};

let btn_x = 0;
let btn_y = 0;
const btn_width = 40;
const btn_height = 20;
const drawStartButton = () => {
  const radii = 5;
  btn_x = canvas.width/2 - btn_width/2;
  btn_y = canvas.height/2 - btn_height/2;
  ctx.fillStyle = "blue";
  ctx.roundRect(btn_x, btn_y, btn_width, btn_height, radii)
  ctx.fill();
}
drawStartButton();

canvas.addEventListener('click', function(event) {
  let click_x = event.clientX - rect.left;
  let click_y = (event.clientY - rect.top)
  if(click_x > btn_x && click_x < btn_x + btn_width && 
    click_y > btn_y && click_y < btn_y + btn_height) {
      play();
  }
});

const drawPendulum = () => {
  let ball_pos = env.get_ball_pos();
  let x = ball_pos[0] * grid_size + canvas.width/2;
  let y = canvas.height - (ball_pos[1] * grid_size + canvas.height/2);
  let pivot_pos = env.get_pivot_pos();
  let o_x = pivot_pos[0] * grid_size + canvas.width/2;
  let o_y = canvas.height - (pivot_pos[1] * grid_size + canvas.height/2);

  let pend_radius = 0.25 * grid_size;

  // draw the pin
  ctx.beginPath();
  ctx.arc(o_x, o_y, grid_size * 0.0625, 0, 2 * Math.PI);
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

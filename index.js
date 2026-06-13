import init, { Environment } from "./pkg/lcolonq_codejam.js";

// Initialize the wasm module (explicit wasm path) before using any exported
// functions or types that rely on the `wasm` variable.
const wasm = await init("./pkg/lcolonq_codejam_bg.wasm");
// const memory = wasm.memory;

// Construct the environment and create a pendulum.
const env = Environment.new();
env.add_bodies();
const width = 240;
const height = 160;
const y_divs = (env.pendulum.get_length() * 2) + 2;
const grid_size = height / y_divs;

// Give the canvas room for all the elements
const canvas = document.getElementById("canvas");
canvas.height = height;
canvas.width = width;

const ctx = canvas.getContext('2d');
ctx.fillStyle = "white";
ctx.fillRect(0, 0, canvas.width, canvas.height);

let mouse_pos = [0, 0];
let rect = canvas.getBoundingClientRect();

function mouse_move_fcn(event) {
    mouse_pos = [event.clientX - rect.left - canvas.width/2,
      event.clientY - rect.top - canvas.height/2];
    mouse_pos[0] = mouse_pos[0] / grid_size;
    mouse_pos[1] = mouse_pos[1] / grid_size;
    env.set_pivot_position(mouse_pos[0]);
}

document.addEventListener('mousemove', mouse_move_fcn);

let start_time = 0;
function renderLoop(current_time) {
  if(current_time) {
    let dt = current_time - start_time;
    start_time = current_time;
    env.set_dt(dt / 1000.0);
    env.step(mouse_pos[0]);
    console.log("fps: " + 1000/dt)
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    drawPendulum();
  }
  requestAnimationFrame(renderLoop);
};

const play = () => {
  for(let i = 0; i < 1; i++) {
    renderLoop();
  }
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

function click_fcn(event) {
  let click_x = event.clientX - rect.left;
  let click_y = (event.clientY - rect.top)
  if(click_x > btn_x && click_x < btn_x + btn_width && 
    click_y > btn_y && click_y < btn_y + btn_height) {
      play();
      mouse_move_fcn(event);
      canvas.removeEventListener('click', click_fcn);
  }
}

canvas.addEventListener('click', click_fcn);

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
  let ball_color = "green";
  if(y > o_y) {
    ball_color = "red";
  }
  ctx.fillStyle = ball_color;
  ctx.fill();
};

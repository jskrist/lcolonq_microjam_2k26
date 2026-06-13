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

let power_level = 0.5;
let countdown = 20;
const ctx = canvas.getContext('2d');
drawBackground();
drawCountdown();
drawPowerBar();
drawPendulum();

let gameover = false;
let mouse_pos = [0, 0];
let rect = canvas.getBoundingClientRect();

function mouse_move_fcn(event) {
    mouse_pos = [event.clientX - rect.left - canvas.width/2,
      event.clientY - rect.top - canvas.height/2];

    mouse_pos[0] = Math.min(Math.max(mouse_pos[0], -canvas.width/2), canvas.width/2);
    mouse_pos[0] = mouse_pos[0] / grid_size;
    mouse_pos[1] = mouse_pos[1] / grid_size;
    env.set_pivot_position(mouse_pos[0]);
}

document.addEventListener('mousemove', mouse_move_fcn);

let start_time = 0;
function renderLoop(current_time) {
  if(current_time) {
    // limit the simulations to at most 100ms time steps
    let dt = Math.min(current_time - start_time, 100);
    start_time = current_time;
    env.set_dt(dt / 1000.0);
    env.step(mouse_pos[0]);
    // console.log("fps: " + 1000/dt)
    drawBackground();
    drawCountdown();
    drawPowerBar();
    drawPendulum();
  }
  if(!gameover) {
    requestAnimationFrame(renderLoop);
  }
};

function play() {
  setTimeout(update_power_bar, 120);
  setTimeout(update_countdown, 1000);
  renderLoop();
};

const btn_width = 40;
const btn_height = 20;
let btn_x = canvas.width/2 - btn_width/2;
let btn_y = canvas.height/2 - btn_height/2;
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

function drawBackground() {
  ctx.fillStyle = "white";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
}

function drawCountdown() {
  ctx.fillStyle = "black";
  ctx.font = "16px Consolas";
  ctx.fillText(String(countdown).padStart(2, '0'), 10, 20);
}

function drawPowerBar() {
  ctx.fillStyle = "gray";
  let powerbar_w = 10;
  const num_cells = 20;
  const cell_height = canvas.height / num_cells;
  let h = 0;
  ctx.fillStyle = "gray";
  ctx.fillRect(canvas.width - powerbar_w, 0, powerbar_w, canvas.height);
  let color = "red";
  // for(let i = 0; i < num_cells; i++) {
    let i = Math.floor(power_level * num_cells);
    h = canvas.height - (Math.max(i, 1) * cell_height);
    ctx.beginPath();
    ctx.rect(canvas.width - powerbar_w, h, powerbar_w, cell_height);
    if((i / num_cells) >= 0.5){
      color = "green";
    }
    // console.log("power_level: " + power_level + "\n val: " + (i / num_cells) + "\ncolor: " + color)
    ctx.fillStyle = color;
    ctx.fill();
    // ctx.stroke();
  // }
}

function drawPendulum() {
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

function mapRange(value, oldMin, oldMax, newMin, newMax) {
  return ((value - oldMin) * (newMax - newMin) / (oldMax - oldMin)) + newMin;
}

function message_handler(event) {
  console.log(event.data);
  switch(event.data.op) {
    case 'start':
      let grav_factor = mapRange(event.data.difficulty, 0, 100, 0, 5);
      console.log("grav_factor: " + grav_factor);
      env.set_gravity_factor(grav_factor);
      play();
      window.parent.postMessage({op: "started", verb: "Balance!"});
      break;
    default:
  }
}

function update_countdown() {
  countdown -= 1;
  if(countdown > 0) {
    setTimeout(update_countdown, 1000);
  }
  else {
    gameover = true; 
    window.parent.postMessage({op: "done", win: power_level >= 0.5});
  }
}

function update_power_bar() {
  let ball_pos = env.get_ball_pos();
  let direction = -2;
  if(ball_pos[1] > 0) {
    direction = 1;
  }
  let scale = 0.025;
  power_level += (scale * direction);
  power_level = Math.max(Math.min(power_level, 1), 0);
  setTimeout(update_power_bar, 120);
}

window.addEventListener("message", message_handler);
window.parent.postMessage({op: "ready"});
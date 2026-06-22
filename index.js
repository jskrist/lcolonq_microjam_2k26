import init, { Environment } from "./pkg/lcolonq_codejam.js";

// Initialize the wasm module (explicit wasm path) before using any exported
// functions or types that rely on the `wasm` variable.
const wasm = await init("./pkg/lcolonq_codejam_bg.wasm");
// const memory = wasm.memory;

// Construct the environment and create a pendulum.
const env = Environment.new();
env.add_bodies();
const y_divs = (env.pendulum.get_length() * 2) + 2;
const canvas = document.getElementById("canvas");

const MINFRAMERATE = Math.floor(1000/30);
// const width = 240;
// const height = 160;
let width = null;
let height = null;
let grid_size = null;
let power_level = null;
let countdown = null;
let gameover = null;
let is_playing = false;
let mouse_pos = null;
let powerbar_timeout = null;
let countdown_timeout = null;
let headsup_timeout = null;
let rect = null;
let start_time = 0;
let heads_up_dt = 0;
const ctx = canvas.getContext('2d');
window_resize([]);

function draw_scene() {
    drawBackground();
    drawCountdown();
    drawPowerBar();
    drawPendulum();
}


function window_resize(event) {
  width = Math.floor(document.body.clientWidth * 1);
  height = Math.floor(document.body.clientHeight * 1);
  if(width/height < 3/2) {
      height = width * 2/3;
  }
  else {
      width = height * 3/2;
  }

  grid_size = height / y_divs;
  canvas.height = height;
  canvas.width = width;
  rect = canvas.getBoundingClientRect();
  if(!powerbar_timeout) {
    draw_scene();
    mouse_pos = [0, 0];
  }
}
window.addEventListener("resize", window_resize);


function reset() {
  if(powerbar_timeout) {
    clearTimeout(powerbar_timeout);
    powerbar_timeout = null;
  }
  if(countdown_timeout) {
    clearTimeout(countdown_timeout);
    countdown_timeout = null;
  }
  if(headsup_timeout) {
    clearTimeout(headsup_timeout);
    headsup_timeout = null;
  }
  power_level = 0.5;
  countdown = 20;
  env.reset_scene();
  draw_scene();

  is_playing = false;
  gameover = false;
  heads_up_dt = 0;
  mouse_pos = [0, 0];
  start_time = performance.now();
}
reset();

function mouse_move_fcn(event) {
    mouse_pos = [event.clientX - rect.left - canvas.width/2,
      event.clientY - rect.top - canvas.height/2];

    mouse_pos[0] = Math.min(Math.max(mouse_pos[0], -canvas.width/2), canvas.width/2);
    mouse_pos[0] = mouse_pos[0] / grid_size;
    mouse_pos[1] = mouse_pos[1] / grid_size;
    env.set_pivot_position(mouse_pos[0]);
}

document.addEventListener('mousemove', mouse_move_fcn);

function renderLoop(current_time) {
  if(!is_playing) {
    return
  }
  if(current_time) {
    // limit the simulations to at most 100ms time steps
    let dt = Math.min(current_time - start_time, 100);
    start_time = current_time;
    env.set_dt(dt / 1000.0);
    env.step(mouse_pos[0]);
    // console.log("fps: " + 1000/dt)
    draw_scene();
  }
  if(is_playing && !gameover) {
    requestAnimationFrame(renderLoop);
  }
};

function play() {
  powerbar_timeout = setTimeout(update_power_bar, 120);
  countdown_timeout = setTimeout(update_countdown, 1000);
  is_playing = true;
  renderLoop();
};


function drawBackground() {
  ctx.fillStyle = "white";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
}

function drawCountdown() {
  ctx.fillStyle = "black";
  let font_height = Math.floor(height/10);
  ctx.font = Math.floor(height/10) + "px Consolas";
  ctx.fillText(String(countdown).padStart(2, '0'), 10, font_height);
}

function drawPowerBar() {
  ctx.fillStyle = "gray";
  let powerbar_w = 1/24*width;
  const num_cells = 20;
  const cell_height = canvas.height / num_cells;
  let h = 0;
  ctx.fillStyle = "gray";
  ctx.fillRect(canvas.width - powerbar_w, 0, powerbar_w, canvas.height);
  let color = "red";
  let i = Math.floor(power_level * num_cells);
  h = canvas.height - (Math.max(i, 1) * cell_height);
  ctx.beginPath();
  ctx.rect(canvas.width - powerbar_w, h, powerbar_w, cell_height);
  if((i / num_cells) >= 0.5){
    color = "green";
  }
  ctx.fillStyle = color;
  ctx.fill();
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
      env.set_gravity_factor(grav_factor);
      reset();
      headsup_timeout = setTimeout(player_heads_up, 1/30);
      window.parent.postMessage({op: "started", verb: "Balance!"});
      break;
    default:
  }
}

function player_heads_up() {
  heads_up_dt = heads_up_dt + MINFRAMERATE;
  console.log("dt: " + heads_up_dt);
  let radius = (Math.sin(Math.PI / 1000 * heads_up_dt) + 1) / 2;
  draw_scene();
  let x = canvas.width/2;
  let y = canvas.height/2;
  // draw indicator circle
  ctx.beginPath();
  ctx.arc(x, y, radius * grid_size/2, 0, 2 * Math.PI);
  ctx.stroke();
  if(heads_up_dt > 3000) {
    start_time = performance.now();
    play();
  }
  else {
    headsup_timeout = setTimeout(player_heads_up, MINFRAMERATE);
  }
};

function update_countdown() {
  countdown -= 1;
  if(countdown > 0) {
    countdown_timeout = setTimeout(update_countdown, 1000);
  }
  else {
    is_playing = false;
    gameover = true;
    draw_scene();
    window.parent.postMessage({op: "done", win: power_level >= 0.5});
  }
}

function update_power_bar() {
  let ball_pos = env.get_ball_pos();
  let direction = -2;
  if(ball_pos[1] > 0) {
    direction = 0.75;
  }
  let scale = 0.025;
  power_level += (scale * direction);
  power_level = Math.max(Math.min(power_level, 1), 0);
  if(!gameover) {
    powerbar_timeout = setTimeout(update_power_bar, 120);
  }
}

window.addEventListener("message", message_handler);
window.parent.postMessage({op: "ready"});
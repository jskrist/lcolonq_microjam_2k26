// Initial code based on examples from:
// https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm#building_our_webassembly_package
// https://rustwasm.github.io/book/game-of-life/implementing.html
extern crate fixedbitset;

use js_sys::Null;
use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[allow(unused_macros)]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// #[wasm_bindgen]
// extern "C" {
//     pub fn alert(s: &str);
// }

// #[wasm_bindgen]
// pub fn greet(name: &str) {
//     alert(&format!("Hello, {}!", name));
// }


#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
    }
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                next.set(idx, match (cell, live_neighbors) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise
                })
            }
        }

        self.cells = next;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [-1, 0, 1].iter() {
            for delta_col in [-1, 0, 1].iter() {
                if *delta_row == 0 && *delta_col == 0 {
                    continue;
                }

                let mut neighbor_row = row as i32 + *delta_row;
                if neighbor_row < 0 {
                    neighbor_row = neighbor_row + self.height as i32;
                }
                else if neighbor_row > self.height as i32 {
                    neighbor_row = neighbor_row - self.height as i32;
                }
                let mut neighbor_col = column  as i32 + *delta_col;
                if neighbor_col < 0 {
                    neighbor_col = neighbor_col + self.width as i32;
                }
                else if neighbor_col > self.width as i32 {
                    neighbor_col = neighbor_col - self.width as i32;
                }
                let idx = self.get_index(neighbor_row as u32, neighbor_col as u32);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for idx in 0..size {
            let rand_byte: u8 = (js_sys::Math::random() * 256.0).floor() as u8;
            cells.set(idx, rand_byte % 2 == 0 || rand_byte % 7 == 0 as u8);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr() as *const u32
    }

    pub fn render(&self) -> String {
        self.to_string()
    }
}

use std::{f64::consts::PI, fmt};
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

// Description:
//  inverted pendulum pivot controled by mouse, (1D or 2D)?
// Objective:
//  keep the stick and mass above the pivot until the timer gets to 0.
// Complications:
// wind?



#[wasm_bindgen]
pub struct Pendulum {
    length: f64,
    mass: f64,
    angle: f64,
    pivot_velocity: [f64;2],
    mass_velocity: [f64;2],
    angular_velocity: f64,
    pivot_position: [f64;2],
    prev_pivot_position: [f64;2],
    prev_pivot_velocity: [f64;2],
}

#[wasm_bindgen]
impl Pendulum {
    pub fn new(length: f64, mass: f64) -> Pendulum {
        // create a new Pendulum with the given length and mass
        // It will start pointing straight up, with a small random
        // nudge to the mass

        const MAX_X_VEL: f64 = 1E-2; // m/s
        let mass_vel_x= (js_sys::Math::random() * 2.0 - 1.0) * MAX_X_VEL;

        Pendulum {
            length,
            mass,
            angle: 90.1 * PI/180.0,
            pivot_velocity: [0.0, 0.0],
            mass_velocity: [mass_vel_x, 0.0],
            angular_velocity: 0.1,
            pivot_position: [0.0, 0.0],
            prev_pivot_velocity: [0.0, 0.0],
            prev_pivot_position: [0.0, 0.0],
        }
    }

    pub fn tick(&mut self, dt: f64) {
        // run a simulation for the given dt
        const G: f64 = 9.81; // m/s

        // Simulation variables (initial state)
        // use stored angular velocity so the state persists between ticks
        let mut omega = self.angular_velocity;       // pendulum angular velocity [rad/s]

        // Determine accelerations using a simple pendulum model with moving pivot.
        // mass position (relative to pivot): r = [cos(theta), sin(theta)] * length
        // approximate angular acceleration: -g/l * sin(theta) + pivot_effects - damping
        let cost = js_sys::Math::cos(self.angle);
        let sint = js_sys::Math::sin(self.angle);

        // pivot movement: update pivot position from pivot_velocity (set externally)
        // Determine current pivot velocity. If JS moved the pivot by calling
        // `set_pivot_position()` (so `pivot_position` changed since last tick),
        // derive velocity from that positional change. Otherwise, use
        // `pivot_velocity` (which may be set directly by JS) and advance
        // pivot_position by that velocity.
        let mut curr_pvx = self.pivot_velocity[0];
        let mut curr_pvy = self.pivot_velocity[1];
        if dt > 0.0 {
            let dx = self.pivot_position[0] - self.prev_pivot_position[0];
            let dy = self.pivot_position[1] - self.prev_pivot_position[1];
            let moved = dx.abs() > 1e-12 || dy.abs() > 1e-12;
            if moved {
                curr_pvx = dx / dt;
                curr_pvy = dy / dt;
            } else {
                // no explicit position change; advance pivot by velocity
                self.pivot_position[0] += self.pivot_velocity[0] * dt;
                self.pivot_position[1] += self.pivot_velocity[1] * dt;
            }
            // cap velocity derived from positional changes to avoid huge deltas
            const MAX_PIVOT_VEL: f64 = 20.0; // m/s, adjust as needed
            if curr_pvx.is_finite() {
                if curr_pvx > MAX_PIVOT_VEL { curr_pvx = MAX_PIVOT_VEL }
                if curr_pvx < -MAX_PIVOT_VEL { curr_pvx = -MAX_PIVOT_VEL }
            }
            if curr_pvy.is_finite() {
                if curr_pvy > MAX_PIVOT_VEL { curr_pvy = MAX_PIVOT_VEL }
                if curr_pvy < -MAX_PIVOT_VEL { curr_pvy = -MAX_PIVOT_VEL }
            }
        }

        // approximate pivot acceleration from change in velocity
        let mut pax = 0.0;
        let mut pay = 0.0;
        if dt > 0.0 {
            pax = (curr_pvx - self.prev_pivot_velocity[0]) / dt;
            pay = (curr_pvy - self.prev_pivot_velocity[1]) / dt;
        }
        // store current velocity/position for next step
        self.prev_pivot_velocity[0] = curr_pvx;
        self.prev_pivot_velocity[1] = curr_pvy;
        self.prev_pivot_position[0] = self.pivot_position[0];
        self.prev_pivot_position[1] = self.pivot_position[1];
        // reflect derived current velocity back into state
        self.pivot_velocity[0] = curr_pvx;
        self.pivot_velocity[1] = curr_pvy;

        // forcing from pivot acceleration projected onto pendulum direction (computed below)

        // damping
        let damping = 0.05;

        // angular acceleration
        // gravity term: projection of gravity (0,-G) onto tangent [-sin, cos] -> -G * cos(theta)
        let gravity_term = - (G / self.length) * cost;
        // pivot contribution: - (a_p · e_t) / l where e_t = [-sin, cos]
        let pivot_contrib = (pax * sint - pay * cost) / self.length;
        // theta_ddot = -g*cos/l + (pax*sin - pay*cos)/l - damping*omega
        let ang_acc = gravity_term + pivot_contrib - damping * omega;

        // Euler integration for angle and angular velocity
        omega += ang_acc * dt;
        self.angle += omega * dt;
        self.angular_velocity = omega;

        // update mass velocity from kinematics: v_mass = v_pivot + d/dt( r )
        // where d/dt( r ) = length * [-sin(theta)*omega, cos(theta)*omega]
        let vr_x = - self.length * js_sys::Math::sin(self.angle) * omega;
        let vr_y =   self.length * js_sys::Math::cos(self.angle) * omega;
        self.mass_velocity[0] = self.pivot_velocity[0] + vr_x;
        self.mass_velocity[1] = self.pivot_velocity[1] + vr_y;

        // Range check
        if self.angle > PI {
            self.angle -= 2.0 * PI;
        }
        if self.angle <= -PI {
            self.angle += 2.0 * PI;
        }
    }
    pub fn angle(&self) -> f64 {
        self.angle
    }
    pub fn length(&self) -> f64 {
        self.length
    }
    pub fn pivot_position(&self) -> Vec<f64> {
        self.pivot_position.to_vec()
    }
    pub fn set_pivot_position(&mut self, x: f64, y: f64) {
        // record previous position so `tick()` can compute velocity = (pos-pos_prev)/dt
        self.prev_pivot_position[0] = self.pivot_position[0];
        self.prev_pivot_position[1] = self.pivot_position[1];
        self.pivot_position[0] = x;
        self.pivot_position[1] = y;
    }
}


// use crate::utils::character;
// use crate::utils::character::CharacterControlMode;
// use rapier_testbed2d::Testbed;
use rapier2d::control::{KinematicCharacterController, PidController};
use rapier2d::prelude::*;

// pub fn ball_balance() {
//     /*
//      * World
//      */
//     // let mut world = PhysicsWorld::new();

//     /*
//      * Ground
//      */
//     let ground_length = 10.0;
//     let ground_height = 0.1;
//     let wall_height = 0.25;
//     let wall_thickness = ground_height;

//     // Floor
//     let rigid_body = RigidBodyBuilder::fixed().translation(Vector::new(0.0, -ground_height));
//     let collider = ColliderBuilder::cuboid(ground_length, ground_height);
//     let _ = world.insert(rigid_body, collider);
//     // Left wall
//     let rigid_body = RigidBodyBuilder::fixed()
//         .translation(Vector::new(-ground_length - wall_thickness, wall_height));
//     let collider = ColliderBuilder::cuboid(wall_thickness, wall_height);
//     let _ = world.insert(rigid_body, collider);
//     // right wall
//     let rigid_body = RigidBodyBuilder::fixed()
//         .translation(Vector::new(ground_length + wall_thickness, wall_height));
//     let collider = ColliderBuilder::cuboid(wall_thickness, wall_height);
//     let _ = world.insert(rigid_body, collider);

//     /*
//      * Character we will control manually.
//      */
//     let character_height = 0.25;
//     let character_width = 0.125;
//     let rigid_body =
//         RigidBodyBuilder::kinematic_position_based()
//         .translation(Vector::new(0.0, character_height));
//     let collider = ColliderBuilder::cuboid(character_width, character_height);
//     let (character_handle, _) = world.insert(rigid_body, collider);

//     // rod
//     let rod_length = 1.0;

//     /*
//      * Tethered Ball
//      */
//     let rad = 0.125;

//     let rigid_body =
//         RigidBodyBuilder::new(RigidBodyType::Dynamic)
//         .translation(Vector::new(0.0, 2.0 * rod_length + character_height))
//         .linear_damping(1.125 as f32);
//     let collider = ColliderBuilder::ball(rad).sensor(true);
//     let (child_handle, _) = world.insert(rigid_body, collider);

//     let joint = RevoluteJointBuilder::new()
//         .local_anchor2(Vector::new(0.0, -2.0 * rod_length));
//     world.insert_impulse_joint(character_handle, child_handle, joint);

//     /*
//      * Callback to update the character based on user inputs.
//      */
//     let mut control_mode = CharacterControlMode::Kinematic(0.075);
//     let mut controller = KinematicCharacterController::default();
//     let mut pid = PidController::default();

//     testbed.add_callback(move |graphics, physics, _, _| {
//         if let Some(graphics) = graphics {
//             character::update_character(
//                 graphics,
//                 physics,
//                 &mut control_mode,
//                 &mut controller,
//                 &mut pid,
//                 character_handle,
//             );
//         }
//     });

//     // /*
//     //  * Set up the testbed.
//     //  */
//     // testbed.set_physics_world(world);
//     // testbed.look_at(Vec2::new(0.0, 1.0), 100.0);
// }

#[wasm_bindgen]
pub fn main() -> f32 {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the ground. */
    let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
    collider_set.insert(collider);

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 10.0].into())
        .build();
    let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
    let ball_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

    /* Create other structures necessary for the simulation. */
    let gravity: Vec2 = vector![0.0, -9.81].into();
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    let mut ball_body = &RigidBodyBuilder::dynamic().build();
    /* Run the game loop, stepping the simulation once per frame. */
    for _ in 0..200 {
        physics_pipeline.step(
            gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );

        ball_body = &rigid_body_set[ball_body_handle];
        println!("Ball altitude: {}", ball_body.translation().y);
    }
    ball_body.translation().y
}
// Initial code based on examples from:
// https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm#building_our_webassembly_package
// https://rustwasm.github.io/book/game-of-life/implementing.html
extern crate fixedbitset;

// use crate::utils::character;
// use crate::utils::character::CharacterControlMode;
// use rapier_testbed2d::Testbed;
use rapier2d::prelude::*;

// use rapier2d::control::{KinematicCharacterController};
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

use std::{fmt};
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
pub struct Environment {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vec2,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
    pub pendulum: Pendulum,
}

#[wasm_bindgen]
impl Environment {
    pub fn new() -> Environment {
        let rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        /* Create the ground. */
        let ground_collider = ColliderBuilder::cuboid(100.0, 0.1).build();
        collider_set.insert(ground_collider);

        /* Create other structures necessary for the simulation. */
        let gravity: Vec2 = vector![0.0, -9.81].into();
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let physics_hooks = ();
        let event_handler = ();

       Environment {
            rigid_body_set,
            collider_set,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            physics_hooks,
            event_handler,
            pendulum: Pendulum::new(2.0)
        }
    }
    pub fn step(& mut self, x: f32) {

        // update the pivot location
        self.set_pivot_position(x);

        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
    pub fn get_pivot_pos(& self) -> Vec<f32> {
        if let Some(body) = self.rigid_body_set.get(self.pendulum.pivot) {
            body.position().translation.to_array().to_vec()
        }
        else {
            vec![0.0, 0.0]
        }
    }
    pub fn get_ball_pos(& self) -> Vec<f32> {
        if let Some(body) = self.rigid_body_set.get(self.pendulum.ball) {
            body.position().translation.to_array().to_vec()
        }
        else {
            vec![0.0, 0.0]
        }
    }
    pub fn set_pivot_position(&mut self, x: f32) {
        if let Some(body) = self.rigid_body_set.get_mut(self.pendulum.pivot) {
            let pos = body.translation();
            body.set_position(
                Pose2::translation(x, pos.y),
                true, // wake up the body if sleeping
            );
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Pendulum {
    pub length: f32,
    pivot: RigidBodyHandle,
    ball: RigidBodyHandle,
}

#[wasm_bindgen]
impl Pendulum {
    pub fn new(length: f32) -> Pendulum {
        // create a new Pendulum with the given length and mass
        // It will start pointing straight up, with a small random
        // nudge to the mass
        Pendulum {
            length,
            pivot: RigidBodyHandle::default(),
            ball: RigidBodyHandle::default(),
        }
    }
    pub fn get_pivot_position(&self, env: Environment) -> Vec<f32> {
        let pivot_body = env.rigid_body_set[self.pivot].clone();
        pivot_body.translation().to_array().to_vec()
    }
    pub fn set_pivot_position(&mut self, mut env: Environment, x: f32, y: f32) {
        let pivot_body = &mut env.rigid_body_set[self.pivot];
        pivot_body.set_next_kinematic_translation(Vec2 { x, y });
    }
    pub fn get_ball_position(&self, env: Environment) -> Vec<f32> {
        let ball_body = &env.rigid_body_set[self.ball];
        ball_body.translation().to_array().into()
    }
    pub fn get_length(&self) -> f32 {
        self.length
    }
}

#[wasm_bindgen]
pub fn add_bodies(env: &mut Environment) {

    let character_height = 0.25;
    let character_width = 0.125;
    let rigid_body =
        RigidBodyBuilder::kinematic_position_based()
        .translation(Vector::new(0.0, character_height));
    let collider = ColliderBuilder::cuboid(character_width, character_height);

    let character_rb_handle = env.rigid_body_set.insert(rigid_body);
    env.collider_set.insert_with_parent(collider, character_rb_handle, &mut env.rigid_body_set);

    // rod
    let rod_length = env.pendulum.length;

    /*
     * Tethered Ball
     */
    let rad = 0.125;

    let rigid_body =
        RigidBodyBuilder::new(RigidBodyType::Dynamic)
        .translation(Vector::new(0.1, rod_length + character_height))
        .linear_damping(1.25);
        // .angular_damping(1000.0);
        // .linear_damping(0.01 as f32);
    let collider = ColliderBuilder::ball(rad).sensor(true);

    let ball_rb_handle = env.rigid_body_set.insert(rigid_body);
    env.collider_set.insert_with_parent(collider, ball_rb_handle, &mut env.rigid_body_set);

    let joint = RevoluteJointBuilder::new()
        .local_anchor2(Vector::new(0.0, -rod_length));
    env.impulse_joint_set.insert(character_rb_handle, ball_rb_handle, joint, true);

    env.pendulum.pivot = character_rb_handle;
    env.pendulum.ball = ball_rb_handle;
}
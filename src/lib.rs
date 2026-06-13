// Initial code based on examples from:
// https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm#building_our_webassembly_package
// https://rustwasm.github.io/book/game-of-life/implementing.html

use rapier2d::prelude::*;
use wasm_bindgen::prelude::*;

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
        let ground_collider: Collider = ColliderBuilder::cuboid(100.0, 0.1).mass(100.0).build();
        collider_set.insert(ground_collider);

        let grav_factor = 1.0;
        /* Create other structures necessary for the simulation. */
        let gravity: Vec2 = vector![0.0, -9.81 * grav_factor].into();
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
    pub fn set_dt(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
    }
    pub fn set_gravity_factor(&mut self, g_factor: f32) {
        if g_factor > 0.0 {
            self.gravity[1] = -9.8 * g_factor;
        }
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
    fn set_ball_position(&mut self, x: f32, y: f32) {
        if let Some(body) = self.rigid_body_set.get_mut(self.pendulum.ball) {
            body.set_position(
                Pose2::translation(x, y),
                true, // wake up the body if sleeping
            );
        }
    }
    pub fn reset_scene(&mut self) {
        self.set_pivot_position(0.0);
        self.set_ball_position(0.1, self.pendulum.length);
    }
    pub fn add_bodies(&mut self) {

        let character_height = 0.0;
        let character_width = 0.125;
        let rigid_body =
            RigidBodyBuilder::kinematic_position_based();
        let collider = ColliderBuilder::cuboid(character_width, character_height).mass(0.001);
        let character_rb_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, character_rb_handle, &mut self.rigid_body_set);

        // rod
        let rod_length = self.pendulum.length;

        /*
        * Tethered Ball
        */
        let rad = 0.125;

        let rigid_body =
            RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .translation(Vector::new(0.1, rod_length + character_height))
            .linear_damping(1.0);
            // .angular_damping(1000.0);
        let collider = ColliderBuilder::ball(rad).sensor(true).mass(2.3);

        let ball_rb_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, ball_rb_handle, &mut self.rigid_body_set);

        let joint = RevoluteJointBuilder::new()
            .local_anchor2(Vector::new(0.0, -rod_length));
            // .motor_velocity(0.00, 5e2);
        self.impulse_joint_set.insert(character_rb_handle, ball_rb_handle, joint, true);

        self.pendulum.pivot = character_rb_handle;
        self.pendulum.ball = ball_rb_handle;
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
    pub fn get_length(&self) -> f32 {
        self.length
    }
}

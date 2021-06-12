use bevy::prelude::*;
use crate::trans_rot_complexes::*;

// simulated particle
#[derive(Clone)]
pub struct Particle {
    pub neighbors: usize,
    mass: f32,
    moment_inertia: f32,
    pos: TRC,
    vel: TRCInfintesimal,
}

impl Particle {
    // Create a particle with mass = 1, at the origin, and resting
    // Parameters can be set using the corresponding builders
    pub fn new() -> Self {
        Self {
            neighbors: 0,
            mass: 1.0,
            moment_inertia: 1.0,
            pos: TRC::IDENTITY,
            vel: TRCInfintesimal::ZERO,
        }
    }

    //////////////////////
    // Builders
    // Use these after initialization to specify relevant attributes
    //

    pub fn set_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        return self;
    }

    pub fn set_moment_inertia(mut self, moment_inertia: f32) -> Self {
        self.moment_inertia = moment_inertia;
        return self;
    }

    pub fn set_pos_translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.pos.translation = Vec3::new(x, y, z);
        return self;
    }

    pub fn set_pos_rotation(mut self, val: Quat) -> Self {
        self.pos.rotation = val;
        return self;
    }

    pub fn set_vel_translation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.vel.translation = Vec3::new(x, y, z);
        return self;
    }

    pub fn set_vel_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.vel.rotation = Vec3::new(x, y, z);
        return self;
    }

    /////////////////////////
    // Getters
    //

    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn get_moment_inertia(&self) -> f32 {
        self.moment_inertia
    }

    pub fn get_pos(&self) -> TRC {
        self.pos
    }

    pub fn get_vel(&self) -> TRCInfintesimal {
        self.vel
    }

    //////////////////////////
    // Steppers
    // Step the relevant quantities through time
    //

    pub fn step_pos(&mut self, dt: f32, coeff: f32) {
        self.pos += self.vel.integrate(dt * coeff);
    }

    pub fn step_vel(&mut self, acc: TRCInfintesimal, dt: f32, coeff: f32) {
        self.vel += acc * dt * coeff;
    }

    pub fn heat(&mut self, dt: f32, amount: f32) {
        self.vel += self.vel * amount * dt;
    }
}

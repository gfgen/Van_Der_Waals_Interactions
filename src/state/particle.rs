use bevy::prelude::Vec3;

// simulated particle
#[derive(Clone)]
pub struct Particle {
    mass: f32,
    pos: Vec3,
    vel: Vec3,
}

impl Particle {
    // Create a particle with mass = 1, at the origin, and resting
    // Parameters can be set using the corresponding builders
    pub fn new() -> Self {
        Self {
            mass: 1.0,
            pos: Vec3::new(0.0, 0.0, 0.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    //////////////////////
    // Builders
    // Use these after initialization to specify relevant attributes
    //

    pub fn set_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        return  self;
    }

    pub fn set_pos(mut self, x: f32, y: f32, z: f32) -> Self {
        self.pos = Vec3::new(x, y, z);
        return  self;
    }

    pub fn set_vel(mut self, x: f32, y: f32, z: f32) -> Self {
        self.vel = Vec3::new(x, y, z);
        return  self;
    }

    /////////////////////////
    // Getters
    //
    
    pub fn get_mass(&self) -> f32 {
        self.mass
    }

    pub fn get_pos(&self) -> Vec3 {
        self.pos
    }

    pub fn get_vel(&self) -> Vec3 {
        self.vel
    }

    //////////////////////////
    // Steppers
    // Step the relevant quantities through time
    //

    pub fn step_pos(&mut self, dt: f32, coeff: f32) { 
        self.pos += coeff * dt * self.vel;
    }

    pub fn step_vel(&mut self, acc: Vec3, dt: f32, coeff: f32) {
        self.vel += coeff * dt * acc;
    }
}



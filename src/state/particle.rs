use na::Vector3;

// simulated particle
#[derive(Clone)]
pub struct Particle {
    mass: f64,
    pos: Vector3<f64>,
    vel: Vector3<f64>,
}

impl Particle {
    // Create a particle with mass = 1, at the origin, and resting
    // Parameters can be set using the corresponding builders
    pub fn new() -> Self {
        Self {
            mass: 1.0,
            pos: Vector3::new(0.0, 0.0, 0.0),
            vel: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    //////////////////////
    // Builders
    // Use these after initialization to specify relevant attributes
    //

    pub fn set_mass(mut self, mass: f64) -> Self {
        self.mass = mass;
        return  self;
    }

    pub fn set_pos(mut self, x: f64, y: f64, z: f64) -> Self {
        self.pos = Vector3::new(x, y, z);
        return  self;
    }

    pub fn set_vel(mut self, x: f64, y: f64, z: f64) -> Self {
        self.vel = Vector3::new(x, y, z);
        return  self;
    }

    /////////////////////////
    // Getters
    //
    
    pub fn get_mass(&self) -> f64 {
        self.mass
    }

    pub fn get_pos(&self) -> &Vector3<f64> {
        &self.pos
    }

    pub fn get_vel(&self) -> &Vector3<f64> {
        &self.vel
    }

    //////////////////////////
    // Steppers
    // Step the relevant quantities through time
    //

    pub fn step_pos(&mut self, dt: f64, coeff: f64) { 
        self.pos += coeff * dt * self.vel;
    }

    pub fn step_vel(&mut self, acc: &Vector3<f64>, dt: f64, coeff: f64) {
        self.vel += coeff * dt * acc;
    }
}



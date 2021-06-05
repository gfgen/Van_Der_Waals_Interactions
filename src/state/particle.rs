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

    pub fn set_pos(mut self, pos: Vector3<f64>) -> Self {
        self.pos = pos;
        return  self;
    }

    pub fn set_vel(mut self, vel: Vector3<f64>) -> Self {
        self.vel = vel;
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


////////////////////////////////////
// A wrapper that contains a potential and its corresponding gradient
// Both are Closures/Function pointers

pub struct Potential<F1, F2>
where F1: Fn(&Particle, &Particle) -> f64,
      F1: Send + Sync + 'static,
      F2: Fn(&Particle, &Particle) -> Vector3<f64>,
      F2: Send + Sync + 'static,
{
    func: F1,
    grad: F2,
}

impl<F1, F2> Potential<F1, F2> 
where F1: Fn(&Particle, &Particle) -> f64,
      F1: Send + Sync + 'static,
      F2: Fn(&Particle, &Particle) -> Vector3<f64>,
      F2: Send + Sync + 'static,
{
    pub fn eval(&self, target: &Particle, other: &Particle) -> f64 {
        let func = &self.func;
        func(target, other)
    }

    pub fn grad(&self, target: &Particle, other: &Particle) -> Vector3<f64> {
        let grad = &self.grad;
        grad(target, other)
    }
}

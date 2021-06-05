mod sim_space;
pub mod particle;
pub mod error;

use na::Vector3;
use sim_space::*;
use particle::*;
use error::*;
use rayon::prelude::*;

/////////////////////////////////////////////////
// Contains all simulation initial conditions
// Need to be compiled into a State to be useable
//
struct StatePrototype {
    bound: Boundary,             // location of the 6 walls of the box
    ext_t: f64,                  // external temperature
    ext_cond: f64,               // the rate of kinetic energy transfer from the outside

    grid_unit_size: f64,         // how big a grid point is
    grid_reach: usize,           // particle interaction cutoff
    dt: f64,                     // time step
    ext_a: Vector3<f64>,         // external acceleration applied to all particles
    particles: Vec<Particle>,
}

impl StatePrototype {
    // Create a new StatePrototype with default settings
    // Parameters can be changed using builders
    pub fn new() -> Self {
        Self {
            bound: Boundary::new(),
            ext_t: 0.0,
            ext_cond: 0.0,

            grid_unit_size: 1.0,
            grid_reach: 1,
            dt: 0.001,
            ext_a: Vector3::new(0.0, 0.0, 0.0),
            particles: Vec::new(),
        }
    }

    /////////////////////////////
    // Builders
    // Use these after initialization to specify relevant attributes
    //
    
    //
    // Builders for Boundary
    //
    
    // Bound
    pub fn set_bound_x(mut self, val: f64) -> Self {
        self.bound.x = val;
        self
    }

    pub fn set_bound_y(mut self, val: f64) -> Self {
        self.bound.y = val;
        self
    }
    pub fn set_bound_z(mut self, val: f64) -> Self {
        self.bound.z = val;
        self
    }

    // ext_t and ext_cond
    pub fn set_ext_t(mut self, ext_t: f64) -> Self {
        self.ext_t = ext_t;
        self
    }

    pub fn set_ext_cond(mut self, ext_cond: f64) -> Self {
        self.ext_cond = ext_cond;
        self
    }

    //
    // Builder for Grid
    //
    pub fn set_grid_unit_size(mut self, unit_size: f64) -> Self {
        self.grid_unit_size = unit_size;
        self
    }

    pub fn set_grid_reach(mut self, reach: usize) -> Self {
        self.grid_reach = reach;
        self
    }

    //
    // Builder for other values
    //
    pub fn set_dt(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
        
    }

    pub fn set_ext_a(mut self, ext_a: Vector3<f64>) -> Self {
        self.ext_a = ext_a;
        self
    }

    pub fn set_particles(mut self, particles: Vec<Particle>) -> Self {
        self.particles = particles;
        self
    }

    ////////////////
    // Compilation
    // Check for consistency and create a State
    // TODO: incorporate potential in the compilation process
    //
    pub fn compile(&self) -> Result<State, InvalidParamError> {
        let mut errors = Vec::new();

        if !self.bound.is_valid()        { errors.push(ErrorKind::Bound); }
        if self.ext_t < 0.0              { errors.push(ErrorKind::ExtT); }
        if self.ext_cond < 0.0           { errors.push(ErrorKind::ExtCond); }
        if self.grid_unit_size < 0.0     { errors.push(ErrorKind::UnitSize); }
        if self.grid_reach < 1           { errors.push(ErrorKind::Reach); }
        if self.dt <= 0.0                { errors.push(ErrorKind::Dt); }

        if !self.particles.iter()
            .map(|x| self.bound.contains_position(x.get_pos()))
            .fold(true, 
                |acc, x| acc && x) { errors.push(ErrorKind::Particle); }

        // Confirm errors and return
        if errors.is_empty() {
            Err(InvalidParamError::new(errors))
        } else {
            Ok(State::new(
                self.bound,
                self.ext_t,
                self.ext_cond,
                Grid::new(self.grid_unit_size, self.grid_reach),
                self.dt,
                self.ext_a,
                self.particles.clone(),
            ))
        }
    }
}

//////////////////////////////////////////////////////////////
// State contains all simulation parameters and particle data
// Can only be created by compiling a StatePrototype
//
pub struct State 
{
    // Parameters
    bound: Boundary,             // location of the 6 walls of the box
    ext_t: f64,                  // external temperature
    ext_cond: f64,               // the rate of kinetic energy transfer from the outside
    grid: Grid,
    dt: f64,
    ext_a: Vector3<f64>,         // external acceleration applied to all particles

    // Particles
    particles: Vec<Particle>,
}

impl State
{
    // Make a new State 
    // This function is only used by StatePrototype's compile method
    fn new(bound: Boundary, ext_t: f64, ext_cond: f64, 
        grid: Grid, dt: f64, ext_a: Vector3<f64>,
        particles: Vec<Particle>) -> Self {
        Self {
            bound,
            ext_t,
            ext_cond,
            grid,
            dt,
            ext_a,

            particles,
        }
    }

    // Render the current state using the anim format
    // TODO: Implement
    pub fn anim_render(&self) {
    }
  
    // Execute one time step
    // For now only uses leapfrog
    pub fn step(&mut self) {
        let dt = self.dt;

        self.particles.par_iter_mut()
            .for_each(|particle| particle.step_pos(dt, 0.5));

        let accelerations = self.calculate_particle_acceleration();
    }

    // Return a list of acceleration correspond to each particle
    // TODO: Also return energy and pressure data
    fn calculate_particle_acceleration(&self) 
    -> Vec<Vector3<f64>>
    {
        let bound_force = self.bound.calculate_force(&self.particles); 
        let grid_force = self.grid.calculate_force(&self.particles);
       
        let acceleration = (&self.particles, &bound_force, &grid_force).into_par_iter()
           // @param bnd_f: force on particle by the bounding box
           // @param grd_f: force on particle by other particles as calculated through the grid
           .map(|(particle, bnd_f, grd_f)| (bnd_f + grd_f) / particle.get_mass() + self.ext_a)
           .collect();

        acceleration
    }

}


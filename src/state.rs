pub mod error;
pub mod particle;
pub mod state_generator;
mod sim_space;

use bevy::prelude::Vec3;
use error::*;
use particle::*;
use rayon::prelude::*;
use sim_space::*;
use std::collections::VecDeque;

/////////////////////////////////////////////////
// Contains all simulation initial conditions
// Need to be compiled into a State to be useable
//
pub struct StatePrototype {
    bound: Boundary, // location of the 6 walls of the box
    target_temp: f32,      // external temperature
    inject_rate: f32,   // the rate of kinetic energy transfer from the outside

    grid_unit_size: f32, // how big a grid point is
    grid_reach: usize,   // particle interaction cutoff
    dt: f32,             // time step
    ext_a: Vec3,         // external acceleration applied to all particles
    particles: Vec<Particle>,
}

impl StatePrototype {
    // Create a new StatePrototype with default settings
    // Parameters can be changed using builders
    pub fn new() -> Self {
        Self {
            bound: Boundary::new(),
            target_temp: 0.0,
            inject_rate: 0.0,

            grid_unit_size: 1.0,
            grid_reach: 1,
            dt: 0.001,
            ext_a: Vec3::new(0.0, 0.0, 0.0),
            particles: Vec::new(),
        }
    }

    ///////////////////////////
    // Getters
    //
    pub fn get_bound(&self) -> Boundary {
        self.bound
    }

    /////////////////////////////
    // Builders
    // Use these after initialization to specify relevant attributes
    //

    //
    // Builders for Boundary
    //

    // Bound
    pub fn set_bound_x(mut self, val: f32) -> Self {
        self.bound.x = val;
        self
    }

    pub fn set_bound_y(mut self, val: f32) -> Self {
        self.bound.y = val;
        self
    }
    pub fn set_bound_z(mut self, val: f32) -> Self {
        self.bound.z = val;
        self
    }

    // target_temp and inject_rate
    pub fn set_target_temp(mut self, target_temp: f32) -> Self {
        self.target_temp = target_temp;
        self
    }

    pub fn set_inject_rate(mut self, inject_rate: f32) -> Self {
        self.inject_rate = inject_rate;
        self
    }

    //
    // Builder for Grid
    //
    pub fn set_grid_unit_size(mut self, unit_size: f32) -> Self {
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
    pub fn set_dt(mut self, dt: f32) -> Self {
        self.dt = dt;
        self
    }

    pub fn set_ext_a(mut self, ext_a: Vec3) -> Self {
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
    //
    pub fn compile(&self) -> Result<State, InvalidParamError> {
        let mut errors = Vec::new();

        if !self.bound.is_valid() {
            errors.push(ErrorKind::Bound);
        }
        if self.target_temp < 0.0 {
            errors.push(ErrorKind::TargTemp);
        }
        if self.inject_rate < 0.0 {
            errors.push(ErrorKind::InjectRate);
        }
        if self.grid_unit_size < 0.0 {
            errors.push(ErrorKind::UnitSize);
        }
        if self.grid_reach < 1 {
            errors.push(ErrorKind::Reach);
        }
        if self.dt <= 0.0 {
            errors.push(ErrorKind::Dt);
        }

        if !self
            .particles
            .iter()
            .map(|x| self.bound.contains_position(x.get_pos()))
            .fold(true, |acc, x| acc && x)
        {
            errors.push(ErrorKind::Particle);
        }

        // Confirm errors and return
        if !errors.is_empty() {
            Err(InvalidParamError::new(errors))
        } else {
            Ok(State::new(
                self.bound,
                self.target_temp,
                self.inject_rate,
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
// To be used as resources in bevy
// Can only be created by compiling a StatePrototype
//
pub struct State {
    bound: Boundary, // location of the 6 walls of the box
    target_temp: f32,      // external temperature
    inject_rate: f32,   // the rate of kinetic energy transfer from the outside
    grid: Grid,
    dt: f32,
    ext_a: Vec3, // external acceleration applied to all particles

    pot_energy: f32,
    kin_energy: f32,
    pressure: VecDeque<f32>,

    pub particles: Vec<Particle>,
}

impl State {
    // Make a new State
    // This function is only used by StatePrototype's compile method
    fn new(
        bound: Boundary,
        target_temp: f32,
        inject_rate: f32,
        grid: Grid,
        dt: f32,
        ext_a: Vec3,
        particles: Vec<Particle>,
    ) -> Self {
        Self {
            bound,
            target_temp,
            inject_rate,
            grid,
            dt,
            ext_a,

            pot_energy: 0.0,
            kin_energy: 0.0,
            pressure: VecDeque::new(),

            particles,
        }
    }

    ///////////////////////////
    // Getters
    //
    pub fn get_bound(&self) -> Boundary {
        self.bound
    }

    // Execute one time step
    // For now only uses leapfrog
    pub fn step(&mut self) {
        let dt = self.dt;

        // step position
        self.particles
            .par_iter_mut()
            .for_each(|particle| particle.step_pos(dt, 0.5));

        // calculate accelerations and step velocity
        let (accelerations, pot_enery, impulse) = self.calculate_particle_acceleration();
        (&mut self.particles, accelerations)
            .into_par_iter()
            .for_each(|(particle, acc)| particle.step_vel(acc, dt, 1.0));

        // step position again
        self.particles
            .par_iter_mut()
            .for_each(|particle| particle.step_pos(dt, 0.5));
    }

    // Return a list of acceleration correspond to each particle
    // Return the potential energy and pressure of the system
    // TODO: Also return energy and pressure data
    fn calculate_particle_acceleration(&self) -> (Vec<Vec3>, f32, f32) {
        let particle_pos = self
            .particles
            .iter()
            .map(|particle| particle.get_pos())
            .collect();
        let bound_force = self.bound.calculate_force(&particle_pos);
        let (grid_force, potential_energies) = self.grid.calculate_force(&particle_pos);

        let accelerations = (&self.particles, &bound_force, &grid_force)
            .into_par_iter()
            // @param bnd_f: force on particle by the bounding box
            // @param grd_f: force on particle by other particles as calculated through the grid
            .map(|(particle, &bnd_f, &grd_f)| (bnd_f + grd_f) / particle.get_mass() + self.ext_a)
            .collect();

        // calculate pressure and potential energy
        // TODO: to be recorded
        let potential_energy: f32 = potential_energies.iter().sum();
        let impulse: f32 = bound_force
            .iter()
            .map(|bnd_f| bnd_f.length() * self.dt)
            .sum();

        (accelerations, potential_energy, impulse)
    }
}

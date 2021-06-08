pub mod error;
pub mod state_generator;
mod particle;
mod physics;
mod sim_space; 
mod ui_systems;
mod sim_systems;

use bevy::prelude::*;
use error::*;
use particle::*;
use sim_space::*;
use std::collections::VecDeque;

/////////////////////////////////////////////////
// Contains all simulation initial conditions
// Need to be compiled into a State to be useable
//
pub struct SimulationPrototype {
    bound: Boundary, // location of the 6 walls of the box

    grid_unit_size: f32, // how big a grid point is
    grid_reach: usize,   // particle interaction cutoff
    dt: f32,             // time step
    ext_a: Vec3,         // external acceleration applied to all particles
    particles: Vec<Particle>,
}

impl SimulationPrototype {
    // Create a new StatePrototype with default settings
    // Parameters can be changed using builders
    pub fn new() -> Self {
        Self {
            bound: Boundary::new(),

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
    pub fn compile(&self) -> Result<VDWSimulation, InvalidParamError> {
        let mut errors = Vec::new();

        if !self.bound.is_valid() {
            errors.push(ErrorKind::Bound);
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
            Ok(VDWSimulation::new(
                self.bound,
                Grid::new(self.grid_unit_size, self.grid_reach),
                self.dt,
                self.ext_a,
                self.particles.clone(),
            ))
        }
    }
}


/////////////////////////////
// State component wrappers

#[derive(Clone, Copy)]
pub struct BoundRate(pub f32);
#[derive(Clone, Copy)]
pub struct TargetTemp(pub f32);
#[derive(Clone, Copy)]
pub struct InjectRate(pub f32);
#[derive(Clone, Copy)]
pub struct Dt(pub f32);
#[derive(Clone, Copy)]
pub struct ExtAccel(pub Vec3);
#[derive(Clone, Copy)]
pub struct PotentialEnergy(pub f32);
#[derive(Clone, Copy)]
pub struct KineticEnergy(pub f32);
pub struct Pressure(pub VecDeque<f32>);
pub struct EnergyHistory(pub VecDeque<f32>);
//////////////////////////////////////////////////////////////
// State contains all simulation parameters and particle data
// Is a bevy Plugin
// Can only be created by compiling a StatePrototype
//
pub struct VDWSimulation {
    bound: Boundary, // location of the 6 walls of the box
    grid: Grid,
    dt: Dt,
    ext_accel: ExtAccel, // external acceleration applied to all particles

    pub particles: Vec<Particle>,
}

impl VDWSimulation {
    // Make a new State
    // This function is only used by StatePrototype's compile method
    fn new(
        bound: Boundary,
        grid: Grid,
        dt: f32,
        ext_accel: Vec3,
        particles: Vec<Particle>,
    ) -> Self {
        Self {
            bound,
            grid,
            dt: Dt(dt),
            ext_accel: ExtAccel(ext_accel),

            particles,
        }
    }
}

impl Plugin for VDWSimulation {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(self.bound)
           .insert_resource(self.grid)
           .insert_resource(self.dt)
           .insert_resource(self.ext_accel)
           .insert_resource(self.particles.clone())

           .insert_resource(PotentialEnergy(0.0))
           .insert_resource(KineticEnergy(0.0))
           .insert_resource(Pressure(VecDeque::new()))
           .insert_resource(TargetTemp(0.0))
           .insert_resource(InjectRate(0.0))
           .insert_resource(BoundRate(0.0))
           .insert_resource(EnergyHistory(VecDeque::new()))
           
           .add_startup_system(sim_systems::setup_bounding_box.system())
           .add_startup_system(sim_systems::setup_particles.system())
           .add_startup_system(sim_systems::setup_camera.system())
           
           .add_system(ui_systems::param_sliders.system())
           .add_system(ui_systems::simulation_info.system())
           .add_system(sim_systems::advance_simulation.system().label("simulation"))
           .add_system(sim_systems::update_particles_renders.system().after("simulation"))
           .add_system(sim_systems::update_bounding_box_renders.system().after("simulation"));
    }
}



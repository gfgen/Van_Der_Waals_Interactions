pub mod error;
mod particle;
mod physics;
mod render_systems;
mod sim_space;
mod sim_systems;
pub mod state_generator;
mod ui_systems;

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
    steps_per_frame: usize,
    ext_a: Vec3, // external acceleration applied to all particles
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
            steps_per_frame: 20,
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

    pub fn set_steps_per_frame(mut self, spf: usize) -> Self {
        self.steps_per_frame = spf;
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
    // Check for consistency and create a VDWSimulation
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
        if self.steps_per_frame <= 0 {
            errors.push(ErrorKind::StepsPerFrame);
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
                self.steps_per_frame,
                self.ext_a,
                self.particles.clone(),
            ))
        }
    }
}

/////////////////////////////
// State component wrappers

pub struct BoundRate(pub f32); // Rate at which the boundary is growing/shrinking
pub struct TargetTemp(pub f32);
pub struct InjectRate(pub f32);
pub struct Dt(pub f32);
pub struct StepsPerFrame(pub usize);
pub struct ExtAccel(pub Vec3);
#[derive(Clone, Copy, Default)]
pub struct Energy {
    pub kinetic: f32,
    pub potential: f32,
}
pub struct EnergyHistory(pub VecDeque<Energy>);

// a struct to keep pressure stablized at a certain value
// done by shrinking or expanding the boundary
pub struct PressurePinned {
    pub is_pinned: bool,
    pub at_value: f32
}
// This is a ring buffer
pub struct Pressure {
    data: VecDeque<f32>,
    sum_cache: f32,
    dt: f32, // time per data point
    history: VecDeque<f32>,
}
impl Pressure {
    // Create ring buffer with capacity, all entries initialized to zero
    pub fn new(capacity: usize, dt: f32) -> Self {
        Self {
            data: VecDeque::from(vec![0.0; capacity]),
            sum_cache: 0.0,
            dt,
            history: VecDeque::from(vec![0.0; 1000]),
        }
    }

    pub fn push_sample(&mut self, value: f32) {
        self.sum_cache -= self.data.pop_front().unwrap_or(0.0);
        self.sum_cache += value;
        self.data.push_back(value);

        self.history.pop_front();
        self.history.push_back(self.get_impulse())
    }

    // Calulate the pressure based on sampled values
    pub fn get_impulse(&self) -> f32 {
        self.sum_cache / self.data.len() as f32 / self.dt
    }
}

//////////////////////////////////////////////////////////////
// State contains all simulation parameters and particle data
// Is a bevy Plugin
// Can only be created by compiling a StatePrototype
//
pub struct VDWSimulation {
    bound: Boundary, // location of the 6 walls of the box
    grid: Grid,
    dt: f32,
    steps_per_frame: usize,
    ext_accel: Vec3, // external acceleration applied to all particles

    pub particles: Vec<Particle>,
}

impl VDWSimulation {
    const PRESSURE_SAMPLING_PERIOD: f32 = 5.0; // Average impulses over this period of time

    // Make a new State
    // This function is only used by StatePrototype's compile method
    fn new(
        bound: Boundary,
        grid: Grid,
        dt: f32,
        steps_per_frame: usize,
        ext_accel: Vec3,
        particles: Vec<Particle>,
    ) -> Self {
        Self {
            bound,
            grid,
            dt,
            steps_per_frame,
            ext_accel,

            particles,
        }
    }
}

impl Plugin for VDWSimulation {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(self.bound)
            .insert_resource(self.grid)
            .insert_resource(Dt(self.dt))
            .insert_resource(StepsPerFrame(self.steps_per_frame))
            .insert_resource(ExtAccel(self.ext_accel))
            .insert_resource(self.particles.clone())
            .insert_resource(Pressure::new(
                (Self::PRESSURE_SAMPLING_PERIOD / self.dt / self.steps_per_frame as f32)
                    as usize,
                self.dt * self.steps_per_frame as f32,
            ))
            .insert_resource(PressurePinned {
                is_pinned: false,
                at_value: 0.1
            })
            .insert_resource(TargetTemp(0.0))
            .insert_resource(InjectRate(0.0))
            .insert_resource(BoundRate(0.0))
            .insert_resource(Energy::default()) // initialize for ui system
            .insert_resource(EnergyHistory(VecDeque::from(vec![Energy::default(); 1000])))
            .add_startup_system(render_systems::setup_bounding_box.system())
            .add_startup_system(render_systems::setup_particles.system())
            .add_startup_system(render_systems::setup_camera.system())
            .add_system(sim_systems::advance_simulation.system().label("simulation"))
            .add_system(
                render_systems::update_particles_renders
                    .system()
                    .after("simulation"),
            )
            .add_system(
                render_systems::update_bounding_box_renders
                    .system()
                    .after("simulation"),
            )
            .add_system(ui_systems::param_sliders.system())
            .add_system(ui_systems::simulation_info.system());
    }
}

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
use rayon::prelude::*;
use sim_space::*;

use crate::ring_buffer::RingBuffer;

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
                self.particles.clone(),
                self.bound,
                Grid::new(self.grid_unit_size, self.grid_reach),
                self.dt,
                self.steps_per_frame,
                self.ext_a,
            ))
        }
    }
}

/////////////////////////////
// State component wrappers
#[derive(Clone, Copy, Default)]
pub struct Energy {
    pub kinetic: f32,
    pub potential: f32,
}

// a struct to keep pressure stablized at a certain value
// done by shrinking or expanding the boundary
#[derive(Clone)]
pub struct PressurePinned {
    pub previous_state: bool, // To reset bound_rate when toggle
    pub is_pinned: bool,
    pub at_value: f32,
}
// Process instantaneous impulse data to return pressure
#[derive(Clone)]
pub struct Pressure {
    data: RingBuffer<f32>,
    sum_cache: f32,
    dt: f32, // time per data point
}
impl Pressure {
    // Create ring buffer with capacity, all entries initialized to zero
    pub fn new(capacity: usize, dt: f32) -> Self {
        Self {
            data: RingBuffer::with_capacity(capacity),
            sum_cache: 0.0,
            dt,
        }
    }

    pub fn push_sample(&mut self, value: f32) {
        self.sum_cache -= self.data.push(value).unwrap_or(0.0);
        self.sum_cache += value;
    }

    // Calulate the average impulse based on sampled values
    pub fn get_pressure(&self) -> f32 {
        self.sum_cache / self.data.len() as f32 / self.dt
    }
}

// Store the previous entries of energy and pressure
#[derive(Clone)]
pub struct History {
    energy: RingBuffer<Energy>,
    pressure: RingBuffer<f32>,
}
impl History {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            energy: RingBuffer::with_capacity(capacity),
            pressure: RingBuffer::with_capacity(capacity),
        }
    }
}

//////////////////////////////////////////////////////////////
// State contains all simulation parameters and particle data
// Can only be created by compiling a StatePrototype
//
#[derive(Clone)]
pub struct SimulationState {
    // Simulated entities
    pub particles: Vec<Particle>,
    pub bound: Boundary, // location of the 6 walls of the box
    grid: Grid,

    // Simulation dynamic quantities
    pub bound_rate: f32,
    pub target_temp: f32,
    pub inject_rate: f32,
    heat_injection_ammount: f32, // private cache
    pub pressure_pinned: PressurePinned,

    // Simulation constants
    pub dt: f32,
    pub steps_per_frame: usize,
    pub ext_accel: Vec3, // external acceleration applied to all particles

    // Simulation measurements
    pub steps: usize, // number of times step is called
    pub energy: Energy,
    pub pressure: Pressure,
    pub impulse_accumultor: f32, // cache for impulse, used to calculate pressure
    pub history: History,        // history of energy and pressure
}

impl SimulationState {
    // Execute one time step
    // For now only uses leapfrog
    // return impulse recorded by boundary
    pub fn step(&mut self) {
        self.steps += 1;
        let dt = self.dt;

        // step position
        self.particles
            .par_iter_mut()
            .for_each(|particle| particle.step_pos(dt, 0.5));

        // calculate accelerations and step velocity
        let (accelerations, neighbors, pot_energy, impulse) =
            self.calculate_particle_acceleration();
        (&mut self.particles, accelerations)
            .into_par_iter()
            .for_each(|(particle, acc)| particle.step_vel(acc, dt, 1.0));

        // inject/drain heat into/from system
        let heat_injection_ammount = self.heat_injection_ammount;
        self.particles.par_iter_mut().for_each(|particle| {
            particle.heat(dt, heat_injection_ammount);
        });

        // save number of neighbors
        // used for rendering particles with different colors
        (&mut self.particles, neighbors)
            .into_par_iter()
            .for_each(|(particle, nei)| particle.neighbors = nei);

        // step position again
        self.particles
            .par_iter_mut()
            .for_each(|particle| particle.step_pos(dt, 0.5));

        // adjust boundary size
        self.bound.expand(self.bound_rate, self.dt);

        // record potential energy
        self.energy.potential = pot_energy;

        // accumulate impulse
        self.impulse_accumultor += impulse;
    }

    // Return a list of acceleration correspond to each particle
    // Return the potential energy and pressure of the system
    // internal helper function
    fn calculate_particle_acceleration(&mut self) -> (Vec<Vec3>, Vec<usize>, f32, f32) {
        // Collect particle positions
        let particle_pos = self
            .particles
            .iter()
            .map(|particle| particle.get_pos())
            .collect();

        // Calculate forces
        let bound_force = self.bound.calculate_force(&particle_pos);
        let (grid_force, potential_energies, neighbors) = self.grid.calculate_force(&self.particles);

        // Sum up accelerations
        let accelerations = (&self.particles, &bound_force, &grid_force)
            .into_par_iter()
            // @param bnd_f: force on particle by the bounding box
            // @param grd_f: force on particle by other particles as calculated through the grid
            .map(|(particle, &bnd_f, &grd_f)| {
                (bnd_f + grd_f) / particle.get_mass() + self.ext_accel
            })
            .collect();

        // calculate impulse and potential energy
        let potential_energy: f32 = potential_energies.iter().sum();
        let impulse: f32 = bound_force
            .iter()
            .map(|bnd_f| bnd_f.length() * self.dt)
            .sum();

        (accelerations, neighbors, potential_energy, impulse)
    }

    // Kinetic energy is cached in a variable, this function updates that cache
    pub fn recalculate_kinetic_energy(&mut self) {
        self.energy.kinetic = self
            .particles
            .iter_mut()
            .map(|particle| 0.5 * particle.get_mass() * particle.get_vel().length_squared())
            .sum();

        // update heat injection per time step
        let current_temp = self.energy.kinetic / self.particles.len() as f32;
        self.heat_injection_ammount = (self.target_temp - current_temp) * self.inject_rate;
    }

    // Commit the impulse value accumulated through many timesteps
    // Reset the value
    pub fn commit_pressure(&mut self) {
        let pressure_value = self.impulse_accumultor / self.bound.get_surface_area();
        self.pressure.push_sample(pressure_value);
        self.impulse_accumultor = 0.0;
    }

    // Save current energy and pressure to history
    pub fn record_history(&mut self) {
        self.history.energy.push(self.energy);
        self.history.pressure.push(self.pressure.get_pressure());
    }
}

// Plugin
pub struct VDWSimulation {
    resources: SimulationState,
}

impl VDWSimulation {
    const PRESSURE_SAMPLING_PERIOD: f32 = 5.0; // Average impulses over this period of time

    // Make a new State
    // This function is only used by StatePrototype's compile method
    fn new(
        particles: Vec<Particle>,
        bound: Boundary,
        grid: Grid,
        dt: f32,
        steps_per_frame: usize,
        ext_accel: Vec3,
    ) -> Self {
        Self {
            resources: SimulationState {
                particles,
                bound,
                grid,

                bound_rate: 0.0,
                target_temp: 0.0,
                inject_rate: 0.0,
                heat_injection_ammount: 0.0,
                pressure_pinned: PressurePinned {
                    previous_state: false,
                    is_pinned: false,
                    at_value: 0.5,
                },

                dt,
                steps_per_frame,
                ext_accel,

                steps: 0,
                energy: Energy::default(),
                pressure: Pressure::new(
                    (Self::PRESSURE_SAMPLING_PERIOD / dt / steps_per_frame as f32) as usize,
                    dt * steps_per_frame as f32,
                ),
                impulse_accumultor: 0.0,
                history: History::with_capacity(1000),
            },
        }
    }
}
impl Plugin for VDWSimulation {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(self.resources.clone())
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

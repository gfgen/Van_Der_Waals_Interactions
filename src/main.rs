#![allow(dead_code)] // TODO: get rid of this when finish developing
extern crate bevy;
extern crate clap;
extern crate itertools;
extern crate ndarray;
extern crate rand;
extern crate rand_distr;
extern crate rayon;
extern crate ringbuffer as rb;

mod bevy_flycam;
mod ring_buffer;
mod state;
mod trans_rot_complexes;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_flycam::NoCameraPlayerPlugin;
use state::state_generator::Initialize;

fn main() -> Result<(), state::error::InvalidParamError> {
    let particle1 = state::particle::Particle::new()
        .set_pos_translation(2.0, 2.0, 2.0);
    let particle2 = state::particle::Particle::new()
        .set_pos_translation(2.06, 2.08, 2.1);
    let particles = vec![particle1, particle2];

    let vdw_simulation = state::SimulationPrototype::new()
        .set_dt(0.001)
        .set_steps_per_frame(20)
        .set_bound_x(10.0)
        .set_bound_y(10.0)
        .set_bound_z(10.0)
        .initialize_spherical_cloud(400, 2.0, 1.0)

        // .set_particles(particles)
        .compile()?;

    App::build()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins)
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(vdw_simulation)
        .add_plugin(EguiPlugin)
        // Set antialiasing to use 4 samples
        // .insert_resource(Msaa { samples: 2 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Van Der Waals Interaction".to_string(),
            width: 800.,
            height: 800.,
            ..Default::default()
        })
        .run();

    Ok(())
}

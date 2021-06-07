#![allow(dead_code)] // TODO: get rid of this when finish developing
extern crate bevy;
extern crate clap;
extern crate itertools;
extern crate ndarray;
extern crate rand;
extern crate rand_distr;
extern crate rayon;
extern crate ringbuffer as rb;

mod state;
mod bevy_flycam;

use bevy::prelude::*;
use bevy_flycam::NoCameraPlayerPlugin;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use state::state_generator::Initialize;


fn ui_example(egui_context: ResMut<EguiContext>) {
    egui::Window::new("Hello").show(egui_context.ctx(), |ui| {
        ui.label("world");
    });
}

fn main() -> Result<(), state::error::InvalidParamError> {
    let vdw_simulation = state::SimulationPrototype::new()
        .set_bound_x(10.0)
        .set_bound_y(10.0)
        .set_bound_z(10.0)
        .initialize_spherical_cloud(1000, 1.0, 0.2)
        .compile()?;

    App::build()
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
        .add_system(ui_example.system())

        .run();

    Ok(())
}

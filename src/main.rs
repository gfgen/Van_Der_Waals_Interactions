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
use state::BoundRate;

fn bound_slider(
    egui_context: ResMut<EguiContext>,
    mut bound_rate: ResMut<BoundRate>
) {
    egui::Window::new("Bound X Slider").show(egui_context.ctx(), |ui| {
        ui.label("world");
        ui.add(egui::Slider::new(&mut bound_rate.0, -1.0..=1.0).text("Tweak Boundary"));
    });
}

fn main() -> Result<(), state::error::InvalidParamError> {
    let vdw_simulation = state::SimulationPrototype::new()
        .set_bound_x(10.0)
        .set_bound_y(10.0)
        .set_bound_z(10.0)
        .initialize_spherical_cloud(1000, 1.0, 0.3)
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
        .add_system(bound_slider.system())

        .run();

    Ok(())
}

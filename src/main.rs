#![allow(dead_code)] // TODO: get rid of this when finish developing
extern crate ringbuffer as rb;
extern crate rayon;
extern crate clap;
extern crate nalgebra as na;
extern crate ndarray;
extern crate itertools;
extern crate bevy;

mod state; 
mod physics;

use bevy::prelude::*;

fn main() {
    App::build()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Van Der Waals Interaction".to_string(),
            width: 1200.,
            height: 800.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .run();
}
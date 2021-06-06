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
mod interactivity;

use bevy::prelude::*;
use state::particle::Particle;
use std::error::Error;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere { 
                radius: 2.0,
                subdivisions: 1
            })),
            material: materials.add(Color::rgb(1., 0.9, 0.9).into()),
            transform: Transform::from_translation(Vec3::new(4., 0., 4.)),
            ..Default::default()
        });
    // Camera
    commands.spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-7.0, 20.0, 4.0))),
            ..Default::default()
        });

        // Light
    commands.spawn()
        .insert_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn main() -> Result<(), state::error::InvalidParamError> {
/*     App::build()
        .add_startup_system(setup.system())
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Van Der Waals Interaction".to_string(),
            width: 1200.,
            height: 800.,
            ..Default::default()
        })
        .add_plugin(interactivity::camera_panning::CameraPanning)
        .add_plugins(DefaultPlugins)
        .run(); */

    let particles = vec![Particle::new().set_pos(1.0, 1.0, 1.0).set_vel(1.0, 1.0, 1.0)];
    let mut state = state::StatePrototype::new().set_particles(particles).compile()?;
    for _i in 1..10000 {
        state.step();
        state.anim_render();
    }

    Ok(())
}
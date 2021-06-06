#![allow(dead_code)] // TODO: get rid of this when finish developing
extern crate ringbuffer as rb;
extern crate rayon;
extern crate clap;
extern crate ndarray;
extern crate itertools;
extern crate bevy;
extern crate bevy_flycam;
extern crate rand;

mod state; 
mod physics;

use bevy::prelude::*;
use bevy_flycam::{NoCameraPlayerPlugin, FlyCam};
use state::particle::Particle;

// TODO: Clean up main.rs
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<state::State>
) {
    // Inserting spheres
    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere { 
                radius: 0.1,
                subdivisions: 0
    }));
    let white_mat = materials.add(Color::rgb(1., 0.9, 0.9).into());
    let n = state.particles.len();
    for _i in 0..n {
        commands.spawn()
            .insert_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: white_mat.clone(),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..Default::default()
            });
    }

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(5.0, 0.0, -5.0).looking_at(Vec3::X, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);

        // Light
    commands.spawn()
        .insert_bundle(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 10.0)),
            ..Default::default()
        });
}

fn update_state(
    mut state: ResMut<state::State>,
    mut query: Query<&mut Transform, With<Handle<Mesh>>>
) {
    for _i in 0..50 {
        state.step();
    }

    for (mut t, particle) in query.iter_mut().zip(state.particles.iter()) {
        let pos = particle.get_pos();
        *t = Transform::from_xyz(pos[0] as f32, pos[1] as f32, pos[2] as f32);
    }
}

// TODO: Move somewhere more appropriate
fn prune(particles: Vec<Particle>) -> Vec<Particle> {
    let mut ret: Vec<Particle> = vec![];
    for p1 in particles.iter() {
        let mut qual = true;
        for p2 in ret.iter() {
            let r = p1.get_pos() - p2.get_pos();
            let rnorm = r.length();
            if rnorm == 0.0 { continue; }
            qual = qual && rnorm >= 0.15
        } 
        if qual { ret.push(p1.clone()); }
    }
    ret
}

fn main() -> Result<(), state::error::InvalidParamError> {
    let temp = 1.5;
    let mut particles = vec![];
    for _i in 0..1000 {
        particles.push(Particle::new()
            .set_pos(rand::random::<f32>() * 2.5 + 5.0, rand::random::<f32>() * 2.5 + 5.0, rand::random::<f32>() * 2.5 + 5.0)
            .set_vel((rand::random::<f32>() - 0.5) * temp, (rand::random::<f32>() - 0.5) * temp, (rand::random::<f32>() - 0.5) * temp));
    }
    let particles = prune(particles);
    let state = state::StatePrototype::new()
        .set_bound_x(10.0)
        .set_bound_y(10.0)
        .set_bound_z(10.0)
        .set_particles(particles)
        .compile()?;

    App::build()
        .add_startup_system(setup.system())
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 2 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Van Der Waals Interaction".to_string(),
            width:  800.,
            height: 800.,
            ..Default::default()
        })
        .insert_resource(state)
        .add_system(update_state.system())
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugins(DefaultPlugins)
        .run();

    Ok(())
}
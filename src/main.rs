#![allow(dead_code)] // TODO: get rid of this when finish developing
extern crate bevy;
extern crate bevy_flycam;
extern crate clap;
extern crate itertools;
extern crate ndarray;
extern crate rand;
extern crate rand_distr;
extern crate rayon;
extern crate ringbuffer as rb;

mod physics;
mod state;

use bevy::prelude::*;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::render::mesh::Indices;

fn create_line_mesh(x: f32, y: f32, z: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<state::State>,
) {

    // Inserting spheres
    let white_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false,
        ..Default::default()
    });

    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.1,
        subdivisions: 0,
    }));

    let n = state.particles.len();
    for _i in 0..n {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: white_mat.clone(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        });
    }

    // Drawing bounding Box
    let bound = state.get_bound();
    let multipliers = [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];
    let white_mat_unlit = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..Default::default()
    });

    let line_x = meshes.add(create_line_mesh(bound.x, 0.0, 0.0));
    for &(mult1, mult2) in multipliers.iter() {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: line_x.clone(),
            material: white_mat_unlit.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, bound.y * mult1, bound.z * mult2)),
            ..Default::default()
        });
    }
    let line_y = meshes.add(create_line_mesh(0.0, bound.y, 0.0));
    for &(mult1, mult2) in multipliers.iter() {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: line_y.clone(),
            material: white_mat_unlit.clone(),
            transform: Transform::from_translation(Vec3::new(bound.x * mult1, 0.0, bound.z * mult2)),
            ..Default::default()
        });
    }
    let line_z = meshes.add(create_line_mesh(0.0, 0.0, bound.z));
    for &(mult1, mult2) in multipliers.iter() {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: line_z.clone(),
            material: white_mat_unlit.clone(),
            transform: Transform::from_translation(Vec3::new(bound.x * mult1, bound.y * mult2, 0.0)),
            ..Default::default()
        });
    }

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(5.0, 0.0, -5.0).looking_at(state.get_bound().center(), Vec3::ZERO),
            ..Default::default()
        })
        .insert(FlyCam);

    // Light
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(state.get_bound().lo_corner()),
        ..Default::default()
    });

    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(state.get_bound().hi_corner()),
        ..Default::default()
    });
}

fn update_state(
    mut state: ResMut<state::State>,
    mut query: Query<&mut Transform, With<Handle<Mesh>>>,
) {
    for _i in 0..50 {
        state.step();
    }

    for (mut t, particle) in query.iter_mut().zip(state.particles.iter()) {
        let pos = particle.get_pos();
        *t = Transform::from_xyz(pos[0] as f32, pos[1] as f32, pos[2] as f32);
    }
}

fn main() -> Result<(), state::error::InvalidParamError> {
    let state = state::StatePrototype::new()
        .set_bound_x(10.0)
        .set_bound_y(10.0)
        .set_bound_z(10.0);
    let particles = state::state_generator::generate_spherical_cloud(state.get_bound(), 1000, 1.0, 0.0);
    let state = state.set_particles(particles).compile()?;

    App::build()
        .add_startup_system(setup.system())
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 2 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: "Van Der Waals Interaction".to_string(),
            width: 800.,
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

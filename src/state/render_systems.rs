// bevy systems that updates the render of the simulation
use super::*;
use crate::bevy_flycam::{FlyCam, InputState};
use bevy::render::pipeline::PrimitiveTopology;
use itertools::iproduct;

// Marker Component:
pub struct IsParticle;
pub struct IsBoundEdge;

// Update the rendering of particles
pub fn update_particles_renders(
    state: Res<SimulationState>,
    particle_mats: Res<ParticleMats>,
    mut particle_renders: Query<(&mut Transform, &mut Handle<StandardMaterial>), With<IsParticle>>,
) {
    for ((mut trans, mut mat), particle) in particle_renders.iter_mut().zip(state.particles.iter())
    {
        let pos = particle.get_pos();
        *trans = Transform::from_xyz(pos[0] as f32, pos[1] as f32, pos[2] as f32);

        if particle.neighbors > 3 {
            *mat = particle_mats.blue.clone();
        } else {
            *mat = particle_mats.white.clone();
        }
    }
}

// Update the rendering of bounding box
pub fn update_bounding_box_renders(
    state: Res<SimulationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut bounding_box_renders: Query<(&mut Transform, &mut Handle<Mesh>), With<IsBoundEdge>>,
) {
    let bound = state.bound;
    let binary = [0.0, 1.0]; // generate the four corners of each axis
    let conditions = [0, 1, 2]; // stands for x, y, z axis
    let multipliers = iproduct!(conditions.iter(), binary.iter(), binary.iter());

    let line_x = meshes.add(create_line_mesh(bound.x, 0.0, 0.0));
    let line_y = meshes.add(create_line_mesh(0.0, bound.y, 0.0));
    let line_z = meshes.add(create_line_mesh(0.0, 0.0, bound.z));

    for ((&cond, &mult1, &mult2), (mut trans, mut mesh)) in
        multipliers.zip(bounding_box_renders.iter_mut())
    {
        // edges along the x axis
        if cond == 0 {
            *mesh = line_x.clone();
            trans.translation = Vec3::new(0.0, bound.y * mult1, bound.z * mult2);
        }
        // edges along the y axis
        else if cond == 1 {
            *mesh = line_y.clone();
            trans.translation = Vec3::new(bound.x * mult1, 0.0, bound.z * mult2);
        }
        // edges along the z axis
        else if cond == 2 {
            *mesh = line_z.clone();
            trans.translation = Vec3::new(bound.x * mult1, bound.y * mult2, 0.0);
        }
    }
}
//////////////////////////////////////////
pub fn setup_bounding_box(
    state: Res<SimulationState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bound = state.bound;

    // Draw bounding Box
    let multipliers = [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)];
    let white_mat_unlit = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..Default::default()
    });

    let line_x = meshes.add(create_line_mesh(bound.x, 0.0, 0.0));
    for &(mult1, mult2) in multipliers.iter() {
        commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: line_x.clone(),
                material: white_mat_unlit.clone(),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    bound.y * mult1,
                    bound.z * mult2,
                )),
                ..Default::default()
            })
            .insert(IsBoundEdge);
    }
    let line_y = meshes.add(create_line_mesh(0.0, bound.y, 0.0));
    for &(mult1, mult2) in multipliers.iter() {
        commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: line_y.clone(),
                material: white_mat_unlit.clone(),
                transform: Transform::from_translation(Vec3::new(
                    bound.x * mult1,
                    0.0,
                    bound.z * mult2,
                )),
                ..Default::default()
            })
            .insert(IsBoundEdge);
    }
    let line_z = meshes.add(create_line_mesh(0.0, 0.0, bound.z));
    for &(mult1, mult2) in multipliers.iter() {
        commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: line_z.clone(),
                material: white_mat_unlit.clone(),
                transform: Transform::from_translation(Vec3::new(
                    bound.x * mult1,
                    bound.y * mult2,
                    0.0,
                )),
                ..Default::default()
            })
            .insert(IsBoundEdge);
    }

    // Add Lights
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(bound.lo_corner()),
        ..Default::default()
    });

    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(bound.hi_corner()),
        ..Default::default()
    });
}

// Helper function for draw bounding box
fn create_line_mesh(x: f32, y: f32, z: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    );
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh
}

////////////////////////////////////////////
pub struct ParticleMats {
    white: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>,
}

pub fn setup_particles(
    state: Res<SimulationState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Insert particle renders
    let white_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: false,
        ..Default::default()
    });

    let blue_mat = materials.add(StandardMaterial {
        base_color: Color::CYAN,
        unlit: false,
        ..Default::default()
    });

    let sphere_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.1,
        subdivisions: 0,
    }));

    let n = state.particles.len();
    for _i in 0..n {
        commands
            .spawn()
            .insert_bundle(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: white_mat.clone(),
                transform: Transform::from_translation(Vec3::ZERO),
                ..Default::default()
            })
            .insert(IsParticle);
    }

    commands.insert_resource(ParticleMats {
        white: white_mat,
        blue: blue_mat,
    })
}

////////////////////////////////////////////////////////////
pub fn setup_camera(
    mut commands: Commands,
    state: Res<SimulationState>,
    mut input_state: ResMut<InputState>,
) {
    let bound = state.bound;

    // Initialize Camera
    let camera_position = Vec3::new(5.0, 3.0, -5.0);
    let camera_trans = Transform::from_translation(camera_position)
        .looking_at(bound.center() - camera_position, Vec3::Y);
    let (axis, angle) = camera_trans.rotation.to_axis_angle();
    input_state.reset_axis_angle(axis, angle);
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: camera_trans,
            ..Default::default()
        })
        .insert(FlyCam);
}

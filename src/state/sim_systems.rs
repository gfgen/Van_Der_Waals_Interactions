use bevy::prelude::*;
use rayon::iter::IntoParallelIterator;
use super::*;
use super::particle::*;
use super::sim_space::*;
use rayon::prelude::*;
use crate::bevy_flycam::{FlyCam, InputState};
use bevy::render::pipeline::PrimitiveTopology;
use itertools::iproduct;

// Marker Component:
pub struct IsParticle;
pub struct IsBoundEdge;

// System that advance one animation frame
// Multiple simulation steps are executed in one animation frame
// TODO: implement steps per frame
pub fn advance_simulation(
    mut particles: ResMut<Vec<Particle>>,
    grid: Res<Grid>,
    mut bound: ResMut<Boundary>,
    dt: Res<Dt>,
    ext_accel: Res<ExtAccel>,
    bound_rate: Res<BoundRate>,

    targ_temp: Res<TargetTemp>,
    inject_rate: Res<InjectRate>,

    mut potential_energy: ResMut<PotentialEnergy>,
    mut kinetic_energy: ResMut<KineticEnergy>
) {
    for _i in 0..19 {
        step(&mut particles, &grid, &bound, &dt, &ext_accel, &targ_temp, &inject_rate);

        if bound_rate.0 != 0.0 {
            bound.expand(bound_rate.0, dt.0)
        }
    }

    let (pot_energy, impulse) = step(&mut particles, &grid, &bound, &dt, &ext_accel, &targ_temp, &inject_rate);

    if bound_rate.0 != 0.0 {
        bound.expand(bound_rate.0, dt.0)
    }

    // Record energy
    potential_energy.0 = pot_energy;
    kinetic_energy.0 = particles
        .iter_mut()
        .map(|particle| 0.5 * particle.get_mass() * particle.get_vel().length_squared())
        .sum();
}

// Execute one time step
// For now only uses leapfrog
// Helper function
fn step(
    particles: &mut Vec<Particle>,
    grid: &Grid,
    bound: &Boundary,
    dt: &Dt,
    ext_accel: &ExtAccel,

    targ_temp: &TargetTemp,
    inject_rate: &InjectRate
) -> (f32, f32) {
    // step position
    particles
        .par_iter_mut()
        .for_each(|particle| particle.step_pos(dt.0, 0.5));

    // calculate accelerations and step velocity
    let (accelerations, neighbors, pot_enery, impulse) = calculate_particle_acceleration(particles, grid, bound, dt, ext_accel);
    (&mut (*particles), accelerations).into_par_iter()
        .for_each(|(particle, acc)| particle.step_vel(acc, dt.0, 1.0));

    // inject/drain heat into/from system
    particles
        .par_iter_mut()
        .for_each(|particle| {
            let amount = (targ_temp.0 - particle.get_vel().length()) * inject_rate.0;
            particle.heat(dt.0, amount);
        });
    // save number of neighbors
    (&mut (*particles), neighbors).into_par_iter()
        .for_each(|(particle, nei)| particle.neighbors = nei);

    // step position again
    particles
        .par_iter_mut()
        .for_each(|particle| particle.step_pos(dt.0, 0.5));

    (pot_enery, impulse)
}

// Return a list of acceleration correspond to each particle
// Return the potential energy and pressure of the system
// Helper function
fn calculate_particle_acceleration(
    particles: &Vec<Particle>,
    grid: &Grid,
    bound: &Boundary,
    dt: &Dt,
    ext_accel: &ExtAccel
) -> (Vec<Vec3>, Vec<usize>, f32, f32) {
    // Collect particle positions
    let particle_pos = particles
        .iter()
        .map(|particle| particle.get_pos())
        .collect();

    // Calculate forces
    let bound_force = bound.calculate_force(&particle_pos);
    let (grid_force, potential_energies, neighbors) = grid.calculate_force(&particle_pos);

    // Sum up accelerations
    let accelerations = (particles, &bound_force, &grid_force)
        .into_par_iter()
        // @param bnd_f: force on particle by the bounding box
        // @param grd_f: force on particle by other particles as calculated through the grid
        .map(|(particle, &bnd_f, &grd_f)| (bnd_f + grd_f) / particle.get_mass() + ext_accel.0)
        .collect();

    // calculate pressure and potential energy
    let potential_energy: f32 = potential_energies.iter().sum();
    let impulse: f32 = bound_force
        .iter()
        .map(|bnd_f| bnd_f.length() * dt.0)
        .sum();

    (accelerations, neighbors, potential_energy, impulse)
}

// Update the rendering of particles
pub fn update_particles_renders(
    particles: Res<Vec<Particle>>,
    particle_mats: Res<ParticleMats>,
    mut particle_renders: Query<(&mut Transform, &mut Handle<StandardMaterial>), With<IsParticle>>
) {
    for ((mut trans, mut mat), particle) in particle_renders.iter_mut().zip(particles.iter()) {
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
    mut meshes: ResMut<Assets<Mesh>>,
    bound: Res<Boundary>,
    mut bounding_box_renders: Query<(&mut Transform, &mut Handle<Mesh>), With<IsBoundEdge>>
) {
    if !bound.is_changed() { return; }

    let binary = [0.0, 1.0];    // generate the four corners of each axis
    let conditions = [0, 1, 2]; // stands for x, y, z axis
    let multipliers = iproduct!(conditions.iter(), binary.iter(), binary.iter());

    let line_x = meshes.add(create_line_mesh(bound.x, 0.0, 0.0));
    let line_y = meshes.add(create_line_mesh(0.0, bound.y, 0.0));
    let line_z = meshes.add(create_line_mesh(0.0, 0.0, bound.z));

    for ((&cond, &mult1, &mult2), (mut trans, mut mesh)) in multipliers.zip(bounding_box_renders.iter_mut()) {
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
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    bound: Res<Boundary>
) {
    // Draw bounding Box
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
        })
        .insert(IsBoundEdge);
    }
    let line_y = meshes.add(create_line_mesh(0.0, bound.y, 0.0));
    for &(mult1, mult2) in multipliers.iter() {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: line_y.clone(),
            material: white_mat_unlit.clone(),
            transform: Transform::from_translation(Vec3::new(bound.x * mult1, 0.0, bound.z * mult2)),
            ..Default::default()
        })
        .insert(IsBoundEdge);
    }
    let line_z = meshes.add(create_line_mesh(0.0, 0.0, bound.z));
    for &(mult1, mult2) in multipliers.iter() {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: line_z.clone(),
            material: white_mat_unlit.clone(),
            transform: Transform::from_translation(Vec3::new(bound.x * mult1, bound.y * mult2, 0.0)),
            ..Default::default()
        })
        .insert(IsBoundEdge);
    }
}

// Helper function for draw bounding box
fn create_line_mesh(x: f32, y: f32, z: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0], [x, y, z]]);
    mesh
}

////////////////////////////////////////////
pub struct ParticleMats {
    white: Handle<StandardMaterial>,
    blue: Handle<StandardMaterial>
}

pub fn setup_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    particles: Res<Vec<Particle>>
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

    let n = particles.len();
    for _i in 0..n {
        commands.spawn().insert_bundle(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: white_mat.clone(),
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        })
        .insert(IsParticle);
    }

    commands.insert_resource(ParticleMats {
        white: white_mat,
        blue: blue_mat
    })
}

////////////////////////////////////////////////////////////
pub fn setup_camera(
    mut commands: Commands,
    bound: Res<Boundary>,
    mut input_state: ResMut<InputState>
) {
    // Initialize Camera
    let camera_position = Vec3::new(5.0, 3.0, -5.0);
    let camera_trans = Transform::from_translation(camera_position).looking_at(bound.center() - camera_position, Vec3::Y);
    let (axis, angle) = camera_trans.rotation.to_axis_angle();
    input_state.reset_axis_angle(axis, angle);
    commands.spawn().insert_bundle(PerspectiveCameraBundle {
            transform: camera_trans,
            ..Default::default()
        })
        .insert(FlyCam);

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
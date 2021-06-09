use super::particle::*;
use super::sim_space::*;
use super::*;
use bevy::prelude::*;
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;

// System that advance one animation frame
// Multiple simulation steps are executed in one animation frame
// TODO: implement steps per frame
pub fn advance_simulation(
    mut particles: ResMut<Vec<Particle>>,
    mut bound: ResMut<Boundary>,
    grid: Res<Grid>,
    dt: Res<Dt>,
    steps_per_frame: Res<StepsPerFrame>,
    ext_accel: Res<ExtAccel>,
    bound_rate: Res<BoundRate>,

    targ_temp: Res<TargetTemp>,
    inject_rate: Res<InjectRate>,

    mut energy: ResMut<Energy>,
    mut pressure: ResMut<Pressure>,
) {
    // calculate heat injection
    let current_temp = energy.kinetic / particles.len() as f32;
    let heat_injection = (targ_temp.0 - current_temp) * inject_rate.0;

    let mut total_impulse = 0.0;

    // Step simulation
    for _i in 0..(steps_per_frame.0 - 1) {
        let (_, impulse) = step(
            &mut particles,
            &grid,
            &bound,
            &dt,
            &ext_accel,
            heat_injection,
        );
        total_impulse += impulse;

        if bound_rate.0 != 0.0 {
            bound.expand(bound_rate.0, dt.0)
        }
    }

    let (pot_energy, impulse) = step(
        &mut particles,
        &grid,
        &bound,
        &dt,
        &ext_accel,
        heat_injection,
    );
    total_impulse += impulse;

    if bound_rate.0 != 0.0 {
        bound.expand(bound_rate.0, dt.0)
    }

    // Record energy
    energy.kinetic = particles
        .iter_mut()
        .map(|particle| 0.5 * particle.get_mass() * particle.get_vel().length_squared())
        .sum();
    energy.potential = pot_energy;
    // record pressure
    pressure.push_sample(total_impulse);
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

    heat_injection: f32,
) -> (f32, f32) {
    // step position
    particles
        .par_iter_mut()
        .for_each(|particle| particle.step_pos(dt.0, 0.5));

    // calculate accelerations and step velocity
    let (accelerations, neighbors, pot_enery, impulse) =
        calculate_particle_acceleration(particles, grid, bound, dt, ext_accel);
    (&mut (*particles), accelerations)
        .into_par_iter()
        .for_each(|(particle, acc)| particle.step_vel(acc, dt.0, 1.0));

    // inject/drain heat into/from system
    particles.par_iter_mut().for_each(|particle| {
        particle.heat(dt.0, heat_injection);
    });
    // save number of neighbors
    (&mut (*particles), neighbors)
        .into_par_iter()
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
    ext_accel: &ExtAccel,
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
    let impulse: f32 = bound_force.iter().map(|bnd_f| bnd_f.length() * dt.0).sum();

    (accelerations, neighbors, potential_energy, impulse)
}

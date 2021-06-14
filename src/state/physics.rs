use crate::trans_rot_complexes::*;
use bevy::prelude::Vec3;
use itertools::iproduct;

// this roughly determines how close the particle can approach each other before getting repelled
const R0: f32 = 0.15;

// In this model a particle is made of smaller charged region
pub fn particle_interaction(
    pos_targ: TRC,
    pos_other: TRC,
    range: f32,
) -> (TRCInfintesimal, f32, usize) {
    let mut total_potential = 0.0;
    let mut total_force = Vec3::ZERO;
    let mut total_torque = Vec3::ZERO;

    let r = (pos_targ - pos_other).translation;
    let r_norm_sqr = r.length_squared();

    if r_norm_sqr < range.powi(2) && false {
        let interaction_intensity = 24.0;

        let r_unit = r / R0;
        let r_unit2 = r_unit.length_squared();
        let r_unit6 = r_unit2.powi(3);
        let r_unit8 = r_unit2 * r_unit6;
        let r_unit12 = r_unit6.powi(2);
        let r_unit14 = r_unit6 * r_unit8;

        total_force += interaction_intensity / r_unit14 * r_unit;

        // calculate potential
        let range_unit = range / R0;
        let range_unit6 = range_unit.powi(6);
        let range_unit12 = range_unit6.powi(2);

        // this is the potential energy between two non-interacting particles need to shift this point to zero
        let free_potential = interaction_intensity * 2.0 / range_unit12 / 12.0 * R0;
        let potential = interaction_intensity * 2.0 / r_unit12 / 12.0 * R0;
        total_potential = (potential - free_potential) / 2.0;
    }

    // location of charged regions relative to center of mass of particle
    let charge_position = [
        Vec3::new(0.0, 0.5 * R0, 0.0),
        Vec3::new(0.0, -0.5 * R0, 0.0),
    ];
    let charge_charge = [-1.0, 1.0];

    // Constructing interaction list
    let charge_targ = charge_position
        .iter()
        .map(|&charge_pos| pos_targ.process_relative_position(charge_pos))
        .zip(charge_charge.iter());
    let charge_other = charge_position
        .iter()
        .map(|&charge_pos| pos_other.process_relative_position(charge_pos))
        .zip(charge_charge.iter());
    let interaction_list = iproduct!(charge_targ, charge_other);

    // Iterate through all combination of  charge interactions
    for ((p_targ, c_targ), (p_other, c_other)) in interaction_list {
        let (force, potential) = charge_interaction(p_targ, p_other, range, c_targ * c_other);
        total_potential += potential;
        total_force += force;

        let r_from_center = p_targ - pos_targ.translation;
        total_torque += r_from_center.cross(force);
    }
    let force_torque = TRCInfintesimal::new(total_force, total_torque);

    // determine neighbor
    let r = pos_targ.translation - pos_other.translation;
    let neighbor_threshold = 4.0 * R0.powi(2);
    let neighbor = if r.length_squared() < neighbor_threshold {
        1
    } else {
        0
    };

    (force_torque, total_potential, neighbor)
}

// calculate force and potential between a pair of charged region
// @param sign: if sign is positive, the force is repulsive instead
fn charge_interaction(pos_targ: Vec3, pos_other: Vec3, range: f32, sign: f32) -> (Vec3, f32) {
    let r = pos_targ - pos_other; // difference in position, point away from other
    let r_norm_sqr = r.length_squared();

    if r_norm_sqr > range.powi(2) {
        return (Vec3::ZERO, 0.0);
    }

    let interaction_scale = R0 / 1.0;
    let interaction_intensity = 40.0;
    let repulsion_intensity = 0.1;
    // Calculate force
    let r_unit = r / interaction_scale;
    let r_unit2 = r_unit.length_squared();
    let r_unit4 = r_unit2.powi(2);
    let r_unit6 = r_unit2.powi(3);
    let r_unit8 = r_unit2 * r_unit6;
    let r_unit12 = r_unit6.powi(2);
    let r_unit14 = r_unit6 * r_unit8;

    let mut force = sign * interaction_intensity / r_unit6 * r_unit;
    force += interaction_intensity * repulsion_intensity / r_unit14 * r_unit;

    // calculate potential
    let range_unit = range / interaction_scale;
    let range_unit4 = range_unit.powi(4);
    let range_unit6 = range_unit.powi(6);
    let range_unit12 = range_unit6.powi(2);

    // this is the potential energy between two non-interacting particles need to shift this point to zero
    let mut free_potential = sign * interaction_intensity / range_unit4 * interaction_scale / 4.0;
    free_potential +=
        interaction_intensity * repulsion_intensity / range_unit12 * interaction_scale / 12.0;

    let mut potential = sign * interaction_intensity / r_unit4 * interaction_scale / 4.0;
    potential += interaction_intensity * repulsion_intensity / r_unit12 * interaction_scale / 12.0;

    let potential_adjusted = (potential - free_potential) / 2.0;

    (force, potential_adjusted)
}

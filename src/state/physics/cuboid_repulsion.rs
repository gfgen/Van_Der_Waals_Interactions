// In this model, the repulsion force is square shaped
use crate::trans_rot_complexes::*;
use bevy::prelude::Vec3;

// this roughly determines the spatial scale of interactions between particles
const R0: f32 = 0.15;

pub fn particle_interaction(
    pos_targ: TRC,
    pos_other: TRC,
    range: f32,
) -> (TRCInfintesimal, f32, usize) {
    let mut total_potential = 0.0;
    let mut total_force = Vec3::ZERO;
    let mut total_torque = Vec3::ZERO;

    // points away from other
    let r = -pos_other + pos_targ;

    let r_trans = r.translation;
    let r_norm_sqr = r_trans.length_squared();

    // a point on the unit circle
    //      represents relative orientation of the two particles

    if r_norm_sqr < range.powi(2) {
        let interaction_intensity = 48.0;

        let r_unit = r_trans / R0;
        let r_unit2 = r_unit.length_squared();
        let r_unit6 = r_unit2.powi(3);
        let r_unit8 = r_unit2 * r_unit6;
        let r_unit12 = r_unit6.powi(2);
        let r_unit14 = r_unit6 * r_unit8;

        // attraction
        total_force -= interaction_intensity / r_unit8 * r_unit;

        // repulsion
        let repulsion_intensity = 0.3;
        let cuboid_power = 5;
        let cuboid_intensity = 1.0 / (3.0_f32).sqrt().powi(cuboid_power);

        // spherical repulsion
        total_force += interaction_intensity * repulsion_intensity / r_unit14 * r_unit;

        // cuboid repulsion
        let r_trans_len = r_trans.length();
        let r_orientation = r.rotation * (-r_trans / r_trans_len);
        let r_orientation_abs = r_orientation.abs();

        let cuboid_factor = r_orientation_abs.max_element(); 
            // cuboid factor is a value that ranges from 3^(-1/2) to 1
            // is the inverse of the length of a point on a unit cube

        // calculating gradient
        let mut max_index = 0;
        let mut sign = 1.0;
        for i in 0..3 {
            if r_orientation_abs[i] == cuboid_factor {
                max_index = i;
                sign = r_orientation[i] / r_orientation_abs[i];
            }
        }
        let mut r_orientation_grad = Vec3::ZERO;
        for i in 0..3 {
            if i == max_index {
                r_orientation_grad[i] =
                    1.0 / r_trans_len - r_trans[i].powi(2) / r_trans_len.powi(3);
            } else {
                r_orientation_grad[i] = -r_trans[i] * r_trans[max_index] / r_trans_len.powi(3);
            }
        }
        r_orientation_grad *= sign;

        total_force += interaction_intensity * repulsion_intensity * cuboid_intensity
            / cuboid_factor.powi(cuboid_power)
            / r_unit14
            * r_unit;
        total_force -=
            interaction_intensity * repulsion_intensity * cuboid_intensity / r_unit12 / 12.0 * R0
                / cuboid_factor.powi(cuboid_power + 1)
                * cuboid_power as f32
                * r_orientation_grad;


        /////////////////////
        // calculate potential
        let range_unit = range / R0;
        let range_unit6 = range_unit.powi(6);
        let range_unit12 = range_unit6.powi(2);

        // this is the potential energy between two non-interacting particles need to shift this point to zero
        let mut free_potential = -interaction_intensity / range_unit6 / 6.0 * R0;
        free_potential += interaction_intensity * repulsion_intensity * cuboid_intensity
            / cuboid_factor.powi(cuboid_power)
            / range_unit12
            / 12.0
            * R0;
        free_potential += interaction_intensity * repulsion_intensity / range_unit12 / 12.0 * R0;

        let mut potential = -interaction_intensity / r_unit6 / 6.0 * R0;
        potential += interaction_intensity * repulsion_intensity * cuboid_intensity
            / cuboid_factor.powi(cuboid_power)
            / r_unit12
            / 12.0
            * R0;
        potential += interaction_intensity * repulsion_intensity / r_unit12 / 12.0 * R0;

        total_potential = (potential - free_potential) / 2.0;
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

// Function to remap the cuboid factor 
// to be in the right range for the logistic curve
// and its derivative
fn remap_cuboid(x: f32) -> f32 {
    50.0 * (x - 0.9)
}

/* fn d_remap_cuboid(x: f32) -> f32 {

} */

// sigmoid and its derivative
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn d_sigmoid(x: f32) -> f32 {
    let exp_x = x.exp();
    exp_x / (1.0 + exp_x).powi(2)
}

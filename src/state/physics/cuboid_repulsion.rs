// In this model, the repulsion force is square shaped
use crate::trans_rot_complexes::*;
use bevy::prelude::*;

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
        let interaction_intensity = 24.0;
        let repulsion_intensity = 0.6;

        let r_scaled = r_trans / R0;
        let r_scaled2 = r_scaled.length_squared();
        let r_scaled6 = r_scaled2.powi(3);
        let r_scaled8 = r_scaled2 * r_scaled6;
        let r_scaled12 = r_scaled6.powi(2);
        let r_scaled14 = r_scaled6 * r_scaled8;

        ///////////////////////////////////////////////////////////////////
        // repulsion based on relative-position and orientation of other
        //

        let r_orientation = pos_other.rotation.inverse() * r_trans;
        let r_orientation_len = r_orientation.length();
        let r_orientation_unit = r_orientation / r_orientation_len;
        let r_orientation_abs = r_orientation_unit.abs();

        // cuboid factor of other is a value that ranges from 3^(-1/2) to 1
        // is the inverse of the length of a point on a unit cube
        let cuboid_factor_other = r_orientation_abs.max_element();

        // calculating gradient d/dr of cuboid_factor
        let mut max_index = 0;
        let mut sign = 1.0;
        for i in 0..3 {
            if r_orientation_abs[i] == cuboid_factor_other {
                max_index = i;
                sign = r_orientation_unit[i] / r_orientation_abs[i];
                break;
            }
        }

        let mut d_cuboid_factor_d_ori_other = Vec3::ZERO;
        for i in 0..3 {
            if i == max_index {
                d_cuboid_factor_d_ori_other[i] =
                    1.0 / r_orientation_len - r_orientation[i].powi(2) / r_orientation_len.powi(3);
            } else {
                d_cuboid_factor_d_ori_other[i] =
                    -r_orientation[i] * r_orientation[max_index] / r_orientation_len.powi(3);
            }
        }
        d_cuboid_factor_d_ori_other *= sign;
        let d_cuboid_factor_dr_other = pos_other.rotation * d_cuboid_factor_d_ori_other;

        // force calculation

        // attraction
        total_force -= interaction_intensity
            * cuboid_factor_function_attraction(cuboid_factor_other)
            / r_scaled8
            * r_scaled;
        total_force += interaction_intensity / r_scaled6 / 6.0
            * R0
            * d_cuboid_factor_function_attraction(cuboid_factor_other)
            * d_cuboid_factor_dr_other;

        // repulsion
        total_force += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_other)
            / r_scaled14
            * r_scaled;
        total_force -= interaction_intensity * repulsion_intensity / r_scaled12 / 12.0
            * R0
            * d_cuboid_factor_function_repulsion(cuboid_factor_other)
            * d_cuboid_factor_dr_other;

        // repulsion based on relative-position and orientation of target
        let r_orientation = pos_targ.rotation.inverse() * (-r_trans);
        let r_orientation_len = r_orientation.length();
        let r_orientation_unit = r_orientation / r_orientation_len;
        let r_orientation_abs = r_orientation_unit.abs();

        // cuboid factor of targ is a value that ranges from 3^(-1/2) to 1
        // is the inverse of the length of a point on a unit cube
        let cuboid_factor_targ = r_orientation_abs.max_element();

        // calculating gradient d/dr of cuboid_factor
        let mut max_index = 0;
        let mut sign = 1.0;
        for i in 0..3 {
            if r_orientation_abs[i] == cuboid_factor_targ {
                max_index = i;
                sign = r_orientation_unit[i] / r_orientation_abs[i];
                break;
            }
        }

        let mut d_cuboid_factor_d_ori_targ = Vec3::ZERO;
        for i in 0..3 {
            if i == max_index {
                d_cuboid_factor_d_ori_targ[i] =
                    1.0 / r_orientation_len - r_orientation[i].powi(2) / r_orientation_len.powi(3);
            } else {
                d_cuboid_factor_d_ori_targ[i] =
                    -r_orientation[i] * r_orientation[max_index] / r_orientation_len.powi(3);
            }
        }
        d_cuboid_factor_d_ori_targ *= sign;
        let d_cuboid_factor_dr_targ = -(pos_targ.rotation * d_cuboid_factor_d_ori_targ);

        // calculating gradient d/drotation of cuboid_factor
        // WARNING:
        //      somehow works, needs inspection
        let d_cuboid_factor_drot_targ = -r_trans.cross(d_cuboid_factor_dr_targ);

        // force calculation

        // attraction
        total_force -= interaction_intensity
            * cuboid_factor_function_attraction(cuboid_factor_targ)
            / r_scaled8
            * r_scaled;
        total_force += interaction_intensity / r_scaled6 / 6.0
            * R0
            * d_cuboid_factor_function_attraction(cuboid_factor_targ)
            * d_cuboid_factor_dr_targ;
        total_torque += interaction_intensity / r_scaled6 / 6.0
            * R0
            * d_cuboid_factor_function_attraction(cuboid_factor_targ)
            * d_cuboid_factor_drot_targ;

        // repulsion
        total_force += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_targ)
            / r_scaled14
            * r_scaled;
        total_force -= interaction_intensity * repulsion_intensity / r_scaled12 / 12.0
            * R0
            * d_cuboid_factor_function_repulsion(cuboid_factor_targ)
            * d_cuboid_factor_dr_targ;
        total_torque -= interaction_intensity * repulsion_intensity / r_scaled12 / 12.0
            * R0
            * d_cuboid_factor_function_repulsion(cuboid_factor_targ)
            * d_cuboid_factor_drot_targ;

        /////////////////////
        // calculate potential
        let range_scaled = range / R0;
        let range_scaled6 = range_scaled.powi(6);
        let range_scaled12 = range_scaled6.powi(2);

        // this is the potential energy between two non-interacting particles need to shift this point to zero
        let mut free_potential = -interaction_intensity
            * cuboid_factor_function_attraction(cuboid_factor_other)
            / range_scaled6
            / 6.0
            * R0;
        free_potential += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_other)
            / range_scaled12
            / 12.0
            * R0;

        free_potential -= interaction_intensity
            * cuboid_factor_function_attraction(cuboid_factor_targ)
            / range_scaled6
            / 6.0
            * R0;
        free_potential += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_targ)
            / range_scaled12
            / 12.0
            * R0;

        let mut potential = -interaction_intensity
            * cuboid_factor_function_attraction(cuboid_factor_other)
            / r_scaled6
            / 6.0
            * R0;
        potential += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_other)
            / r_scaled12
            / 12.0
            * R0;

        potential -= interaction_intensity * cuboid_factor_function_attraction(cuboid_factor_targ)
            / r_scaled6
            / 6.0
            * R0;
        potential += interaction_intensity
            * repulsion_intensity
            * cuboid_factor_function_repulsion(cuboid_factor_targ)
            / r_scaled12
            / 12.0
            * R0;

        total_potential = (potential - free_potential) / 2.0;
    }

    // total_force = Vec3::ZERO;// TODO: remove this line
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

// use these function to manipulate the shape of the interaction
// the attraction and repulsion force is scaled by the output of these functions
fn cuboid_factor_function_repulsion(cuboid_factor: f32) -> f32 {
    let cuboid_deepness = 1.0;
    sigmoid(remap_cuboid(cuboid_factor), cuboid_deepness)
}
fn d_cuboid_factor_function_repulsion(cuboid_factor: f32) -> f32 {
    let cuboid_deepness = 1.0;
    d_sigmoid(remap_cuboid(cuboid_factor), cuboid_deepness) * d_remap_cuboid(cuboid_factor)
}

fn cuboid_factor_function_attraction(cuboid_factor: f32) -> f32 {
    1.0
}
fn d_cuboid_factor_function_attraction(cuboid_factor: f32) -> f32 {
    0.0
}

// Function to remap the cuboid factor
// to be in the right range for the logistic curve
// and its derivative
fn remap_cuboid(x: f32) -> f32 {
    -50.0 * (x - 0.98)
}

fn d_remap_cuboid(_x: f32) -> f32 {
    -50.0
}

// sigmoid and its derivative
fn sigmoid(x: f32, depth: f32) -> f32 {
    depth / (1.0 + (-x).exp()) + 1.0 - depth
}

fn d_sigmoid(x: f32, depth: f32) -> f32 {
    let exp_x = x.exp();
    depth * exp_x / (1.0 + exp_x).powi(2)
}

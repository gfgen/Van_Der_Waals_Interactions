use bevy::prelude::Vec3;
use itertools::free;

// this roughly determines how close the particle can approach each other before getting repelled
const R0: f32 = 0.15;

// calculate force and potential on position 1
pub fn vdw_interaction(pos_targ: Vec3, pos_other: Vec3, range: f32) -> (Vec3, f32, usize) {
    let r = pos_targ - pos_other;
    let r_norm_sqr = r.length_squared();
    let interaction_intensity = 48.0;

    if r_norm_sqr > range.powi(2) {
        return (Vec3::new(0.0, 0.0, 0.0), 0.0, 0);
    }

    // Calculate force
    let r_unit = r / R0;
    let r_unit2 = r_unit.length_squared();
    let r_unit6 = r_unit2.powi(3);
    let r_unit8 = r_unit2 * r_unit6;
    let r_unit12 = r_unit6.powi(2);
    let r_unit14 = r_unit6 * r_unit8;

    let r_flat_unit = 1.4 / R0;
    let k = 200.0;
    let p = 6;
    let gaussian_mask = (-k * (r_unit.length() - r_flat_unit).powi(p)).exp();
    let d_gaussian_mask = (-k * p as f32 * (r_unit.length() - r_flat_unit).powi(p - 1)) * gaussian_mask;


    let mut force = interaction_intensity / r_unit14 * r_unit;
    force -= interaction_intensity / r_unit14 * gaussian_mask * r_unit;
    force += interaction_intensity
        * (1.0 / r_unit12 - 1.0 / r_flat_unit.powi(12))
        * d_gaussian_mask
        / r_unit.length()
        * r_unit;

    force -= interaction_intensity / r_unit8 * r_unit;
    force += interaction_intensity / r_unit8 * gaussian_mask * r_unit;
    force -= interaction_intensity
        * (1.0 / r_unit6 - 1.0 / r_flat_unit.powi(6))
        * d_gaussian_mask
        / r_unit.length()
        * r_unit;

    // calculate potential
    let range_unit = range / R0;
    let range_unit6 = range_unit.powi(6);
    let range_unit12 = range_unit6.powi(2);

    // this is the potential energy between two non-interacting particles need to shift this point to zero
    let mut free_potential = interaction_intensity / range_unit12 / 12.0 * R0;
    free_potential -= interaction_intensity 
        * (1.0 / range_unit12 - 1.0 / r_flat_unit.powi(12))
        * gaussian_mask
        / 12.0
        * R0;

    free_potential -= interaction_intensity / range_unit6 / 6.0 * R0;
    free_potential += interaction_intensity 
        * (1.0 / range_unit6 - 1.0 / r_flat_unit.powi(6))
        * gaussian_mask
        / 6.0
        * R0;

    let mut potential = interaction_intensity / r_unit12 / 12.0 * R0;
    potential -= interaction_intensity 
        * (1.0 / r_unit12 - 1.0 / r_flat_unit.powi(12))
        * gaussian_mask
        / 12.0
        * R0;

    potential -= interaction_intensity / r_unit6 / 6.0 * R0;
    potential += interaction_intensity 
        * (1.0 / r_unit6 - 1.0 / r_flat_unit.powi(6))
        * gaussian_mask
        / 6.0
        * R0;

    let potential_adjusted = (potential - free_potential) / 2.0;

    // determine neighbor
    let neighbor_threshold = 4.0 * R0.powi(2);
    let neighbor = if r_norm_sqr < neighbor_threshold {
        1
    } else {
        0
    };

    (force, potential_adjusted, neighbor)
}

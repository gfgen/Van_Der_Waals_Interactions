use bevy::prelude::Vec3;
use crate::state::particle::Particle;

// this roughly determines how close the particle can approach each other before getting repelled
const R0: f32 = 0.15;

// calculate force and potential on position 1
pub fn vdw_interaction(targ: &Particle, other: &Particle, range: f32) -> (Vec3, f32, usize) {
    let r = targ.get_pos() - other.get_pos();
    let r_norm_sqr = r.length_squared();

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

    let interaction_intensity = 12.0;
    let repulsion_intensity = 0.5;

    let mut force = interaction_intensity * repulsion_intensity / r_unit14 * r_unit;

    if !targ.inert && !other.inert {
        force -= interaction_intensity / r_unit8 * r_unit;
    }

    // calculate potential
    let range_unit = range / R0;
    let range_unit6 = range_unit.powi(6);
    let range_unit12 = range_unit6.powi(2);

    // this is the potential energy between two non-interacting particles need to shift this point to zero
    let mut potential_adjusted = 0.0;
    if !targ.inert && !other.inert {
        let mut free_potential = interaction_intensity * repulsion_intensity / 12.0 / range_unit12 * R0;
        free_potential -= interaction_intensity / 6.0 / range_unit6 * R0;

        let mut potential = interaction_intensity * repulsion_intensity / 12.0 / r_unit12 * R0;
        potential -= interaction_intensity / 6.0 / r_unit6 * R0;

        potential_adjusted = (potential - free_potential) / 2.0;
    }

    // determine neighbor
    let neighbor_threshold = 4.0 * R0.powi(2);
    let neighbor = if r_norm_sqr < neighbor_threshold {
        1
    } else {
        0
    };

    (force, potential_adjusted, neighbor)
}

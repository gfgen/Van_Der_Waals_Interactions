use crate::trans_rot_complexes::*;

// sub modules are various interaction model
mod charged_regions;
mod cuboid_repulsion;

// This function is an interface to conveniently switch between models
pub fn particle_interaction(
    pos_targ: TRC,
    pos_other: TRC,
    range: f32,
) -> (TRCInfintesimal, f32, usize) {
    cuboid_repulsion::particle_interaction(pos_targ, pos_other, range)
}

use na::Vector3;

// this roughly determines how close the particle can approach each other before getting repelled
const R0: f64 = 0.15; 

// calculate force and potential on position 1
pub fn vdw_interaction(pos_targ: &Vector3<f64>, pos_other: &Vector3<f64>, range: f64) -> (Vector3<f64>, f64) {
    let r = pos_targ - pos_other;
    let r_norm_sqr = r.norm_squared();


    if r_norm_sqr > range.powi(2) {
        return (Vector3::new(0.0, 0.0, 0.0), 0.0)
    }

    // Calculate force 
    let r_unit = r / R0;
    let r_unit2 = r_unit.norm_squared();
    let r_unit6 = r_unit2.powi(3);
    let r_unit8 = r_unit2 * r_unit6;
    let r_unit12 = r_unit6.powi(2);
    let r_unit14 = r_unit6 * r_unit8;

    let force = 24.0 * ((2.0 / r_unit14) - (1.0 / r_unit8)) * r_unit;

    // calculate potential
    let range_unit = range / R0;
    let range_unit6 = range_unit.powi(6);
    let range_unit12 = range_unit6.powi(2);

    // this is the potential energy between two non-interacting particles need to shift this point to zero
    let free_potential = 4.0 * R0 * (1.0 / range_unit12) - (1.0 / range_unit6); 
    let potential = 4.0 * R0 * (1.0 / r_unit12) - (1.0 / r_unit6);
    let potential_adjusted = (potential - free_potential) / 2.0;

    (force, potential_adjusted)
}
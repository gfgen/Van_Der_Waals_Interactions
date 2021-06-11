// bevy systems that advances the simulation
use super::*;
use bevy::prelude::*;

// System that advance one animation frame
// Multiple simulation steps are executed in one animation frame
// TODO: implement steps per frame
pub fn advance_simulation(
    mut state: ResMut<SimulationState>
) {
    let mut total_impulse = 0.0;

    // Step simulation
    for _i in 0..state.steps_per_frame {
        let impulse = state.step();
        total_impulse += impulse;

    }
    state.recalculate_kinetic_energy();

    // record pressure
    let area = state.bound.get_surface_area();
    state.pressure.push_sample(total_impulse / area);

    // Stablize pressure if applicable
    if state.pressure_pinned.is_pinned {
        let current_pressure = state.pressure.history.back().unwrap_or(&0.0);
        let delta = current_pressure - state.pressure_pinned.at_value;
        
        state.bound_rate = delta;
    }
}

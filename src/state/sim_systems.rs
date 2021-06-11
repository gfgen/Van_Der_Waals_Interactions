// bevy systems that advances the simulation
use super::*;
use bevy::prelude::*;

// System that advance one animation frame
// Multiple simulation steps are executed in one animation frame
pub fn advance_simulation(mut state: ResMut<SimulationState>) {
    // Step simulation
    for _i in 0..state.steps_per_frame {
        state.step();
    }
    state.recalculate_kinetic_energy();
    state.commit_pressure();
    state.record_history();

    // Stablize pressure if applicable
    if state.pressure_pinned.is_pinned {
        let current_pressure = state.history.pressure.peak().unwrap_or(&0.0);
        let delta = current_pressure - state.pressure_pinned.at_value;

        state.bound_rate = delta;
    }

    if state.steps % 300 == 0 {
        println!(
            "{}, {}",
            state.energy.kinetic + state.energy.potential,
            state.energy.kinetic
        );
    }
}

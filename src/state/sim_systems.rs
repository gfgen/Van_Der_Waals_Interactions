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
        let past_pressure = state.history.pressure.elem(700).unwrap_or(&0.0);

        let delta = state.pressure_pinned.at_value - current_pressure;
        let slope = current_pressure - past_pressure;

        let bound_size = state.bound.x;

        let bound_rate = slope - delta;
        let bound_rate = bound_rate.max(-bound_size * 0.01);
        let bound_rate = bound_rate.min(bound_size * 0.01);

        state.bound_rate = bound_rate;
    }
    // Reset bound_rate on toggle off
    else if state.pressure_pinned.previous_state {
        state.bound_rate = 0.0;
    }

    // dump energy status to terminal
    // TODO: separate into independent system
    if state.steps % 300 == 0 {
        println!(
            "{}, {}",
            state.energy.kinetic + state.energy.potential,
            state.energy.kinetic
        );
    }
}

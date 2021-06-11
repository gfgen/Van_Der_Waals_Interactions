// Contains bevy systems that draws the gui

use super::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use egui::plot::{Curve, Plot, Value};

pub fn param_sliders(
    egui_context: ResMut<EguiContext>,
    mut state: ResMut<SimulationState>
) {
    egui::Window::new("Sliders").show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            ui.checkbox(&mut state.pressure_pinned.is_pinned, "Pin pressure at: ");
            ui.add(egui::widgets::DragValue::new(&mut state.pressure_pinned.at_value));
        });
        ui.add(egui::Slider::new(&mut state.bound_rate, -0.2..=0.2).text("Boundary"));
        ui.add(egui::Slider::new(&mut state.target_temp, 0.0..=3.0).text("Target Temperature").clamp_to_range(true));
        ui.add(egui::Slider::new(&mut state.inject_rate, 0.0..=0.5).text("Injection Rate").clamp_to_range(true));
    });
}

pub fn simulation_info(
    egui_context: ResMut<EguiContext>,
    state: Res<SimulationState>
) {
    let total_energy = state.energy.kinetic + state.energy.potential;

    let pressure_val = state.pressure.get_pressure();
    let volume = state.bound.get_volume();
    let k = 2.0 / 3.0;

/*     let pressure_curve = Curve::from_values_iter(
        state.pressure
            .history
            .iter()
            .enumerate()
            .map(|(i, &p)| Value::new(i as f64, p))
    ); */

/*     let kin_energy_curve = Curve::from_values_iter(
        energy_history.0.iter()
            .enumerate()
            .map(|(i, e)| Value::new(i as f64, e.kinetic))
    );
    let tot_energy_curve = Curve::from_values_iter(
        energy_history.0.iter()
            .enumerate()
            .map(|(i, e)| Value::new(i as f64, e.kinetic + e.potential))
    ); */

    egui::Window::new("Pressure/Volume/Temperature").show(egui_context.ctx(), |ui| {
        ui.label(format!(
            "PV/nkT: {:.5}",
            pressure_val * volume / k / state.energy.kinetic
        ));
        ui.label(format!("P: {:.5}", pressure_val));
        ui.label(format!("V: {:.5}", volume));
        ui.label(format!("T: {:.5}", state.energy.kinetic / state.particles.len() as f32));
        // ui.add(Plot::new("Pressure").curve(pressure_curve));
    });

    egui::Window::new("Energy").show(egui_context.ctx(), |ui| {
        ui.label(format!("KE: {:.5}", state.energy.kinetic));
        ui.label(format!("PE: {:.5}", state.energy.potential));
        ui.label(format!("Total Energy: {:.5}", total_energy));
        // ui.add(Plot::new("Energy").curve(kin_energy_curve).curve(tot_energy_curve));
    });
}

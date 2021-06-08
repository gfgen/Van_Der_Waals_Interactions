// Contains the ui systems

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use super::*;

pub fn param_sliders(
    egui_context: ResMut<EguiContext>,
    mut bound_rate: ResMut<BoundRate>,
    mut targ_temp: ResMut<TargetTemp>,
    mut inject_rate: ResMut<InjectRate>,
) {
    egui::Window::new("Sliders").show(egui_context.ctx(), |ui| {
        ui.add(egui::Slider::new(&mut bound_rate.0, -0.2..=0.2).text("Boundary"));
        ui.add(egui::Slider::new(&mut targ_temp.0, 0.0..=3.0).text("Target Temperature"));
        ui.add(egui::Slider::new(&mut inject_rate.0, 0.0..=0.2).text("Injection Rate"));

    });
}

pub fn simulation_info(
    egui_context: ResMut<EguiContext>,
    bound: Res<Boundary>,
    potential_energy: Res<PotentialEnergy>,
    kinetic_energy: Res<KineticEnergy>,
) {
    let total_energy = potential_energy.0 + kinetic_energy.0;
    egui::Window::new("Info").show(egui_context.ctx(), |ui| {
        ui.label(format!("V: {:.6}", bound.get_volume()));
        ui.label(format!("PE: {:.6}", potential_energy.0));
        ui.label(format!("KE: {:.6}", kinetic_energy.0));
        ui.label(format!("Total Energy: {:.6}", total_energy));
    });

}

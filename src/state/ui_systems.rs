// Contains the ui systems

use super::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
// use egui::plot::{Curve, Plot, Value};

pub fn param_sliders(
    egui_context: ResMut<EguiContext>,
    mut bound_rate: ResMut<BoundRate>,
    mut targ_temp: ResMut<TargetTemp>,
    mut inject_rate: ResMut<InjectRate>,
) {
    egui::Window::new("Sliders").show(egui_context.ctx(), |ui| {
        ui.add(egui::Slider::new(&mut bound_rate.0, -0.2..=0.2).text("Boundary"));
        ui.add(egui::Slider::new(&mut targ_temp.0, 0.0..=3.0).text("Target Temperature"));
        ui.add(egui::Slider::new(&mut inject_rate.0, 0.0..=0.5).text("Injection Rate"));
    });
}

pub fn simulation_info(
    egui_context: ResMut<EguiContext>,
    particles: Res<Vec<Particle>>,
    bound: Res<Boundary>,
    energy: Res<Energy>,
    pressure: Res<Pressure>,
) {
    let total_energy = energy.kinetic + energy.potential;

    let pressure = pressure.get_value(bound.get_surface_area());
    let volume = bound.get_volume();
    let k = 2.0 / 3.0;

    egui::Window::new("Pressure/Volume/Temperature").show(egui_context.ctx(), |ui| {
        ui.label(format!(
            "PV/nkT: {:.5}",
            pressure * volume / k / energy.kinetic
        ));
        ui.label(format!("P: {:.5}", pressure));
        ui.label(format!("V: {:.5}", volume));
        ui.label(format!("T: {:.5}", energy.kinetic / particles.len() as f32));
    });

    egui::Window::new("Energy").show(egui_context.ctx(), |ui| {
        ui.label(format!("KE: {:.5}", energy.kinetic));
        ui.label(format!("PE: {:.5}", energy.potential));
        ui.label(format!("Total Energy: {:.5}", total_energy));
    });
}

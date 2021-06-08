// Contains the ui systems

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use super::*;
use egui::plot::{Curve, Plot, Value};

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
    pressure: Res<Pressure>
) {
    let total_energy = energy.kinetic + energy.potential;
    egui::Window::new("Info").show(egui_context.ctx(), |ui| {
        ui.label(format!("P: {}", pressure.get_value(bound.get_surface_area())));
        ui.label(format!("T: {}", energy.kinetic / particles.len() as f32));
        ui.label(format!("V: {:.6}", bound.get_volume()));
        ui.label(format!("KE: {:.6}", energy.kinetic));
        ui.label(format!("PE: {:.6}", energy.potential));
        ui.label(format!("Total Energy: {:.6}", total_energy));
    });
}


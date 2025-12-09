//! Parameter control panel

use egui::Ui;
use crate::state::{SimulationState, FormationType, AlgorithmType};

pub fn show(ui: &mut Ui, state: &mut SimulationState) {
    ui.heading("Parameters");
    ui.add_space(5.0);

    // Simulation Controls
    ui.collapsing("Simulation", |ui| {
        ui.horizontal(|ui| {
            if ui.button(if state.is_running { "⏸ Pause" } else { "▶ Play" }).clicked() {
                state.is_running = !state.is_running;
            }
            if ui.button("⟳ Reset").clicked() {
                state.reset();
            }
        });

        ui.add_space(5.0);
        ui.add(egui::Slider::new(&mut state.simulation_speed, 0.1..=5.0).text("Speed"));
    });

    ui.add_space(10.0);

    // Formation Controls
    ui.collapsing("Formation", |ui| {
        let mut formation_changed = false;

        egui::ComboBox::from_label("Type")
            .selected_text(format!("{:?}", state.formation))
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut state.formation, FormationType::Circle, "Circle").clicked() {
                    formation_changed = true;
                }
                if ui.selectable_value(&mut state.formation, FormationType::Grid, "Grid").clicked() {
                    formation_changed = true;
                }
                if ui.selectable_value(&mut state.formation, FormationType::Line, "Line").clicked() {
                    formation_changed = true;
                }
                if ui.selectable_value(&mut state.formation, FormationType::VFormation, "V-Formation").clicked() {
                    formation_changed = true;
                }
                if ui.selectable_value(&mut state.formation, FormationType::Random, "Random").clicked() {
                    formation_changed = true;
                }
            });

        ui.add_space(5.0);

        let mut drone_count_changed = false;
        let mut drone_count = state.formation_params.drone_count as i32;
        if ui.add(egui::Slider::new(&mut drone_count, 1..=50).text("Drones")).changed() {
            state.formation_params.drone_count = drone_count as usize;
            drone_count_changed = true;
        }

        match state.formation {
            FormationType::Circle => {
                let mut radius = state.formation_params.circle_radius as i32;
                if ui.add(egui::Slider::new(&mut radius, 20..=200).text("Radius")).changed() {
                    state.formation_params.circle_radius = radius as u32;
                }
            }
            FormationType::Grid => {
                let mut spacing = state.formation_params.grid_spacing as i32;
                if ui.add(egui::Slider::new(&mut spacing, 10..=100).text("Spacing")).changed() {
                    state.formation_params.grid_spacing = spacing as u32;
                }
            }
            FormationType::Line => {
                let mut spacing = state.formation_params.line_spacing as i32;
                if ui.add(egui::Slider::new(&mut spacing, 10..=100).text("Spacing")).changed() {
                    state.formation_params.line_spacing = spacing as u32;
                }
            }
            FormationType::VFormation => {
                let mut spacing = state.formation_params.v_spacing as i32;
                if ui.add(egui::Slider::new(&mut spacing, 10..=100).text("Spacing")).changed() {
                    state.formation_params.v_spacing = spacing as u32;
                }
            }
            FormationType::Random => {}
        }

        if formation_changed || drone_count_changed {
            state.spawn_drones(state.formation_params.drone_count);
        }
    });

    ui.add_space(10.0);

    // Algorithm Selection
    ui.collapsing("Algorithm", |ui| {
        egui::ComboBox::from_label("Active")
            .selected_text(format!("{:?}", state.active_algorithm))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.active_algorithm, AlgorithmType::PSO, "PSO");
                ui.selectable_value(&mut state.active_algorithm, AlgorithmType::ACO, "ACO");
                ui.selectable_value(&mut state.active_algorithm, AlgorithmType::GWO, "GWO");
            });
    });

    ui.add_space(10.0);

    // PSO Parameters
    if let Some(ref mut pso) = state.pso_state {
        ui.collapsing("PSO Parameters", |ui| {
            ui.add(egui::Slider::new(&mut pso.cognitive, 0.0..=4.0).text("Cognitive (c1)"));
            ui.add(egui::Slider::new(&mut pso.social, 0.0..=4.0).text("Social (c2)"));
            ui.add(egui::Slider::new(&mut pso.inertia, 0.0..=1.0).text("Inertia (w)"));

            ui.add_space(5.0);
            ui.label(format!("Particles: {}", pso.particles.len()));
            ui.label(format!("Iteration: {}", pso.iteration));
            ui.label(format!("Best Cost: {:.6}", pso.gbest_cost));
        });
    }

    ui.add_space(10.0);

    // ACO Parameters
    if let Some(ref mut aco) = state.aco_state {
        ui.collapsing("ACO Parameters", |ui| {
            ui.add(egui::Slider::new(&mut aco.evaporation_rate, 0.01..=0.5).text("Evaporation"));
            ui.add(egui::Slider::new(&mut aco.alpha, 0.1..=5.0).text("Alpha (α)"));
            ui.add(egui::Slider::new(&mut aco.beta, 0.1..=5.0).text("Beta (β)"));

            ui.add_space(5.0);
            ui.label(format!("Ants: {}", aco.ants.len()));
            ui.label(format!("Iteration: {}", aco.iteration));
            ui.label(format!("Best Path Length: {}", aco.best_path.len()));
        });
    }

    ui.add_space(10.0);

    // GWO Parameters
    if let Some(ref gwo) = state.gwo_state {
        ui.collapsing("GWO Parameters", |ui| {
            ui.label(format!("Wolves: {}", gwo.wolves.len()));
            ui.label(format!("Iteration: {}", gwo.iteration));
            ui.label(format!("Convergence (a): {:.3}", gwo.convergence_param));

            if let Some(ref alpha) = gwo.alpha {
                ui.label(format!("Alpha Fitness: {:.4}", alpha.fitness));
            }
        });
    }

    ui.add_space(10.0);

    // Viewport Options
    ui.collapsing("Viewport", |ui| {
        ui.checkbox(&mut state.viewport.show_grid, "Show Grid");
        ui.checkbox(&mut state.viewport.show_trails, "Show Trails");
        ui.checkbox(&mut state.viewport.show_velocities, "Show Velocities");

        ui.add_space(5.0);
        ui.add(egui::Slider::new(&mut state.viewport.zoom, 0.5..=10.0).text("Zoom"));

        if ui.button("Reset View").clicked() {
            state.viewport.center = egui::Pos2::ZERO;
            state.viewport.zoom = 2.0;
        }
    });
}

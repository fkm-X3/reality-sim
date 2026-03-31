//! UI panels for the simulator

use eframe::egui;

use super::app::SimulatorApp;

/// Simulation controls panel
pub fn controls_panel(ui: &mut egui::Ui, app: &mut SimulatorApp) {
    ui.heading("⚙ Controls");
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        if ui.button(if app.running { "⏸ Pause" } else { "▶ Play" }).clicked() {
            app.running = !app.running;
        }
        if ui.button("⏭ Step").clicked() {
            if let Some(ref mut sim) = app.sim {
                sim.tick();
            }
            app.record_history();
        }
        if ui.button("🏠 Menu").clicked() {
            app.return_to_menu();
        }
    });

    ui.add_space(5.0);
    ui.horizontal(|ui| {
        ui.label("Speed:");
        ui.add(egui::Slider::new(&mut app.speed, 1..=50).text("ticks/frame"));
    });

    ui.horizontal(|ui| {
        ui.label("Zoom:");
        ui.add(egui::Slider::new(&mut app.zoom, 0.25..=4.0).logarithmic(true));
    });

    ui.add_space(5.0);
    ui.label(format!("FPS: {:.1}", app.fps));
    
    if let Some(ref sim) = app.sim {
        ui.label(format!("Tick: {}", sim.world.tick));
        
        let year = sim.world.current_year;
        let year_str = if year < 0 {
            format!("{} BCE", -year)
        } else {
            format!("{} CE", year)
        };
        ui.label(format!("Year: {}", year_str));
        ui.label(format!("Era: {:?}", sim.world.era));
    }
}

/// Statistics panel
pub fn stats_panel(ui: &mut egui::Ui, app: &SimulatorApp) {
    ui.heading("📊 Statistics");
    ui.add_space(5.0);

    if let Some(ref sim) = app.sim {
        let stats = &sim.world.stats;
        
        egui::Grid::new("stats_grid")
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                ui.label("Population:");
                ui.label(format!("{}", stats.current_population));
                ui.end_row();

                ui.label("Total Spawned:");
                ui.label(format!("{}", stats.total_agents_spawned));
                ui.end_row();

                ui.label("Total Deaths:");
                ui.label(format!("{}", stats.total_deaths));
                ui.end_row();

                ui.label("Avg Fitness:");
                ui.label(format!("{:.2}", stats.average_fitness));
                ui.end_row();

                ui.label("Max Generation:");
                ui.label(format!("{}", stats.max_generation));
                ui.end_row();

                ui.label("Resources:");
                ui.label(format!("{}", sim.world.resources.len()));
                ui.end_row();
            });
    } else {
        ui.label("No simulation running");
    }
}

/// Factions panel
pub fn factions_panel(ui: &mut egui::Ui, app: &SimulatorApp) {
    ui.heading("🏴 Factions");
    ui.add_space(5.0);

    if let Some(ref sim) = app.sim {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (idx, faction) in sim.world.factions.factions.iter().enumerate() {
                    // Count agents in this faction
                    let member_count = sim.world.agents.iter()
                        .filter(|a| a.read().faction == idx)
                        .count();

                    ui.horizontal(|ui| {
                        // Color swatch
                        let color = egui::Color32::from_rgb(faction.color.r, faction.color.g, faction.color.b);
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, color);
                        
                        ui.label(format!("{}: {}", &faction.name, member_count));
                    });
                }
            });
    }
}

/// Agent inspector panel
pub fn agent_panel(ui: &mut egui::Ui, app: &mut SimulatorApp) {
    ui.heading("🔍 Agent Inspector");
    
    if let Some(agent_id) = app.selected_agent {
        if let Some(ref sim) = app.sim {
            // Find the agent
            let agent_data = sim.world.agents.iter()
                .find(|a| a.read().id == agent_id)
                .map(|a| a.read().clone());

            if let Some(agent) = agent_data {
                ui.add_space(5.0);
                
                // Faction color indicator
                let faction_color = sim.world.factions.get_color(agent.faction);
                let color = egui::Color32::from_rgb(faction_color.r, faction_color.g, faction_color.b);
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, color);
                    if let Some(faction) = sim.world.factions.get(agent.faction) {
                        ui.label(&faction.name);
                    }
                });

                ui.separator();
                
                egui::Grid::new("agent_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("ID:");
                        ui.label(format!("{:.8}...", agent.id));
                        ui.end_row();

                        ui.label("Position:");
                        ui.label(format!("({:.1}, {:.1})", agent.position.x, agent.position.y));
                        ui.end_row();

                        ui.label("Health:");
                        ui.add(egui::ProgressBar::new(agent.health).text(format!("{:.0}%", agent.health * 100.0)));
                        ui.end_row();

                        ui.label("Hunger:");
                        ui.add(egui::ProgressBar::new(agent.stimuli.hunger)
                            .fill(egui::Color32::from_rgb(230, 126, 34))
                            .text(format!("{:.0}%", agent.stimuli.hunger * 100.0)));
                        ui.end_row();

                        ui.label("Thirst:");
                        ui.add(egui::ProgressBar::new(agent.stimuli.thirst)
                            .fill(egui::Color32::from_rgb(46, 134, 193))
                            .text(format!("{:.0}%", agent.stimuli.thirst * 100.0)));
                        ui.end_row();

                        ui.label("Age:");
                        ui.label(format!("{} ticks", agent.age));
                        ui.end_row();

                        ui.label("Generation:");
                        ui.label(format!("{}", agent.generation));
                        ui.end_row();

                        ui.label("Fitness:");
                        ui.label(format!("{:.2}", agent.fitness()));
                        ui.end_row();
                    });

                ui.add_space(10.0);
                if ui.button("Deselect").clicked() {
                    app.selected_agent = None;
                }
            } else {
                ui.label("Agent no longer exists");
                app.selected_agent = None;
            }
        }
    }
}

/// Graphs panel - responsive layout with vertical stacking on narrow screens
pub fn graphs_panel(ui: &mut egui::Ui, app: &SimulatorApp) {
    let available_width = ui.available_width();
    let use_vertical = available_width < 500.0;
    
    egui::ScrollArea::horizontal().show(ui, |ui| {
        if use_vertical {
            // Vertical layout for narrow screens
            ui.vertical(|ui| {
                render_population_graph(ui, app, available_width - 20.0);
                ui.add_space(10.0);
                render_fitness_graph(ui, app, available_width - 20.0);
            });
        } else {
            // Horizontal layout for wide screens
            ui.horizontal(|ui| {
                let graph_width = (available_width / 2.0 - 25.0).max(150.0);
                render_population_graph(ui, app, graph_width);
                ui.add_space(10.0);
                render_fitness_graph(ui, app, graph_width);
            });
        }
    });
}

fn render_population_graph(ui: &mut egui::Ui, app: &SimulatorApp, width: f32) {
    ui.group(|ui| {
        ui.label("Population");
        let points: egui_plot::PlotPoints = app.population_history.iter()
            .enumerate()
            .map(|(i, &v)| [i as f64, v as f64])
            .collect();
        let line = egui_plot::Line::new(points).color(egui::Color32::from_rgb(46, 134, 193));
        
        egui_plot::Plot::new("population_plot")
            .height(80.0)
            .width(width)
            .show_axes(true)
            .allow_drag(false)
            .allow_zoom(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    });
}

fn render_fitness_graph(ui: &mut egui::Ui, app: &SimulatorApp, width: f32) {
    ui.group(|ui| {
        ui.label("Average Fitness");
        let points: egui_plot::PlotPoints = app.fitness_history.iter()
            .enumerate()
            .map(|(i, &v)| [i as f64, v as f64])
            .collect();
        let line = egui_plot::Line::new(points).color(egui::Color32::from_rgb(39, 174, 96));
        
        egui_plot::Plot::new("fitness_plot")
            .height(80.0)
            .width(width)
            .show_axes(true)
            .allow_drag(false)
            .allow_zoom(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    });
}

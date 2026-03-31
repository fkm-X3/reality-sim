//! Main GUI application state and update loop

use eframe::egui;
use std::time::Instant;

use reality_sim::config::WorldConfig;
use reality_sim::simulation::SimulationRunner;
use reality_sim::world::{FactionManager, ResourceType};

use super::panels;
use super::renderer::{AgentRenderData, FactionRenderData, RenderData, ResourceRenderData, WorldRenderer};

/// Application state - main menu or simulation
pub enum AppState {
    MainMenu,
    Simulation,
}

/// Configuration for new simulation (edited in main menu)
#[derive(Clone)]
pub struct SimConfig {
    pub starting_year: i32,
    pub agent_count: usize,
    pub faction_count: usize,
    pub climate_severity: f32,
    pub resource_density: f32,
    pub world_width: f32,
    pub world_height: f32,
    pub mutation_rate: f32,
    pub reproduction_threshold: f32,
    pub max_agent_age: u64,
    pub day_night_cycle: bool,
    pub natural_disasters: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            starting_year: -10000,
            agent_count: 500,
            faction_count: 6,
            climate_severity: 0.5,
            resource_density: 0.5,
            world_width: 1000.0,
            world_height: 800.0,
            mutation_rate: 0.1,
            reproduction_threshold: 0.7,
            max_agent_age: 10000,
            day_night_cycle: true,
            natural_disasters: false,
        }
    }
}

/// Main application state
pub struct SimulatorApp {
    /// Current app state (menu or simulation)
    pub state: AppState,
    /// Configuration being edited in menu
    pub menu_config: SimConfig,
    /// Simulation runner (None until started)
    pub sim: Option<SimulationRunner>,
    /// World renderer
    pub renderer: WorldRenderer,
    /// Is simulation running
    pub running: bool,
    /// Simulation speed (ticks per frame)
    pub speed: u32,
    /// Show grid overlay
    pub show_grid: bool,
    /// Show resource nodes
    pub show_resources: bool,
    /// Selected agent (for inspection)
    pub selected_agent: Option<uuid::Uuid>,
    /// Zoom level
    pub zoom: f32,
    /// Pan offset
    pub pan: egui::Vec2,
    /// Last frame time for FPS calculation
    last_frame: Instant,
    /// Frame count for FPS
    frame_count: u32,
    /// Calculated FPS
    pub fps: f32,
    /// Stats history for graphs
    pub population_history: Vec<f32>,
    pub fitness_history: Vec<f32>,
    /// Max history length
    pub max_history: usize,
    /// Selected preset name
    pub selected_preset: String,
}

impl SimulatorApp {
    /// Create a new application - starts at main menu
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::MainMenu,
            menu_config: SimConfig::default(),
            sim: None,
            renderer: WorldRenderer::new(),
            running: false,
            speed: 1,
            show_grid: false,
            show_resources: true,
            selected_agent: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            last_frame: Instant::now(),
            frame_count: 0,
            fps: 0.0,
            population_history: Vec::with_capacity(500),
            fitness_history: Vec::with_capacity(500),
            max_history: 500,
            selected_preset: "Custom".to_string(),
        }
    }

    /// Start simulation with current config
    pub fn start_simulation(&mut self) {
        let config = WorldConfig {
            starting_year: self.menu_config.starting_year,
            agent_count: self.menu_config.agent_count,
            climate_severity: self.menu_config.climate_severity,
            resource_density: self.menu_config.resource_density,
            ticks_per_second: 60,
            world_width: self.menu_config.world_width,
            world_height: self.menu_config.world_height,
        };

        let mut sim = SimulationRunner::new(config);
        sim.world.factions = FactionManager::new(
            self.menu_config.faction_count,
            self.menu_config.world_width,
            self.menu_config.world_height,
        );
        
        self.sim = Some(sim);
        self.state = AppState::Simulation;
        self.running = false;
        self.population_history.clear();
        self.fitness_history.clear();
        self.selected_agent = None;
        self.zoom = 1.0;
        self.pan = egui::Vec2::ZERO;
    }

    /// Return to main menu
    pub fn return_to_menu(&mut self) {
        self.sim = None;
        self.state = AppState::MainMenu;
        self.running = false;
    }

    /// Apply a preset configuration
    pub fn apply_preset(&mut self, preset: &str) {
        self.selected_preset = preset.to_string();
        match preset {
            "Dawn of Humanity" => {
                self.menu_config = SimConfig {
                    starting_year: -100000,
                    agent_count: 100,
                    faction_count: 4,
                    climate_severity: 0.8,
                    resource_density: 0.3,
                    world_width: 800.0,
                    world_height: 600.0,
                    mutation_rate: 0.2,
                    reproduction_threshold: 0.6,
                    max_agent_age: 5000,
                    day_night_cycle: true,
                    natural_disasters: true,
                };
            }
            "Prehistoric Era" => {
                self.menu_config = SimConfig {
                    starting_year: -10000,
                    agent_count: 500,
                    faction_count: 6,
                    climate_severity: 0.6,
                    resource_density: 0.5,
                    world_width: 1000.0,
                    world_height: 800.0,
                    mutation_rate: 0.15,
                    reproduction_threshold: 0.7,
                    max_agent_age: 8000,
                    day_night_cycle: true,
                    natural_disasters: false,
                };
            }
            "Ancient Civilizations" => {
                self.menu_config = SimConfig {
                    starting_year: -3000,
                    agent_count: 2000,
                    faction_count: 8,
                    climate_severity: 0.4,
                    resource_density: 0.6,
                    world_width: 1200.0,
                    world_height: 900.0,
                    mutation_rate: 0.1,
                    reproduction_threshold: 0.75,
                    max_agent_age: 10000,
                    day_night_cycle: true,
                    natural_disasters: false,
                };
            }
            "Medieval Period" => {
                self.menu_config = SimConfig {
                    starting_year: 500,
                    agent_count: 5000,
                    faction_count: 10,
                    climate_severity: 0.5,
                    resource_density: 0.5,
                    world_width: 1400.0,
                    world_height: 1000.0,
                    mutation_rate: 0.08,
                    reproduction_threshold: 0.8,
                    max_agent_age: 12000,
                    day_night_cycle: true,
                    natural_disasters: false,
                };
            }
            "Massive Scale" => {
                self.menu_config = SimConfig {
                    starting_year: -5000,
                    agent_count: 10000,
                    faction_count: 12,
                    climate_severity: 0.5,
                    resource_density: 0.6,
                    world_width: 2000.0,
                    world_height: 1500.0,
                    mutation_rate: 0.1,
                    reproduction_threshold: 0.7,
                    max_agent_age: 10000,
                    day_night_cycle: false,
                    natural_disasters: false,
                };
            }
            "Survival Mode" => {
                self.menu_config = SimConfig {
                    starting_year: -50000,
                    agent_count: 200,
                    faction_count: 3,
                    climate_severity: 0.95,
                    resource_density: 0.2,
                    world_width: 600.0,
                    world_height: 400.0,
                    mutation_rate: 0.25,
                    reproduction_threshold: 0.5,
                    max_agent_age: 3000,
                    day_night_cycle: true,
                    natural_disasters: true,
                };
            }
            _ => {
                self.menu_config = SimConfig::default();
            }
        }
    }

    /// Update FPS counter
    fn update_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_frame.elapsed();
        if elapsed.as_secs_f32() >= 1.0 {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_frame = Instant::now();
        }
    }

    /// Record statistics history
    pub fn record_history(&mut self) {
        if let Some(ref sim) = self.sim {
            let stats = &sim.world.stats;
            
            if self.population_history.len() >= self.max_history {
                self.population_history.remove(0);
            }
            self.population_history.push(stats.current_population as f32);

            if self.fitness_history.len() >= self.max_history {
                self.fitness_history.remove(0);
            }
            self.fitness_history.push(stats.average_fitness);
        }
    }

    /// Render the main menu
    fn render_main_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    // Title - responsive font size
                    let title_size = if ui.available_width() < 500.0 { 28.0 } else { 42.0 };
                    ui.heading(egui::RichText::new("🌍 REALITY SIMULATOR")
                        .size(title_size)
                        .color(egui::Color32::from_rgb(100, 200, 255)));
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("A Neural Network Humanity Simulation")
                        .size(14.0)
                        .color(egui::Color32::GRAY));
                    ui.add_space(20.0);

                    let available_width = ui.available_width();
                    let panel_width = (available_width - 40.0).min(500.0);
                    
                    // Always vertical layout - presets then config
                    self.render_presets_panel(ui, panel_width);
                    ui.add_space(15.0);
                    self.render_config_panel(ui, panel_width);
                    
                    ui.add_space(20.0);
                    
                    // Start button
                    let button_width = panel_width.min(300.0);
                    let start_button = egui::Button::new(
                        egui::RichText::new("▶  START SIMULATION")
                            .size(18.0)
                            .color(egui::Color32::WHITE)
                    )
                    .min_size(egui::vec2(button_width, 40.0))
                    .fill(egui::Color32::from_rgb(40, 120, 80));
                    
                    if ui.add(start_button).clicked() {
                        self.start_simulation();
                    }
                    
                    ui.add_space(15.0);
                    
                    // Footer info
                    ui.label(egui::RichText::new(format!(
                        "{} agents | {} factions | {} | {:.0}% climate",
                        self.menu_config.agent_count,
                        self.menu_config.faction_count,
                        if self.menu_config.starting_year < 0 {
                            format!("{} BCE", -self.menu_config.starting_year)
                        } else {
                            format!("{} CE", self.menu_config.starting_year)
                        },
                        self.menu_config.climate_severity * 100.0
                    )).size(11.0).color(egui::Color32::GRAY));
                    
                    ui.add_space(10.0);
                });
            });
        });
    }

    /// Render presets panel
    fn render_presets_panel(&mut self, ui: &mut egui::Ui, width: f32) {
        egui::Frame::dark_canvas(ui.style())
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.set_width(width);
                ui.heading("📋 Presets");
                ui.add_space(8.0);
                
                let presets = [
                    ("Dawn of Humanity", "100 agents, harsh survival"),
                    ("Prehistoric Era", "500 agents, balanced start"),
                    ("Ancient Civilizations", "2000 agents, fertile lands"),
                    ("Medieval Period", "5000 agents, established world"),
                    ("Massive Scale", "10000 agents, stress test"),
                    ("Survival Mode", "200 agents, extreme difficulty"),
                ];
                
                for (name, desc) in presets {
                    let selected = self.selected_preset == name;
                    if ui.selectable_label(selected, egui::RichText::new(name).size(14.0)).clicked() {
                        self.apply_preset(name);
                    }
                    ui.label(egui::RichText::new(desc).size(11.0).color(egui::Color32::GRAY));
                    ui.add_space(3.0);
                }
            });
    }

    /// Render configuration panel - always vertical layout
    fn render_config_panel(&mut self, ui: &mut egui::Ui, width: f32) {
        egui::Frame::dark_canvas(ui.style())
            .inner_margin(15.0)
            .show(ui, |ui| {
                ui.set_width(width);
                ui.heading("⚙️ Configuration");
                ui.add_space(8.0);
                
                // Always use single column (vertical) layout
                self.render_config_column_1(ui);
                ui.add_space(8.0);
                self.render_config_column_2(ui);
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);
                
                // Advanced options - always vertical
                ui.checkbox(&mut self.menu_config.day_night_cycle, "Day/Night Cycle");
                ui.checkbox(&mut self.menu_config.natural_disasters, "Natural Disasters");
            });
    }

    /// Render first column of configuration options
    fn render_config_column_1(&mut self, ui: &mut egui::Ui) {
        ui.label("Starting Year");
        let year_suffix = if self.menu_config.starting_year < 0 { " BCE" } else { " CE" };
        ui.add(egui::Slider::new(&mut self.menu_config.starting_year, -100000..=2000)
            .suffix(year_suffix));
        
        ui.add_space(6.0);
        ui.label("Initial Agents");
        ui.add(egui::Slider::new(&mut self.menu_config.agent_count, 10..=50000)
            .logarithmic(true));
        
        ui.add_space(6.0);
        ui.label("Number of Factions");
        ui.add(egui::Slider::new(&mut self.menu_config.faction_count, 2..=12));
        
        ui.add_space(6.0);
        ui.label("Climate Severity");
        ui.add(egui::Slider::new(&mut self.menu_config.climate_severity, 0.0..=1.0)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
        
        ui.add_space(6.0);
        ui.label("Resource Density");
        ui.add(egui::Slider::new(&mut self.menu_config.resource_density, 0.0..=1.0)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
    }

    /// Render second column of configuration options
    fn render_config_column_2(&mut self, ui: &mut egui::Ui) {
        ui.label("World Width");
        ui.add(egui::Slider::new(&mut self.menu_config.world_width, 400.0..=3000.0)
            .suffix(" px"));
        
        ui.add_space(6.0);
        ui.label("World Height");
        ui.add(egui::Slider::new(&mut self.menu_config.world_height, 300.0..=2000.0)
            .suffix(" px"));
        
        ui.add_space(6.0);
        ui.label("Mutation Rate");
        ui.add(egui::Slider::new(&mut self.menu_config.mutation_rate, 0.0..=0.5)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
        
        ui.add_space(6.0);
        ui.label("Reproduction Threshold");
        ui.add(egui::Slider::new(&mut self.menu_config.reproduction_threshold, 0.3..=1.0)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)));
        
        ui.add_space(6.0);
        ui.label("Max Agent Age (ticks)");
        ui.add(egui::Slider::new(&mut self.menu_config.max_agent_age, 1000..=50000)
            .logarithmic(true));
    }

    /// Render the simulation view
    fn render_simulation(&mut self, ctx: &egui::Context) {
        // Run simulation ticks if running
        if self.running {
            if let Some(ref mut sim) = self.sim {
                for _ in 0..self.speed {
                    sim.tick();
                }
            }
            self.record_history();
            ctx.request_repaint();
        }

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("🏠 Main Menu").clicked() {
                        self.return_to_menu();
                        ui.close_menu();
                    }
                    if ui.button("🔄 Restart").clicked() {
                        self.start_simulation();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_grid, "Show Grid");
                    ui.checkbox(&mut self.show_resources, "Show Resources");
                });
            });
        });

        // Left panel - Controls and Stats with scroll
        egui::SidePanel::left("left_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    panels::controls_panel(ui, self);
                    ui.separator();
                    panels::stats_panel(ui, self);
                    ui.separator();
                    panels::factions_panel(ui, self);
                });
            });

        // Right panel - Agent inspector (if selected) with scroll
        if self.selected_agent.is_some() {
            egui::SidePanel::right("agent_panel")
                .resizable(true)
                .default_width(250.0)
                .min_width(180.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        panels::agent_panel(ui, self);
                    });
                });
        }

        // Bottom panel - Graphs
        egui::TopBottomPanel::bottom("graphs_panel")
            .resizable(true)
            .default_height(150.0)
            .show(ctx, |ui| {
                panels::graphs_panel(ui, self);
            });

        // Central panel - World view
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref sim) = self.sim {
                let render_data = self.collect_render_data_from(sim);
                let result = self.renderer.render(ui, &render_data);
                
                if let Some(agent_id) = result.clicked_agent {
                    self.selected_agent = Some(agent_id);
                }
                self.pan += result.pan_delta;
                if result.zoom_delta != 0.0 {
                    self.zoom = (self.zoom * (1.0 + result.zoom_delta)).clamp(0.25, 4.0);
                }
            }
        });
    }

    /// Collect render data from simulation
    fn collect_render_data_from(&self, sim: &SimulationRunner) -> RenderData {
        let agents: Vec<AgentRenderData> = sim.world.agents.iter()
            .map(|a| {
                let agent = a.read();
                let faction_color = sim.world.factions.get_color(agent.faction);
                AgentRenderData {
                    id: agent.id,
                    x: agent.position.x,
                    y: agent.position.y,
                    color: egui::Color32::from_rgb(faction_color.r, faction_color.g, faction_color.b),
                    alive: agent.alive,
                }
            })
            .collect();

        let resources: Vec<ResourceRenderData> = sim.world.resources.iter()
            .map(|r| ResourceRenderData {
                x: r.position.x,
                y: r.position.y,
                resource_type: match r.resource_type {
                    ResourceType::Food => 0,
                    ResourceType::Water => 1,
                    ResourceType::Shelter => 2,
                    ResourceType::Material => 3,
                },
                amount: r.amount,
            })
            .collect();

        let factions: Vec<FactionRenderData> = sim.world.factions.factions.iter()
            .map(|f| FactionRenderData {
                center_x: f.territory_center.0,
                center_y: f.territory_center.1,
                color: egui::Color32::from_rgb(f.color.r, f.color.g, f.color.b),
            })
            .collect();

        RenderData {
            world_width: sim.world.config.world_width,
            world_height: sim.world.config.world_height,
            zoom: self.zoom,
            pan: self.pan,
            show_grid: self.show_grid,
            show_resources: self.show_resources,
            selected_agent: self.selected_agent,
            population: sim.world.stats.current_population,
            tick: sim.world.tick,
            agents,
            resources,
            factions,
        }
    }
}

impl eframe::App for SimulatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_fps();

        match self.state {
            AppState::MainMenu => self.render_main_menu(ctx),
            AppState::Simulation => self.render_simulation(ctx),
        }
    }
}

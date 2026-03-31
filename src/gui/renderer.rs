//! World renderer - draws agents as colored dots

use eframe::egui;

/// Data needed for rendering (avoids borrow issues)
pub struct RenderData {
    pub world_width: f32,
    pub world_height: f32,
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub show_grid: bool,
    pub show_resources: bool,
    pub selected_agent: Option<uuid::Uuid>,
    pub population: usize,
    pub tick: u64,
    pub agents: Vec<AgentRenderData>,
    pub resources: Vec<ResourceRenderData>,
    pub factions: Vec<FactionRenderData>,
}

pub struct AgentRenderData {
    pub id: uuid::Uuid,
    pub x: f32,
    pub y: f32,
    pub color: egui::Color32,
    pub alive: bool,
}

pub struct ResourceRenderData {
    pub x: f32,
    pub y: f32,
    pub resource_type: u8, // 0=food, 1=water, 2=shelter, 3=material
    pub amount: f32,
}

pub struct FactionRenderData {
    pub center_x: f32,
    pub center_y: f32,
    pub color: egui::Color32,
}

/// Result of rendering (clicks, etc)
pub struct RenderResult {
    pub clicked_agent: Option<uuid::Uuid>,
    pub pan_delta: egui::Vec2,
    pub zoom_delta: f32,
}

/// Renders the world view
pub struct WorldRenderer {
    /// Cached agent positions and colors for rendering
    agent_cache: Vec<(egui::Pos2, egui::Color32, bool)>, // (pos, color, is_selected)
}

impl WorldRenderer {
    pub fn new() -> Self {
        Self {
            agent_cache: Vec::with_capacity(10000),
        }
    }

    /// Render the world view
    pub fn render(&mut self, ui: &mut egui::Ui, data: &RenderData) -> RenderResult {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());
        
        let rect = response.rect;
        
        let mut result = RenderResult {
            clicked_agent: None,
            pan_delta: egui::Vec2::ZERO,
            zoom_delta: 0.0,
        };
        
        // Background
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 25, 30));

        // Calculate view transform
        let world_width = data.world_width;
        let world_height = data.world_height;
        
        let scale_x = (rect.width() / world_width) * data.zoom;
        let scale_y = (rect.height() / world_height) * data.zoom;
        let scale = scale_x.min(scale_y);

        let offset_x = rect.min.x + (rect.width() - world_width * scale) / 2.0 + data.pan.x;
        let offset_y = rect.min.y + (rect.height() - world_height * scale) / 2.0 + data.pan.y;

        // World to screen transform
        let world_to_screen = |x: f32, y: f32| -> egui::Pos2 {
            egui::pos2(offset_x + x * scale, offset_y + y * scale)
        };

        // Draw world bounds
        let world_rect = egui::Rect::from_min_max(
            world_to_screen(0.0, 0.0),
            world_to_screen(world_width, world_height),
        );
        painter.rect_stroke(world_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(60, 70, 80)), egui::StrokeKind::Outside);

        // Draw grid if enabled
        if data.show_grid {
            let grid_step = 100.0;
            let grid_color = egui::Color32::from_rgba_unmultiplied(100, 100, 100, 30);
            
            let mut x = 0.0;
            while x <= world_width {
                let p1 = world_to_screen(x, 0.0);
                let p2 = world_to_screen(x, world_height);
                painter.line_segment([p1, p2], egui::Stroke::new(1.0, grid_color));
                x += grid_step;
            }
            
            let mut y = 0.0;
            while y <= world_height {
                let p1 = world_to_screen(0.0, y);
                let p2 = world_to_screen(world_width, y);
                painter.line_segment([p1, p2], egui::Stroke::new(1.0, grid_color));
                y += grid_step;
            }
        }

        // Draw resources if enabled
        if data.show_resources {
            for resource in &data.resources {
                let pos = world_to_screen(resource.x, resource.y);
                let color = match resource.resource_type {
                    0 => egui::Color32::from_rgb(139, 195, 74),  // Food
                    1 => egui::Color32::from_rgb(66, 165, 245),  // Water
                    2 => egui::Color32::from_rgb(121, 85, 72),   // Shelter
                    _ => egui::Color32::from_rgb(158, 158, 158), // Material
                };
                let size = (resource.amount / 50.0).clamp(2.0, 6.0) * scale.sqrt();
                painter.rect_filled(
                    egui::Rect::from_center_size(pos, egui::vec2(size, size)),
                    1.0,
                    color.gamma_multiply(0.6),
                );
            }
        }

        // Draw faction territory centers (subtle)
        for faction in &data.factions {
            let pos = world_to_screen(faction.center_x, faction.center_y);
            let color = egui::Color32::from_rgba_unmultiplied(
                faction.color.r(),
                faction.color.g(),
                faction.color.b(),
                30,
            );
            painter.circle_filled(pos, 30.0 * scale.sqrt(), color);
        }

        // Cache agent data for rendering
        self.agent_cache.clear();
        for agent in &data.agents {
            if !agent.alive {
                continue;
            }
            
            let pos = world_to_screen(agent.x, agent.y);
            let is_selected = data.selected_agent == Some(agent.id);
            
            self.agent_cache.push((pos, agent.color, is_selected));
        }

        // Draw agents as dots
        let base_radius = (3.0 * scale.sqrt()).max(2.0);
        for &(pos, color, is_selected) in &self.agent_cache {
            if is_selected {
                // Selected agent highlight
                painter.circle_stroke(pos, base_radius + 4.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            }
            painter.circle_filled(pos, base_radius, color);
        }

        // Handle interactions
        if response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                // Find clicked agent
                let mut closest_agent: Option<(f32, uuid::Uuid)> = None;
                
                for agent in &data.agents {
                    if !agent.alive {
                        continue;
                    }
                    
                    let agent_screen_pos = world_to_screen(agent.x, agent.y);
                    let dist = click_pos.distance(agent_screen_pos);
                    
                    if dist < base_radius + 5.0 {
                        if closest_agent.is_none() || dist < closest_agent.unwrap().0 {
                            closest_agent = Some((dist, agent.id));
                        }
                    }
                }
                
                result.clicked_agent = closest_agent.map(|(_, id)| id);
            }
        }

        // Handle panning
        if response.dragged() {
            result.pan_delta = response.drag_delta();
        }

        // Handle zoom with scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta != 0.0 {
                result.zoom_delta = scroll_delta * 0.001;
            }
        }

        // Draw HUD overlay
        let hud_text = format!(
            "Population: {} | Tick: {} | Zoom: {:.1}x",
            data.population,
            data.tick,
            data.zoom
        );
        painter.text(
            rect.min + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            hud_text,
            egui::FontId::proportional(14.0),
            egui::Color32::WHITE,
        );

        // Instructions
        let help_text = "Click agent to select | Drag to pan | Scroll to zoom";
        painter.text(
            egui::pos2(rect.max.x - 10.0, rect.min.y + 10.0),
            egui::Align2::RIGHT_TOP,
            help_text,
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(150, 150, 150),
        );

        result
    }
}

impl Default for WorldRenderer {
    fn default() -> Self {
        Self::new()
    }
}

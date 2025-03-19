use eframe::egui;
use egui::{Color32, Stroke, Rounding, Vec2, Rect};

pub struct UiComponents;

impl UiComponents {
    pub fn status_bar(ui: &mut egui::Ui, text: &str, additional_info: Option<&str>) {
        ui.horizontal(|ui| {
            ui.label(text);
            
            if let Some(info) = additional_info {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(info);
                });
            }
        });
    }
    
    pub fn file_tab(ui: &mut egui::Ui, name: &str, is_active: bool, is_modified: bool) -> bool {
        let mut clicked = false;
        
        let padding = Vec2::new(10.0, 5.0);
        let rounding = Rounding::same(4.0);
        
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().min(120.0), 24.0),
            egui::Sense::click(),
        );
        
        if response.clicked() {
            clicked = true;
        }
        
        let bg_color = if is_active {
            ui.visuals().widgets.active.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        
        let stroke = if is_active {
            ui.visuals().widgets.active.bg_stroke
        } else {
            Stroke::NONE
        };
        
        let text_color = if is_active {
            ui.visuals().widgets.active.fg_stroke.color
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        };
        
        let painter = ui.painter();
        
        painter.rect(rect, rounding, bg_color, stroke);
        
        let mut tab_text = name.to_string();
        if is_modified {
            tab_text.push('*');
        }
        
        let text_rect = rect.shrink2(padding);
        painter.text(
            text_rect.center(),
            egui::Align2::CENTER_CENTER,
            &tab_text,
            egui::FontId::default(),
            text_color,
        );
        
        clicked
    }
    
    pub fn line_info_panel(ui: &mut egui::Ui, line: usize, column: usize, selection: Option<(usize, usize)>) {
        ui.horizontal(|ui| {
            ui.label(format!("Ln {}, Col {}", line + 1, column + 1));
            
            if let Some((start, end)) = selection {
                ui.label(format!("Sel: {} chars", end - start));
            }
        });
    }
    
    pub fn search_box(ui: &mut egui::Ui, text: &mut String) -> bool {
        let response = ui.horizontal(|ui| {
            ui.label("Find:");
            let text_edit = ui.text_edit_singleline(text);
            text_edit.lost_focus()
        });
        
        response.inner
    }
    
    pub fn replace_box(ui: &mut egui::Ui, find: &mut String, replace: &mut String) -> (bool, bool) {
        let (find_changed, replace_clicked) = ui.horizontal(|ui| {
            ui.label("Replace:");
            let text_edit = ui.text_edit_singleline(find);
            ui.label("With:");
            let _ = ui.text_edit_singleline(replace);
            let replace_btn = ui.button("Replace").clicked();
            (text_edit.lost_focus(), replace_btn)
        }).inner;
        
        (find_changed, replace_clicked)
    }
    
    pub fn draw_tooltip(ui: &egui::Ui, text: &str, rect: Rect) {
        let layer_id = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tooltip"));
        let tooltip_rect = Rect::from_min_size(
            rect.left_bottom() + Vec2::new(0.0, 5.0),
            Vec2::new(200.0, 40.0),
        );
        
        let painter = ui.ctx().layer_painter(layer_id);
        painter.rect(
            tooltip_rect,
            Rounding::same(6.0),
            Color32::from_rgb(50, 50, 50),
            Stroke::NONE,
        );
        
        painter.text(
            tooltip_rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            Color32::WHITE,
        );
    }
} 
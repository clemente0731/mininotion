use eframe::egui::{self, Visuals, Color32, Stroke, Rounding};

pub struct Theme {
    pub name: String,
    pub text_color: Color32,
    pub background_color: Color32,
    pub accent_color: Color32,
}

impl Theme {
    pub fn new(name: &str) -> Self {
        match name {
            "Dark" => Self::dark(),
            "Light" => Self::light(),
            "Blue" => Self::blue(),
            "Green" => Self::green(),
            "Solarized" => Self::solarized(),
            _ => Self::light(),
        }
    }
    
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            text_color: Color32::from_rgb(70, 70, 70),
            background_color: Color32::from_rgb(245, 243, 240),
            accent_color: Color32::from_rgb(100, 130, 170),
        }
    }
    
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            text_color: Color32::from_rgb(210, 210, 210),
            background_color: Color32::from_rgb(40, 42, 45),
            accent_color: Color32::from_rgb(80, 100, 130),
        }
    }
    
    pub fn blue() -> Self {
        Self {
            name: "Blue".to_string(),
            text_color: Color32::from_rgb(230, 230, 230),
            background_color: Color32::from_rgb(45, 55, 68),
            accent_color: Color32::from_rgb(75, 105, 140),
        }
    }
    
    pub fn green() -> Self {
        Self {
            name: "Green".to_string(),
            text_color: Color32::from_rgb(230, 230, 230),
            background_color: Color32::from_rgb(40, 55, 45),
            accent_color: Color32::from_rgb(70, 130, 90),
        }
    }
    
    pub fn solarized() -> Self {
        Self {
            name: "Solarized".to_string(),
            text_color: Color32::from_rgb(101, 123, 131),
            background_color: Color32::from_rgb(253, 246, 227),
            accent_color: Color32::from_rgb(38, 139, 210),
        }
    }
    
    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut visuals = if self.name == "Light" || self.name == "Solarized" {
            Visuals::light()
        } else {
            Visuals::dark()
        };
        
        visuals.override_text_color = Some(self.text_color);
        
        // Window customization
        visuals.window_fill = self.background_color;
        visuals.window_stroke = Stroke::new(1.0, self.accent_color.linear_multiply(0.5));
        visuals.widgets.noninteractive.bg_fill = self.background_color;
        
        // Button customization
        visuals.widgets.inactive.bg_fill = self.background_color.linear_multiply(1.1);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.inactive.rounding = Rounding::same(4.0);
        
        visuals.widgets.hovered.bg_fill = self.accent_color.linear_multiply(0.15);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.hovered.rounding = Rounding::same(4.0);
        
        visuals.widgets.active.bg_fill = self.accent_color.linear_multiply(0.7);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.rounding = Rounding::same(4.0);
        
        visuals.selection.bg_fill = self.accent_color.linear_multiply(0.4);
        visuals.selection.stroke = Stroke::new(1.0, self.accent_color);
        
        // 使用egui默认的字体配置
        // egui的默认字体设置已经对大多数文字有一定支持
        // 我们不需要特别配置，因为它会使用系统字体回退机制
        
        // 调整滚动条风格
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing.y = 6.0; // 增加项目间距
        style.spacing.window_margin = egui::Margin::same(8.0); // 窗口边距
        ctx.set_style(style);
        
        ctx.set_visuals(visuals);
    }
} 
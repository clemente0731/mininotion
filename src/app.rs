use eframe::egui;
use anyhow::Result;
use std::time::Instant;

use crate::editor::{Document, DocumentCollection};
use crate::theme::Theme;
use crate::config::Config;
use crate::ui::UiComponents;

pub struct NotionApp {
    documents: DocumentCollection,
    active_document_index: Option<usize>,
    theme: Theme,
    config: Config,
    show_settings: bool,
    show_about: bool,
    show_find_dialog: bool,
    show_replace_dialog: bool,
    find_text: String,
    replace_text: String,
    status_message: Option<(String, Instant)>,
    show_document_map: bool,
    show_function_list: bool,
}

impl NotionApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // set up custom fonts if needed
        let fonts = egui::FontDefinitions::default();
        // TODO: Add custom fonts if needed
        
        cc.egui_ctx.set_fonts(fonts);
        
        // load config
        let config = Config::load().unwrap_or_default();
        
        // apply theme
        let theme = Theme::new(&config.theme_name);
        theme.apply_to_ctx(&cc.egui_ctx);
        
        Self {
            documents: DocumentCollection::new(),
            active_document_index: None,
            theme,
            config,
            show_settings: false,
            show_about: false,
            show_find_dialog: false,
            show_replace_dialog: false,
            find_text: String::new(),
            replace_text: String::new(),
            status_message: None,
            show_document_map: false,
            show_function_list: false,
        }
    }
    
    pub fn new_document(&mut self) {
        let mut doc = Document::new();
        // 使用配置中的设置
        doc.line_numbers = self.config.line_numbers;
        doc.word_wrap = self.config.word_wrap;
        
        self.documents.add(doc);
        self.active_document_index = Some(self.documents.len() - 1);
        self.set_status_message("New document created");
    }
    
    pub fn open_document(&mut self) -> Result<()> {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md", "rs", "toml", "json", "py", "js", "html", "css"])
            .add_filter("All Files", &["*"])
            .pick_file() 
        {
            match Document::from_file(&path) {
                Ok(mut doc) => {
                    // 使用配置中的设置
                    doc.line_numbers = self.config.line_numbers;
                    doc.word_wrap = self.config.word_wrap;
                    
                    self.documents.add(doc);
                    self.active_document_index = Some(self.documents.len() - 1);
                    self.set_status_message(format!("Opened {}", path.display()));
                    Ok(())
                },
                Err(err) => {
                    self.set_status_message(format!("Error opening file: {}", err));
                    Err(err)
                }
            }
        } else {
            Ok(())
        }
    }
    
    pub fn save_document(&mut self) -> Result<()> {
        if let Some(idx) = self.active_document_index {
            let mut saved_path = None;
            if let Some(doc) = self.documents.get_mut(idx) {
                if doc.path.is_none() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Text", &["txt", "md", "rs", "toml", "json", "py", "js", "html", "css"])
                        .add_filter("All Files", &["*"])
                        .save_file() 
                    {
                        doc.save_to_file(&path)?;
                        saved_path = Some(format!("Saved to {}", path.display()));
                    }
                } else {
                    doc.save()?;
                    if let Some(path) = &doc.path {
                        saved_path = Some(format!("Saved {}", path.display()));
                    }
                }
            }
            
            if let Some(msg) = saved_path {
                self.set_status_message(msg);
            }
        }
        Ok(())
    }
    
    pub fn save_document_as(&mut self) -> Result<()> {
        if let Some(idx) = self.active_document_index {
            if let Some(doc) = self.documents.get_mut(idx) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Text", &["txt", "md", "rs", "toml", "json", "py", "js", "html", "css"])
                    .add_filter("All Files", &["*"])
                    .save_file() 
                {
                    doc.save_to_file(&path)?;
                    self.set_status_message(format!("Saved to {}", path.display()));
                }
            }
        }
        Ok(())
    }
    
    pub fn close_document(&mut self) {
        if let Some(idx) = self.active_document_index {
            // TODO: Check for unsaved changes before closing
            if self.documents.close(idx) {
                self.set_status_message("Document closed");
                if self.documents.len() == 0 {
                    self.active_document_index = None;
                } else {
                    self.active_document_index = Some(idx.min(self.documents.len() - 1));
                }
            }
        }
    }
    
    pub fn find_text(&mut self) {
        if let Some(doc_idx) = self.active_document_index {
            if let Some(doc) = self.documents.get_mut(doc_idx) {
                // 简单查找，仅查找第一个匹配项
                if let Some(pos) = doc.content.find(&self.find_text) {
                    doc.cursor_position = pos;
                    doc.selection = Some((pos, pos + self.find_text.len()));
                    self.set_status_message(format!("Found text at position {}", pos));
                } else {
                    self.set_status_message("Text not found");
                }
            }
        }
    }
    
    pub fn replace_text(&mut self) {
        if let Some(doc_idx) = self.active_document_index {
            if let Some(doc) = self.documents.get_mut(doc_idx) {
                if let Some((start, end)) = doc.selection {
                    // 确保选中的文本与查找文本匹配
                    if doc.content[start..end] == self.find_text {
                        // 执行替换
                        let before = doc.content[..start].to_string();
                        let after = doc.content[end..].to_string();
                        doc.content = format!("{}{}{}", before, self.replace_text, after);
                        doc.selection = Some((start, start + self.replace_text.len()));
                        doc.is_modified = true;
                        self.set_status_message("Text replaced");
                    } else {
                        self.set_status_message("Selected text doesn't match search text");
                    }
                } else {
                    self.set_status_message("No text selected");
                }
            }
        }
    }
    
    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some((message.into(), Instant::now()));
    }
    
    pub fn apply_settings_to_documents(&mut self) {
        for i in 0..self.documents.len() {
            if let Some(doc) = self.documents.get_mut(i) {
                doc.line_numbers = self.config.line_numbers;
                doc.word_wrap = self.config.word_wrap;
            }
        }
    }
}

impl eframe::App for NotionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_menu_bar(ctx);
        self.show_tabs_bar(ctx);
        self.show_document_area(ctx);
        self.show_status_bar(ctx);
        
        if self.show_settings {
            self.show_settings_window(ctx);
        }
        
        if self.show_about {
            self.show_about_window(ctx);
        }
        
        if self.show_find_dialog {
            self.show_find_window(ctx);
        }
        
        if self.show_replace_dialog {
            self.show_replace_window(ctx);
        }
        
        if self.show_document_map {
            self.show_document_map_panel(ctx);
        }
        
        if self.show_function_list {
            self.show_function_list_panel(ctx);
        }
    }
}

impl NotionApp {
    fn show_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_document();
                        ui.close_menu();
                    }
                    
                    if ui.button("Open").clicked() {
                        if let Err(err) = self.open_document() {
                            log::error!("Failed to open document: {}", err);
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    let can_save = self.active_document_index.is_some();
                    if ui.add_enabled(can_save, egui::Button::new("Save")).clicked() {
                        if let Err(err) = self.save_document() {
                            log::error!("Failed to save document: {}", err);
                        }
                        ui.close_menu();
                    }
                    
                    if ui.add_enabled(can_save, egui::Button::new("Save As...")).clicked() {
                        if let Err(err) = self.save_document_as() {
                            log::error!("Failed to save document: {}", err);
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.add_enabled(can_save, egui::Button::new("Close")).clicked() {
                        self.close_document();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Exit").clicked() {
                        // TODO: Check for unsaved changes
                        std::process::exit(0);
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    // TODO: Implement edit menu (copy, paste, undo, redo)
                    if ui.button("Undo").clicked() {
                        // TODO
                        ui.close_menu();
                    }
                    
                    if ui.button("Redo").clicked() {
                        // TODO
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Cut").clicked() {
                        // TODO
                        ui.close_menu();
                    }
                    
                    if ui.button("Copy").clicked() {
                        // TODO
                        ui.close_menu();
                    }
                    
                    if ui.button("Paste").clicked() {
                        // TODO
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Find...").clicked() {
                        self.show_find_dialog = true;
                        ui.close_menu();
                    }
                    
                    if ui.button("Replace...").clicked() {
                        self.show_replace_dialog = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.config.word_wrap, "Word Wrap").clicked() {
                        if let Err(err) = self.config.save() {
                            log::error!("Failed to save config: {}", err);
                        }
                        self.apply_settings_to_documents();
                    }
                    
                    if ui.checkbox(&mut self.config.line_numbers, "Line Numbers").clicked() {
                        if let Err(err) = self.config.save() {
                            log::error!("Failed to save config: {}", err);
                        }
                        self.apply_settings_to_documents();
                    }
                    
                    if ui.checkbox(&mut self.config.syntax_highlighting, "Syntax Highlighting").clicked() {
                        if let Err(err) = self.config.save() {
                            log::error!("Failed to save config: {}", err);
                        }
                        // TODO: Apply syntax highlighting setting
                    }
                    
                    ui.separator();
                    
                    if ui.checkbox(&mut self.show_document_map, "Document Map").clicked() {
                        // 切换文档映射侧边栏
                    }
                    
                    if ui.checkbox(&mut self.show_function_list, "Function List").clicked() {
                        // 切换函数列表侧边栏
                    }
                    
                    ui.separator();
                    
                    if ui.button("Settings").clicked() {
                        self.show_settings = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
    }
    
    fn show_tabs_bar(&mut self, ctx: &egui::Context) {
        if self.documents.len() > 0 {
            egui::TopBottomPanel::top("tabs_bar")
                .show_separator_line(true)
                .show(ctx, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut clicked_idx = None;
                        let mut close_idx = None;
                        
                        for i in 0..self.documents.len() {
                            let doc = self.documents.get(i).unwrap();
                            let is_active = Some(i) == self.active_document_index;
                            
                            ui.horizontal(|ui| {
                                if UiComponents::file_tab(
                                    ui,
                                    &doc.filename,
                                    is_active,
                                    doc.is_modified
                                ) {
                                    clicked_idx = Some(i);
                                }
                                
                                if ui.small_button("×").clicked() {
                                    close_idx = Some(i);
                                }
                            });
                            
                            ui.separator();
                        }
                        
                        if let Some(idx) = clicked_idx {
                            self.active_document_index = Some(idx);
                        }
                        
                        if let Some(idx) = close_idx {
                            if self.documents.close(idx) {
                                if self.documents.len() == 0 {
                                    self.active_document_index = None;
                                } else {
                                    self.active_document_index = Some(idx.min(self.documents.len() - 1));
                                }
                            }
                        }
                    });
                });
        }
    }
    
    fn show_document_area(&mut self, ctx: &egui::Context) {
        let panel = egui::CentralPanel::default();
        
        if self.show_document_map {
            egui::SidePanel::right("document_map")
                .resizable(true)
                .default_width(200.0)
                .width_range(150.0..=400.0)
                .show(ctx, |ui| {
                    ui.heading("Document Map");
                    ui.separator();
                    // 这里实现文档映射功能
                    if let Some(idx) = self.active_document_index {
                        if let Some(doc) = self.documents.get(idx) {
                            // 显示文档的简化缩略图
                            let lines = doc.content.lines().take(100);
                            for line in lines {
                                let shortened = if line.len() > 30 {
                                    format!("{}...", &line[0..30])
                                } else {
                                    line.to_string()
                                };
                                ui.label(egui::RichText::new(shortened).weak().small());
                            }
                        }
                    }
                });
        }
        
        panel.show(ctx, |ui| {
            if let Some(idx) = self.active_document_index {
                if let Some(doc) = self.documents.get_mut(idx) {
                    doc.ui(ui);
                }
            } else {
                // Show welcome screen
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Welcome to Notion++");
                    ui.add_space(20.0);
                    ui.label("A modern, cross-platform text editor written in Rust");
                    ui.add_space(40.0);
                    
                    if ui.button("New Document").clicked() {
                        self.new_document();
                    }
                    
                    if ui.button("Open Document").clicked() {
                        if let Err(err) = self.open_document() {
                            log::error!("Failed to open document: {}", err);
                        }
                    }
                });
            }
        });
    }
    
    fn show_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左侧: 状态消息
                if let Some((message, time)) = &self.status_message {
                    let elapsed = time.elapsed().as_secs();
                    // 3秒后消息消失
                    if elapsed < 3 {
                        ui.label(message);
                    } else {
                        self.status_message = None;
                    }
                }
                
                // 右侧: 当前位置和其他信息
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(idx) = self.active_document_index {
                        if let Some(doc) = self.documents.get(idx) {
                            // 编码信息
                            ui.label("UTF-8");
                            
                            // 当前主题名称
                            ui.label(format!("Theme: {}", self.theme.name));
                            
                            // 行列信息
                            let (line, col) = doc.get_current_position();
                            ui.label(format!("Ln {}, Col {}", line + 1, col + 1));
                        }
                    }
                });
            });
        });
    }
    
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        let mut settings_open = self.show_settings;
        let theme_before = self.theme.name.clone();
        let mut apply_settings = false;
        let mut need_save = false;
        let mut font_size = self.config.font_size;
        let mut word_wrap = self.config.word_wrap;
        let mut line_numbers = self.config.line_numbers;
        let mut syntax_highlighting = self.config.syntax_highlighting;
        let mut auto_save = self.config.auto_save;
        let mut auto_save_interval_secs = self.config.auto_save_interval_secs;
        let mut theme_name = self.theme.name.clone();
        
        egui::Window::new("Settings")
            .open(&mut settings_open)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                
                egui::ComboBox::from_label("Theme")
                    .selected_text(&theme_name)
                    .show_ui(ui, |ui| {
                        for name in ["Light", "Dark", "Blue", "Green", "Solarized"] {
                            if ui.selectable_value(&mut theme_name, name.to_string(), name).clicked() {
                                need_save = true;
                            }
                        }
                    });
                
                ui.separator();
                ui.heading("Editor");
                
                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui.add(egui::Slider::new(&mut font_size, 8.0..=24.0).step_by(1.0)).changed() {
                        need_save = true;
                    }
                });
                
                if ui.checkbox(&mut word_wrap, "Word Wrap").changed() {
                    need_save = true;
                }
                
                if ui.checkbox(&mut line_numbers, "Line Numbers").changed() {
                    need_save = true;
                }
                
                if ui.checkbox(&mut syntax_highlighting, "Syntax Highlighting").changed() {
                    need_save = true;
                }
                
                if ui.checkbox(&mut auto_save, "Auto Save").changed() {
                    need_save = true;
                }
                
                if auto_save {
                    ui.horizontal(|ui| {
                        ui.label("Auto Save Interval (seconds):");
                        let mut interval = auto_save_interval_secs as f64;
                        if ui.add(egui::Slider::new(&mut interval, 10.0..=300.0).step_by(10.0)).changed() {
                            auto_save_interval_secs = interval as u64;
                            need_save = true;
                        }
                    });
                }
                
                ui.separator();
                
                if ui.button("Save Settings").clicked() {
                    apply_settings = true;
                }
            });
        
        if apply_settings || (need_save && settings_open != self.show_settings) {
            // 应用设置
            if theme_name != theme_before {
                self.theme = Theme::new(&theme_name);
                self.theme.apply_to_ctx(ctx);
                self.config.theme_name = theme_name;
            }
            
            self.config.font_size = font_size;
            self.config.word_wrap = word_wrap;
            self.config.line_numbers = line_numbers;
            self.config.syntax_highlighting = syntax_highlighting;
            self.config.auto_save = auto_save;
            self.config.auto_save_interval_secs = auto_save_interval_secs;
            
            if let Err(err) = self.config.save() {
                log::error!("Failed to save config: {}", err);
            }
            
            self.apply_settings_to_documents();
            self.set_status_message("Settings saved");
        }
        
        self.show_settings = settings_open;
    }
    
    fn show_about_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("About Notion++")
            .open(&mut self.show_about)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Notion++");
                    ui.label("Version 0.1.0");
                    ui.label("A modern, cross-platform text editor written in Rust");
                });
                
                ui.separator();
                
                ui.label("Inspired by Notepad++");
                ui.label("Uses egui for the UI");
                ui.label("Licensed under the GPL");
                
                ui.separator();
                
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("GitHub Repository: ");
                    ui.hyperlink_to("https://github.com/yourusername/notion-plus-plus", "https://github.com/yourusername/notion-plus-plus");
                });
            });
    }
    
    fn show_find_window(&mut self, ctx: &egui::Context) {
        let mut find_open = self.show_find_dialog;
        let mut find_text = self.find_text.clone();
        let mut button_clicked = None;
        
        egui::Window::new("Find")
            .open(&mut find_open)
            .collapsible(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    let response = ui.text_edit_singleline(&mut find_text);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        button_clicked = Some("find");
                    }
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Find Next").clicked() {
                        button_clicked = Some("find");
                    }
                    
                    if ui.button("Close").clicked() {
                        button_clicked = Some("close");
                    }
                });
                
                // TODO: 添加更多选项，如区分大小写、全词匹配等
            });
        
        self.find_text = find_text;
        
        if let Some(action) = button_clicked {
            match action {
                "find" => self.find_text(),
                "close" => find_open = false,
                _ => {}
            }
        }
        
        self.show_find_dialog = find_open;
    }
    
    fn show_replace_window(&mut self, ctx: &egui::Context) {
        let mut replace_open = self.show_replace_dialog;
        let mut find_text = self.find_text.clone();
        let mut replace_text = self.replace_text.clone();
        let mut button_clicked = None;
        
        egui::Window::new("Replace")
            .open(&mut replace_open)
            .collapsible(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    ui.text_edit_singleline(&mut find_text);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Replace with:");
                    ui.text_edit_singleline(&mut replace_text);
                });
                
                ui.horizontal(|ui| {
                    if ui.button("Find Next").clicked() {
                        button_clicked = Some("find");
                    }
                    
                    if ui.button("Replace").clicked() {
                        button_clicked = Some("replace");
                    }
                    
                    if ui.button("Close").clicked() {
                        button_clicked = Some("close");
                    }
                });
                
                // TODO: 添加更多选项，如区分大小写、全词匹配等
            });
        
        self.find_text = find_text;
        self.replace_text = replace_text;
        
        if let Some(action) = button_clicked {
            match action {
                "find" => self.find_text(),
                "replace" => self.replace_text(),
                "close" => replace_open = false,
                _ => {}
            }
        }
        
        self.show_replace_dialog = replace_open;
    }
    
    fn show_document_map_panel(&mut self, _ctx: &egui::Context) {
        // 文档映射面板在show_document_area中实现
    }
    
    fn show_function_list_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("function_list")
            .resizable(true)
            .default_width(200.0)
            .width_range(150.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("Function List");
                ui.separator();
                
                if let Some(idx) = self.active_document_index {
                    let mut jump_to_line = None;
                    
                    // 先获取内容进行解析
                    if let Some(doc) = self.documents.get(idx) {
                        // 简单解析函数列表
                        let lines: Vec<_> = doc.content.lines().collect();
                        for (i, line) in lines.iter().enumerate() {
                            let line = line.trim();
                            // 非常简单的判断，实际应用中应该使用正则表达式或专门的解析器
                            if line.contains("fn ") || line.contains("function ") || line.contains("def ") || 
                               line.contains("class ") || line.contains("struct ") || line.contains("impl ") {
                                if ui.selectable_label(false, line).clicked() {
                                    // 跳转到该函数行，保存行号
                                    jump_to_line = Some(i);
                                }
                            }
                        }
                    }
                    
                    // 然后再执行跳转
                    if let Some(line) = jump_to_line {
                        if let Some(doc) = self.documents.get_mut(idx) {
                            doc.current_line = line;
                            doc.scroll_to_line(line);
                        }
                    }
                }
            });
    }
} 
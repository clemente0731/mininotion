use eframe::egui;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxSet, SyntaxReference};

pub struct Document {
    pub path: Option<PathBuf>,
    pub content: String,
    pub filename: String,
    pub is_modified: bool,
    pub scroll_offset: f32,
    pub cursor_position: usize,
    pub syntax: Option<SyntaxReference>,
    pub line_numbers: bool,
    pub word_wrap: bool,
    pub selection: Option<(usize, usize)>,
    pub current_line: usize,
    pub current_column: usize,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Document {
    pub fn new() -> Self {
        Self {
            path: None,
            content: String::new(),
            filename: "Untitled".to_string(),
            is_modified: false,
            scroll_offset: 0.0,
            cursor_position: 0,
            syntax: None,
            line_numbers: true,
            word_wrap: true,
            selection: None,
            current_line: 0,
            current_column: 0,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
    
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
            
        let filename = path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        let syntax_set = SyntaxSet::load_defaults_newlines();
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let syntax = syntax_set.find_syntax_by_extension(extension).cloned();
            
        Ok(Self {
            path: Some(path.to_path_buf()),
            content,
            filename,
            is_modified: false,
            scroll_offset: 0.0,
            cursor_position: 0,
            syntax,
            line_numbers: true,
            word_wrap: true,
            selection: None,
            current_line: 0,
            current_column: 0,
            syntax_set,
            theme_set: ThemeSet::load_defaults(),
        })
    }
    
    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = self.path.clone() {
            self.save_to_file(&path)?;
        } else {
            // If no path, do nothing
            return Ok(());
        }
        Ok(())
    }
    
    pub fn save_to_file(&mut self, path: &Path) -> Result<()> {
        fs::write(path, &self.content)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;
            
        self.path = Some(path.to_path_buf());
        self.filename = path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        self.is_modified = false;
        
        Ok(())
    }
    
    pub fn get_line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }
    
    pub fn get_current_position(&self) -> (usize, usize) {
        (self.current_line, self.current_column)
    }
    
    pub fn scroll_to_line(&mut self, line: usize) {
        // 设置当前行并请求滚动到该行
        self.current_line = line;
        self.scroll_offset = line as f32 * 18.0; // 近似行高
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let _panel_width = ui.available_width();
        let _panel_height = ui.available_height();
        
        // 创建滚动区域以支持垂直滚动
        let mut scroll_area = egui::ScrollArea::vertical()
            .id_source("editor_scroll")
            .auto_shrink([false; 2])
            .enable_scrolling(true);
        
        // 设置滚动区域初始滚动位置
        let offset = egui::vec2(0.0, self.scroll_offset);
        scroll_area = scroll_area.scroll_offset(offset);
        
        // 显示滚动区域内容
        scroll_area.show(ui, |ui| {
            let avail_width = ui.available_width();
            let _start_rect = ui.min_rect();
            
            ui.horizontal_top(|ui| {
                // 如果启用了行号，在左侧添加行号面板
                if self.line_numbers {
                    let line_count = self.get_line_count();
                    let digit_count = (line_count as f32).log10().floor() as usize + 1;
                    let line_number_width = digit_count as f32 * 10.0 + 16.0;
                    
                    ui.vertical(|ui| {
                        ui.set_min_width(line_number_width);
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                        
                        let _line_height = ui.text_style_height(&egui::TextStyle::Monospace);
                        let lines = self.content.lines().count().max(1);
                        
                        for line_number in 1..=lines {
                            let text = format!("{:>width$}", line_number, width = digit_count);
                            let is_current_line = line_number - 1 == self.current_line;
                            
                            if is_current_line {
                                ui.label(egui::RichText::new(text).strong().color(ui.visuals().strong_text_color()));
                            } else {
                                ui.label(egui::RichText::new(text).weak().color(ui.visuals().weak_text_color()));
                            }
                        }
                        
                        // 如果内容为空，至少显示一个行号
                        if lines == 0 {
                            ui.label("1");
                        }
                    });
                    
                    // 在行号和内容之间添加分隔线
                    let line_pos = ui.cursor().min.x;
                    let top = ui.min_rect().top();
                    let bottom = top + (self.get_line_count() as f32) * ui.text_style_height(&egui::TextStyle::Monospace);
                    
                    ui.painter().line_segment(
                        [egui::pos2(line_pos, top), egui::pos2(line_pos, bottom)],
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                }
                
                // 主要文本编辑区域
                let text_edit_width = if self.line_numbers {
                    avail_width - 30.0 // 减去行号宽度和内边距
                } else {
                    avail_width
                };
                
                let mut text_edit = egui::TextEdit::multiline(&mut self.content)
                    .desired_width(text_edit_width)
                    .desired_rows(30)
                    .lock_focus(true)
                    .code_editor()
                    .hint_text("Type here...")
                    .cursor_at_end(false)
                    .interactive(true);
                
                // 使用固定宽度字体，但支持中日韩文字
                text_edit = text_edit.font(egui::FontId::monospace(14.0));
                
                // 单词换行设置
                if !self.word_wrap {
                    text_edit = text_edit.desired_width(f32::INFINITY);
                }
                
                let response = ui.add(text_edit);
                
                if response.changed() {
                    self.is_modified = true;
                    
                    // 当文本改变时更新光标位置和行列（支持UTF-8多字节字符）
                    self.update_cursor_position_from_content();
                }
                
                // 处理文本选中情况
                if response.has_focus() {
                    // 简化光标位置更新和选择逻辑
                    if response.clicked() || response.dragged() {
                        // 当点击或拖动时，更新光标位置
                        self.update_cursor_position_from_content();
                        
                        // 简单模拟选择：当拖动时选择最后几个字符
                        if response.dragged() && self.cursor_position > 0 {
                            let start = if self.cursor_position > 10 { self.cursor_position - 10 } else { 0 };
                            self.selection = Some((start, self.cursor_position));
                        } else {
                            self.selection = None;
                        }
                    }
                }
            });
            
            // 保存滚动位置，反转滚动方向
            let current_scroll = ui.ctx().input(|i| -i.raw_scroll_delta.y);  // 反转滚动方向
            if current_scroll != 0.0 {
                self.scroll_offset += current_scroll;
                
                // 确保滚动偏移不会为负
                self.scroll_offset = self.scroll_offset.max(0.0);
                
                // 限制最大滚动范围
                let max_offset = (self.get_line_count() as f32) * 18.0;  // 假设每行高度18像素
                self.scroll_offset = self.scroll_offset.min(max_offset);
            }
        });
        
        // 在编辑器底部显示状态栏
        ui.horizontal(|ui| {
            ui.label(format!("Ln {}, Col {}", self.current_line + 1, self.current_column + 1));
            
            if let Some((start, end)) = self.selection {
                ui.label(format!("Sel: {} chars", end - start));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(syntax) = &self.syntax {
                    ui.label(format!("Syntax: {}", syntax.name));
                } else {
                    ui.label("Syntax: Plain Text");
                }
                
                // 显示当前使用的编码
                ui.label("UTF-8");
                
                if self.word_wrap {
                    ui.label("Word Wrap: On");
                } else {
                    ui.label("Word Wrap: Off");
                }
            });
        });
    }
    
    // 新增方法：更新光标位置并考虑UTF-8多字节字符
    fn update_cursor_position_from_content(&mut self) {
        let cursor_pos = self.content.len();
        self.cursor_position = cursor_pos;
        
        // 计算当前行和列，考虑UTF-8多字节字符
        let text_before_cursor = &self.content[..cursor_pos];
        self.current_line = text_before_cursor.chars().filter(|&c| c == '\n').count();
        
        let last_newline_pos = text_before_cursor.rfind('\n');
        
        // 计算列位置（需要按字符计算而非字节）
        if let Some(pos) = last_newline_pos {
            // 有换行符，计算最后一行的列位置
            let last_line_text = &text_before_cursor[(pos + 1)..];
            self.current_column = last_line_text.chars().count();
        } else {
            // 没有换行符，整个文本就是一行
            self.current_column = text_before_cursor.chars().count();
        }
    }
    
    // 计算文本的宽度（以字符数为单位，而非字节数）
    pub fn text_width(&self, text: &str) -> usize {
        text.chars().count()
    }
}

pub struct DocumentCollection {
    documents: Vec<Document>,
}

impl DocumentCollection {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }
    
    pub fn add(&mut self, document: Document) {
        self.documents.push(document);
    }
    
    pub fn get(&self, index: usize) -> Option<&Document> {
        self.documents.get(index)
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Document> {
        self.documents.get_mut(index)
    }
    
    pub fn len(&self) -> usize {
        self.documents.len()
    }
    
    pub fn close(&mut self, index: usize) -> bool {
        if index < self.documents.len() {
            self.documents.remove(index);
            true
        } else {
            false
        }
    }
} 
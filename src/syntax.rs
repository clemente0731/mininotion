use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use std::path::Path;

pub struct SyntaxHighlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub current_theme: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            current_theme: "base16-ocean.dark".to_string(),
        }
    }
    
    pub fn get_syntax_for_file(&self, path: &Path) -> Option<&SyntaxReference> {
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        self.syntax_set.find_syntax_by_extension(extension)
    }
    
    pub fn get_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_name(name)
    }
    
    pub fn get_theme(&self) -> &Theme {
        &self.theme_set.themes[&self.current_theme]
    }
    
    pub fn set_theme(&mut self, name: &str) -> bool {
        if self.theme_set.themes.contains_key(name) {
            self.current_theme = name.to_string();
            true
        } else {
            false
        }
    }
    
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_set.themes.keys().map(|k| k.to_string()).collect()
    }
} 
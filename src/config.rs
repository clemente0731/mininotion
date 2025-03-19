use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme_name: String,
    pub font_size: f32,
    pub word_wrap: bool,
    pub line_numbers: bool,
    pub syntax_highlighting: bool,
    pub auto_save: bool,
    pub auto_save_interval_secs: u64,
    pub recent_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: "Light".to_string(),
            font_size: 14.0,
            word_wrap: true,
            line_numbers: true,
            syntax_highlighting: true,
            auto_save: false,
            auto_save_interval_secs: 60,
            recent_files: Vec::new(),
        }
    }
}

impl Config {
    fn config_dir() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "notion-pp", "notion-pp")?;
        let config_dir = proj_dirs.config_dir();
        
        // Ensure the config directory exists
        if !config_dir.exists() {
            if let Err(err) = fs::create_dir_all(config_dir) {
                log::error!("Failed to create config directory: {}", err);
                return None;
            }
        }
        
        Some(config_dir.to_path_buf())
    }
    
    fn config_file_path() -> Option<PathBuf> {
        let config_dir = Self::config_dir()?;
        Some(config_dir.join("config.json"))
    }
    
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
            
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        let config_str = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            
        let config = serde_json::from_str(&config_str)
            .with_context(|| "Failed to parse config file")?;
            
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
            
        let config_str = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
            
        fs::write(&config_path, config_str)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
            
        Ok(())
    }
    
    pub fn add_recent_file(&mut self, path: &str) {
        // Remove if already exists
        self.recent_files.retain(|p| p != path);
        
        // Add to the beginning of the list
        self.recent_files.insert(0, path.to_string());
        
        // Limit to 10 recent files
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }
} 
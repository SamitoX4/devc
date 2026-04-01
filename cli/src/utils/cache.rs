use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::utils::fetcher::TemplateFetcher;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub templates_version: String,
    pub last_check: String,
    #[serde(default)]
    pub git: GitConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            templates_version: "0.0.0".to_string(),
            last_check: "never".to_string(),
            git: GitConfig::default(),
        }
    }
}

pub struct CacheManager {
    #[allow(dead_code)]
    cache_dir: PathBuf,
    templates_dir: PathBuf,
    bundled_templates_dir: PathBuf,
    config_path: PathBuf,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".devc");
        
        let templates_dir = cache_dir.join("cache").join("templates");
        let config_path = cache_dir.join("config.json");

        let bundled_templates_dir = Self::find_bundled_templates();

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            CacheConfig::default()
        };

        fs::create_dir_all(&templates_dir)
            .context("Failed to create cache directory")?;

        Ok(Self {
            cache_dir,
            templates_dir,
            bundled_templates_dir,
            config_path,
            config,
        })
    }

    fn find_bundled_templates() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("/"));
            let bundled = exe_dir.join("templates");
            if bundled.exists() && bundled.is_dir() {
                return bundled;
            }
        }
        PathBuf::new()
    }

    pub fn has_templates(&self) -> bool {
        if self.templates_dir.join("nodejs").exists() {
            return true;
        }
        if !self.bundled_templates_dir.as_os_str().is_empty() 
           && self.bundled_templates_dir.join("nodejs").exists() {
            return true;
        }
        false
    }

    pub fn templates_dir(&self) -> &PathBuf {
        if self.templates_dir.join("nodejs").exists() {
            return &self.templates_dir;
        }
        if !self.bundled_templates_dir.as_os_str().is_empty() 
           && self.bundled_templates_dir.join("nodejs").exists() {
            return &self.bundled_templates_dir;
        }
        &self.templates_dir
    }

    pub fn get_git_config(&self) -> &GitConfig {
        &self.config.git
    }

    pub fn save_git_config(&mut self, name: &str, email: &str) -> Result<()> {
        self.config.git.name = Some(name.to_string());
        self.config.git.email = Some(email.to_string());
        self.save_config()?;
        Ok(())
    }

    pub async fn check_updates(&self, verbose: bool) -> Result<()> {
        let fetcher = TemplateFetcher::new();

        match fetcher.get_latest_version(verbose).await {
            Ok(latest_version) => {
                if self.config.templates_version != latest_version {
                    if verbose {
                        println!(
                            "New version available: {} (current: {})",
                            latest_version, self.config.templates_version
                        );
                    }
                } else if verbose {
                    println!("Templates are up to date (v{})", self.config.templates_version);
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("Could not check for updates: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn download_templates(&self, force: bool, verbose: bool) -> Result<()> {
        if !self.bundled_templates_dir.as_os_str().is_empty() 
           && self.bundled_templates_dir.join("nodejs").exists() 
           && !force {
            if verbose {
                println!("Using bundled templates (offline mode)");
            }
            if self.templates_dir.join("nodejs").exists() {
                if verbose {
                    println!("Templates already cached, skipping copy");
                }
            } else {
                if verbose {
                    println!("Copying bundled templates to cache...");
                }
                Self::copy_dir_recursive(&self.bundled_templates_dir, &self.templates_dir)?;
            }
            return Ok(());
        }

        let fetcher = TemplateFetcher::new();

        if self.templates_dir.join("nodejs").exists() && !force {
            if verbose {
                println!("Templates already cached, skipping download");
            }
            return Ok(());
        }

        if verbose {
            println!("Downloading templates from repository...");
        }

        fetcher.download_templates(&self.templates_dir, verbose).await?;

        println!("Templates downloaded successfully!");
        Ok(())
    }

    fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> Result<()> {
        fs::create_dir_all(dst)?;
        if let Ok(entries) = fs::read_dir(src) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());
                if src_path.is_dir() {
                    Self::copy_dir_recursive(&src_path, &dst_path)?;
                } else {
                    fs::copy(&src_path, &dst_path)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_available_templates(&self) -> Vec<String> {
        let mut templates = Vec::new();
        let templates_base = self.templates_dir();
        
        if let Ok(entries) = fs::read_dir(templates_base) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy().to_string();
                        if !name_str.starts_with('.') && name_str != "target" {
                            templates.push(name_str);
                        }
                    }
                }
            }
        }

        templates.sort();
        templates
    }

    fn save_config(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}

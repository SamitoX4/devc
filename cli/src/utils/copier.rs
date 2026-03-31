use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct TemplateCopier;

impl TemplateCopier {
    pub fn copy_template(source: &Path, target: &Path) -> Result<()> {
        if !source.exists() {
            anyhow::bail!("Template not found: {}", source.display());
        }

        fs::create_dir_all(target)
            .context("Failed to create target directory")?;

        let devcontainer_src = source.join(".devcontainer");
        let devcontainer_dst = target.join(".devcontainer");

        if devcontainer_src.exists() {
            fs::create_dir_all(&devcontainer_dst)?;

            for entry in fs::read_dir(&devcontainer_src)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    Self::copy_dir_recursive(&path, &devcontainer_dst.join(path.file_name().unwrap()))?;
                } else {
                    fs::copy(&path, &devcontainer_dst.join(path.file_name().unwrap()))?;
                }
            }
        }

        Ok(())
    }

    fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
        fs::create_dir_all(to)?;
        
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            let dest = to.join(path.file_name().unwrap());

            if path.is_dir() {
                Self::copy_dir_recursive(&path, &dest)?;
            } else {
                fs::copy(&path, &dest)?;
            }
        }

        Ok(())
    }

    pub fn find_template_dir(cache_dir: &Path, template_name: &str) -> Option<PathBuf> {
        let template_path = cache_dir.join(template_name);
        if template_path.exists() {
            return Some(template_path);
        }

        for entry in WalkDir::new(cache_dir)
            .max_depth(3)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_dir() {
                let name = entry.file_name();
                if name == template_name {
                    return Some(entry.path().to_path_buf());
                }
            }
        }

        None
    }
}

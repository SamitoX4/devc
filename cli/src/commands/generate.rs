use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, Select};
use crate::utils::cache::CacheManager;
use crate::utils::copier::TemplateCopier;
use crate::utils::merger::ConfigMerger;

pub async fn run(
    template: Option<&str>,
    name: Option<&str>,
    git_name: Option<&str>,
    git_email: Option<&str>,
    cache: &mut CacheManager,
) -> Result<()> {
    println!();
    println!("{}", "DevContainer Generator".bold().cyan());
    println!("{}", "====================".cyan());
    println!();

    // 1. Template
    let selected_template = if let Some(t) = template {
        println!("{}", format!("  Template: {}", t).dimmed());
        t.to_string()
    } else {
        select_template_interactive(cache)?
    };

    // 2. Project Name
    let project_name = if let Some(n) = name {
        println!("{}", format!("  Project: {}", n).dimmed());
        n.to_string()
    } else {
        prompt_project_name()?
    };

    // 3. Target Directory
    let target_dir = prompt_target_directory()?;

    // 4. Git Config
    let git_config = cache.get_git_config();

    let final_git_name = if let Some(n) = git_name {
        println!("{}", format!("  Git Name: {}", n).dimmed());
        Some(n.to_string())
    } else if let Some(n) = git_config.name.as_ref() {
        println!("{}", format!("  Git Name: {} (from config)", n).dimmed());
        Some(n.clone())
    } else {
        Some(prompt_git_name())
    };

    let final_git_email = if let Some(e) = git_email {
        println!("{}", format!("  Git Email: {}", e).dimmed());
        Some(e.to_string())
    } else if let Some(e) = git_config.email.as_ref() {
        println!("{}", format!("  Git Email: {} (from config)", e).dimmed());
        Some(e.clone())
    } else {
        Some(prompt_git_email())
    };

    if let (Some(n), Some(e)) = (&final_git_name, &final_git_email) {
        cache.save_git_config(n, e)?;
    }

    println!();
    println!("{}", format!("Generating {} devcontainer...", selected_template).cyan());
    println!();

    let template_dir = TemplateCopier::find_template_dir(cache.templates_dir(), &selected_template)
        .context("Template not found. Run 'devc update' to download templates.")?;

    TemplateCopier::copy_template(&template_dir, &target_dir)?;

    ConfigMerger::merge_template(
        &target_dir,
        &project_name,
        final_git_name.as_deref(),
        final_git_email.as_deref(),
    )?;
    ConfigMerger::update_docker_compose(&target_dir, &project_name)?;

    let dockerfile_path = target_dir.join(".devcontainer").join("Dockerfile");
    if dockerfile_path.exists() {
        if let Some(custom_args) = prompt_custom_versions(&dockerfile_path)? {
            apply_custom_versions(&dockerfile_path, &custom_args)?;
            println!();
            println!("{}", "✓ Versiones personalizadas aplicadas".green());
        }
    }

    println!();
    println!("{}", "✓ Devcontainer generated successfully!".green());
    println!();
    println!("Location: {}", target_dir.display().to_string().cyan());
    println!();
    println!("Next steps:");
    println!("  1. cd {}", target_dir.display());
    println!("  2. Open in VS Code: code .");
    println!("  3. Press F1 and select: Dev Containers: Reopen in Container");

    Ok(())
}

fn select_template_interactive(cache: &CacheManager) -> Result<String> {
    let templates = cache.get_available_templates();

    if templates.is_empty() {
        anyhow::bail!("No templates found. Run 'devc update' to download templates.");
    }

    println!("{}", "Select a template:".bold());

    let selection = Select::new()
        .items(&templates)
        .default(0)
        .interact()
        .context("Failed to display template selection")?;

    let selected = &templates[selection];
    println!("{}", format!("  ✓ {}", selected).green());
    Ok(selected.to_string())
}

fn prompt_project_name() -> Result<String> {
    let default_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    let input: String = Input::new()
        .with_prompt("Project name")
        .default(default_name)
        .interact_text()
        .context("Failed to read project name")?;

    println!("{}", format!("  ✓ {}", input).green());
    Ok(input)
}

fn prompt_target_directory() -> Result<std::path::PathBuf> {
    let current_dir = std::env::current_dir()
        .context("Could not determine current directory")?;
    let default_path = current_dir.to_string_lossy().to_string();

    let input: String = Input::new()
        .with_prompt("¿En qué carpeta querés generar el devcontainer?")
        .default(default_path)
        .allow_empty(true)
        .interact_text()
        .context("Failed to read target directory")?;

    let target = if input.trim().is_empty() {
        current_dir
    } else {
        let path = std::path::PathBuf::from(input.trim());
        if path.is_relative() {
            current_dir.join(path)
        } else {
            path
        }
    };

    if !target.exists() {
        std::fs::create_dir_all(&target)
            .context(format!("Failed to create directory: {}", target.display()))?;
        println!("{}", format!("  ✓ Carpeta creada: {}", target.display()).green());
    }

    Ok(target)
}

fn prompt_git_name() -> String {
    let input: String = Input::new()
        .with_prompt("Git User Name")
        .default("user".to_string())
        .interact_text()
        .unwrap_or_else(|_| "user".to_string());

    input
}

fn prompt_git_email() -> String {
    let input: String = Input::new()
        .with_prompt("Git User Email")
        .default("user@example.com".to_string())
        .interact_text()
        .unwrap_or_else(|_| "user@example.com".to_string());

    input
}

fn prompt_custom_versions(dockerfile_path: &std::path::Path) -> Result<Option<Vec<(String, String)>>> {
    let content = std::fs::read_to_string(dockerfile_path)?;
    let mut args = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("ARG ") {
            let rest = &trimmed[4..];
            if let Some(eq_pos) = rest.find('=') {
                let name = rest[..eq_pos].trim().to_string();
                let value = rest[eq_pos + 1..].trim().to_string();
                // Skip ARGs that are just ENV passthroughs with variable references
                if !value.starts_with("${") {
                    args.push((name, value));
                }
            }
        }
    }

    if args.is_empty() {
        return Ok(None);
    }

    println!();
    println!("{}", "Versiones disponibles para personalizar:".bold());
    for (name, value) in &args {
        println!("  {} = {}", name.dimmed(), value.cyan());
    }

    let customize = Confirm::new()
        .with_prompt("¿Quieres personalizar alguna versión?")
        .default(false)
        .interact()
        .context("Failed to display confirmation prompt")?;

    if !customize {
        return Ok(None);
    }

    let mut selections = std::collections::HashMap::new();
    let mut result = Vec::new();

    for (name, default_value) in args {
        // Apply auto-suggestion logic based on previous selections
        let suggested_default = match name.as_str() {
            "BUILD_TOOLS_VERSION" => {
                selections.get("ANDROID_API_LEVEL")
                    .map(|api| format!("{}.0.0", api))
                    .unwrap_or_else(|| default_value.clone())
            }
            _ => default_value.clone(),
        };

        let final_value = if let Some(options) = get_version_options(&name) {
            let default_idx = options
                .iter()
                .position(|o| o == &suggested_default)
                .unwrap_or_else(|| {
                    options.iter()
                        .position(|o| o == &default_value)
                        .unwrap_or(0)
                });

            let selection = Select::new()
                .with_prompt(format!("{}", name))
                .items(&options)
                .default(default_idx)
                .interact()
                .context(format!("Failed to select version for {}", name))?;

            options[selection].clone()
        } else {
            let input: String = Input::new()
                .with_prompt(format!("{} (Enter para mantener {})", name, suggested_default))
                .allow_empty(true)
                .interact_text()
                .context(format!("Failed to read custom version for {}", name))?;

            if input.trim().is_empty() {
                suggested_default
            } else {
                input.trim().to_string()
            }
        };

        selections.insert(name.clone(), final_value.clone());
        result.push((name, final_value));
    }

    Ok(Some(result))
}

fn get_version_options(arg_name: &str) -> Option<Vec<String>> {
    match arg_name {
        "ANDROID_API_LEVEL" => Some(vec![
            "33".to_string(),
            "34".to_string(),
            "35".to_string(),
            "36".to_string(),
        ]),
        "BUILD_TOOLS_VERSION" => Some(vec![
            "33.0.0".to_string(),
            "34.0.0".to_string(),
            "35.0.0".to_string(),
            "36.0.0".to_string(),
        ]),
        "NDK_VERSION" => Some(vec![
            "25.2.9519653".to_string(),
            "26.1.10909125".to_string(),
            "27.0.12077973".to_string(),
            "27.2.12479018".to_string(),
        ]),
        "KOTLIN_VERSION" => Some(vec![
            "2.0.0".to_string(),
            "2.0.10".to_string(),
            "2.0.21".to_string(),
            "2.1.0".to_string(),
            "2.1.10".to_string(),
        ]),
        "NODE_MAJOR_VERSION" => Some(vec![
            "18".to_string(),
            "20".to_string(),
            "22".to_string(),
        ]),
        "VARIANT" => Some(vec![
            "18-bullseye".to_string(),
            "20-bullseye".to_string(),
            "22-bullseye".to_string(),
        ]),
        "MAVEN_VERSION" => Some(vec![
            "3.9.6".to_string(),
            "3.9.8".to_string(),
            "3.9.9".to_string(),
        ]),
        "GO_VERSION" => Some(vec![
            "1.21.0".to_string(),
            "1.22.0".to_string(),
            "1.23.0".to_string(),
            "1.24.0".to_string(),
        ]),
        "RUST_TOOLCHAIN" => Some(vec![
            "stable".to_string(),
            "beta".to_string(),
            "nightly".to_string(),
        ]),
        "PYTHON_VERSION" => Some(vec![
            "3.11".to_string(),
            "3.12".to_string(),
            "3.13".to_string(),
        ]),
        "PHP_VERSION" => Some(vec![
            "8.2".to_string(),
            "8.3".to_string(),
            "8.4".to_string(),
        ]),
        "FLUTTER_BRANCH" => Some(vec![
            "stable".to_string(),
            "beta".to_string(),
            "master".to_string(),
        ]),
        "CMAKE_VERSION" => Some(vec![
            "3.18.1".to_string(),
            "3.22.1".to_string(),
            "3.25.2".to_string(),
            "3.31.1".to_string(),
        ]),
        "CMDLINE_TOOLS_VERSION" => Some(vec![
            "7302050_latest".to_string(),
            "10406996_latest".to_string(),
        ]),
        _ => None,
    }
}

fn apply_custom_versions(dockerfile_path: &std::path::Path, args: &[(String, String)]) -> Result<()> {
    let content = std::fs::read_to_string(dockerfile_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with("ARG ") {
            let rest = &trimmed[4..];
            if let Some(eq_pos) = rest.find('=') {
                let arg_name = rest[..eq_pos].trim();
                if let Some((_, new_value)) = args.iter().find(|(name, _)| name == arg_name) {
                    if let Some(line_eq_pos) = line.find('=') {
                        let before = &line[..line_eq_pos + 1];
                        *line = format!("{}{}", before, new_value);
                    }
                }
            }
        }
    }

    std::fs::write(dockerfile_path, lines.join("\n"))
        .context("Failed to write customized Dockerfile")?;
    Ok(())
}

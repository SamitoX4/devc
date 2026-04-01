use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Input, Select};
use crate::utils::cache::CacheManager;
use crate::utils::copier::TemplateCopier;
use crate::utils::merger::ConfigMerger;

const TEMPLATES: &[&str] = &[
    "nodejs",
    "android",
    "react-native",
    "java",
    "laravel",
    "rust",
    "go",
    "python",
];

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

    let selected_template = if let Some(t) = template {
        println!("{}", format!("  Template: {}", t).dimmed());
        t.to_string()
    } else {
        select_template_interactive()?
    };

    let project_name = if let Some(n) = name {
        println!("{}", format!("  Project: {}", n).dimmed());
        n.to_string()
    } else {
        prompt_project_name()?
    };

    let git_config = cache.get_git_config();

    let final_git_name = git_name.map(String::from)
        .or_else(|| git_config.name.clone())
        .or_else(|| {
            if git_name.is_none() && git_email.is_none() && git_config.name.is_none() {
                Some(prompt_git_name())
            } else {
                None
            }
        });

    let final_git_email = git_email.map(String::from)
        .or_else(|| git_config.email.clone())
        .or_else(|| {
            if git_name.is_none() && git_email.is_none() && git_config.email.is_none() {
                Some(prompt_git_email())
            } else {
                None
            }
        });

    if let (Some(n), Some(e)) = (&final_git_name, &final_git_email) {
        if git_name.is_some() || git_email.is_some() || git_config.name.is_none() {
            cache.save_git_config(n, e)?;
        }
    }

    println!();
    println!("{}", format!("Generating {} devcontainer...", selected_template).cyan());
    println!();

    let template_dir = TemplateCopier::find_template_dir(cache.templates_dir(), &selected_template)
        .context("Template not found. Run 'devc update' to download templates.")?;

    let current_dir = std::env::current_dir()
        .context("Could not determine current directory")?;

    TemplateCopier::copy_template(&template_dir, &current_dir)?;

    ConfigMerger::merge_template(
        &current_dir,
        &project_name,
        final_git_name.as_deref(),
        final_git_email.as_deref(),
    )?;
    ConfigMerger::update_docker_compose(&current_dir, &project_name)?;

    println!();
    println!("{}", "✓ Devcontainer generated successfully!".green());
    println!();
    println!("Next steps:");
    println!("  1. Open in VS Code: code .");
    println!("  2. Press F1 and select: Dev Containers: Reopen in Container");

    Ok(())
}

fn select_template_interactive() -> Result<String> {
    println!("{}", "Select a template:".bold());

    let selection = Select::new()
        .items(TEMPLATES)
        .default(0)
        .interact()
        .context("Failed to display template selection")?;

    let selected = TEMPLATES[selection];
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

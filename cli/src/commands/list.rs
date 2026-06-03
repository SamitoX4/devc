use anyhow::Result;
use colored::*;
use crate::utils::cache::CacheManager;

pub fn run(cache: &CacheManager, detailed: bool) -> Result<()> {
    let templates = cache.get_available_templates();

    if templates.is_empty() {
        println!("No templates found. Run 'devc update' to download templates.");
        return Ok(());
    }

    println!();
    println!("{}", "Available Templates:".bold());
    println!("{}", "==================".bold());
    println!();

    for template in &templates {
        println!("  {}", format!("• {}", template).cyan());
        
        if detailed {
            let desc = get_template_description(cache, template);
            println!("    {}", desc.dimmed());
            println!();
        }
    }

    println!();
    println!("Run 'devc gen <template>' to generate a devcontainer.");
    println!("Example: devc gen nodejs --name my-project");

    Ok(())
}

fn get_template_description(cache: &CacheManager, template: &str) -> String {
    let templates_dir = cache.templates_dir();
    let template_path = templates_dir.join(template).join(".devcontainer").join("devcontainer.json");
    
    if let Ok(content) = std::fs::read_to_string(&template_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                return name.to_string();
            }
        }
    }
    
    format!("Development container template ({})", template)
}

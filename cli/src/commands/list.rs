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
            let desc = get_template_description(template);
            println!("    {}", desc.dimmed());
            println!();
        }
    }

    println!();
    println!("Run 'devc gen <template>' to generate a devcontainer.");
    println!("Example: devc gen nodejs --name my-project");

    Ok(())
}

fn get_template_description(template: &str) -> &str {
    match template {
        "nodejs" => "Node.js with TypeScript, npm/pnpm, and common tools",
        "android" => "Java 17 + Android SDK for Android development",
        "react-native" => "Node.js + React Native + Android SDK for mobile apps",
        "java" => "Java 17 + Maven for Java/Kotlin development",
        "laravel" => "PHP 8.3 + Composer for Laravel development",
        "rust" => "Rust (stable) + Cargo for Rust development",
        "go" => "Go 1.22 for Go development",
        "python" => "Python 3.12 with pip and virtualenv",
        _ => "Development container template",
    }
}

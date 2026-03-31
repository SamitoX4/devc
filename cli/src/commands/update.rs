use anyhow::Result;
use colored::*;
use crate::utils::cache::CacheManager;

pub async fn run(cache: &CacheManager, force: bool) -> Result<()> {
    println!("{}", "Checking for template updates...".bold());

    match cache.download_templates(force, true).await {
        Ok(_) => {
            println!();
            println!("{}", "✓ Templates updated successfully!".green());
            println!();
            println!("Run 'devc list' to see available templates.");
        }
        Err(e) => {
            eprintln!();
            eprintln!("{}", format!("✗ Failed to update templates: {}", e).red());
            std::process::exit(1);
        }
    }

    Ok(())
}

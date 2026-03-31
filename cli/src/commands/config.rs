use anyhow::Result;
use colored::*;
use crate::utils::cache::CacheManager;

pub fn run(
    cache: &mut CacheManager,
    git_name: Option<&str>,
    git_email: Option<&str>,
    show: bool,
) -> Result<()> {
    if show {
        let config = cache.get_git_config();
        println!();
        println!("{}", "Git Configuration:".bold());
        println!("{}", "==================".bold());
        println!();
        if let (Some(name), Some(email)) = (&config.name, &config.email) {
            println!("  {}", format!("Git User Name:  {}", name).green());
            println!("  {}", format!("Git User Email: {}", email).green());
        } else {
            println!("  {}", "Not configured".yellow());
            println!();
            println!("  Run: devc config --git-name \"Your Name\" --git-email \"your@email.com\"");
        }
        println!();
        return Ok(());
    }

    let mut name = git_name.map(String::from);
    let mut email = git_email.map(String::from);

    let current_config = cache.get_git_config();

    if name.is_none() && email.is_none() && current_config.name.is_none() {
        println!();
        println!("{}", "Git Configuration".bold());
        println!("{}", "=================".bold());
        println!();
        
        print!("{} ", "Git User Name:".cyan());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input_name = input.trim();
        
        if input_name.is_empty() {
            println!("{}", "Git user name is required".red());
            return Ok(());
        }
        name = Some(input_name.to_string());

        print!("{} ", "Git User Email:".cyan());
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        let input_email = input.trim();
        
        if input_email.is_empty() {
            println!("{}", "Git user email is required".red());
            return Ok(());
        }
        email = Some(input_email.to_string());
    }

    if name.is_none() {
        name = current_config.name.clone();
    }
    if email.is_none() {
        email = current_config.email.clone();
    }

    if let (Some(n), Some(e)) = (&name, &email) {
        cache.save_git_config(n, e)?;
        println!();
        println!("{}", "✓ Git configuration saved successfully!".green());
        println!();
        println!("  {} {}", "Name:".dimmed(), n);
        println!("  {} {}", "Email:".dimmed(), e);
        println!();
    }

    Ok(())
}

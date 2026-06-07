use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};
use crate::utils::cache::CacheManager;
use crate::utils::copier::TemplateCopier;
use crate::utils::credentials;
use crate::utils::merger::ConfigMerger;
use crate::utils::password;
use crate::utils::tui::Tui;
use crate::utils::SecurityConfig;

pub async fn run(
    template: Option<&str>,
    name: Option<&str>,
    git_name: Option<&str>,
    git_email: Option<&str>,
    security_mode: Option<&str>,
    remote_user: Option<&str>,
    remote_password: Option<&str>,
    container_password: Option<&str>,
    sudo_mode: Option<&str>,
    network_mode: Option<&str>,
    save_credentials_flag: Option<&str>,
    cache: &mut CacheManager,
) -> Result<()> {
    let tui = Tui::new("DevContainer Generator");

    // 1. Template
    let selected_template = if let Some(t) = template {
        println!("{}", format!("  Template: {}", t).dimmed());
        t.to_string()
    } else {
        tui.draw_frame("Selección de template", None)?;
        select_template_interactive(cache, &tui)?
    };

    // 2. Project Name
    let project_name = if let Some(n) = name {
        println!("{}", format!("  Project: {}", n).dimmed());
        n.to_string()
    } else {
        tui.draw_frame("Nombre del proyecto", Some(&selected_template))?;
        prompt_project_name(&tui)?
    };

    // 3. Target Directory
    tui.draw_frame("Ubicación del devcontainer", Some(&selected_template))?;
    let target_dir = prompt_target_directory(&tui)?;

    // 4. Git Config
    let git_config = cache.get_git_config();

    let final_git_name = if let Some(n) = git_name {
        println!("{}", format!("  Git Name: {}", n).dimmed());
        Some(n.to_string())
    } else if let Some(n) = git_config.name.as_ref() {
        println!("{}", format!("  Git Name: {} (from config)", n).dimmed());
        Some(n.clone())
    } else {
        tui.draw_frame("Configuración de Git", Some(&selected_template))?;
        Some(prompt_git_name(&tui))
    };

    let final_git_email = if let Some(e) = git_email {
        println!("{}", format!("  Git Email: {}", e).dimmed());
        Some(e.to_string())
    } else if let Some(e) = git_config.email.as_ref() {
        println!("{}", format!("  Git Email: {} (from config)", e).dimmed());
        Some(e.clone())
    } else {
        tui.draw_frame("Configuración de Git", Some(&selected_template))?;
        Some(prompt_git_email(&tui))
    };

    if let (Some(n), Some(e)) = (&final_git_name, &final_git_email) {
        cache.save_git_config(n, e)?;
    }

    // 5. Security Config
    tui.draw_frame("Configuración de seguridad", Some(&selected_template))?;
    let security = build_security_config(
        &selected_template,
        security_mode,
        remote_user,
        remote_password,
        container_password,
        sudo_mode,
        network_mode,
        &tui,
    )?;

    tui.cleanup()?;

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
        &security,
    )?;
    ConfigMerger::update_docker_compose(&target_dir, &project_name, &security)?;

    let dockerfile_path = target_dir.join(".devcontainer").join("Dockerfile");
    if dockerfile_path.exists() {
        if let Some(custom_args) = prompt_custom_versions(&dockerfile_path, &selected_template, &tui)? {
            apply_custom_versions(&dockerfile_path, &custom_args)?;
            apply_custom_versions_to_config_files(&target_dir, &custom_args)?;
            println!();
            println!("{}", "✓ Versiones personalizadas aplicadas".green());
        }
    }

    // Show generated passwords if auto-generated
    if remote_password.is_none() || container_password.is_none() {
        println!();
        println!("{}", "🔐 Credenciales del contenedor:".bold().yellow());
        if security.mode != "root" {
            println!("  {}: {}", "Usuario de desarrollo".dimmed(), security.remote_user.cyan());
            println!("  {}: {}", "Contraseña de desarrollo".dimmed(), security.remote_password.cyan());
        }
        println!("  {}: {}", "Contraseña de root".dimmed(), security.container_password.cyan());
        println!("{}", "  (Guárdalas, no se vuelven a mostrar)".dimmed());

        // Offer to save credentials
        if let Some(saved_path) = maybe_save_credentials(
            &project_name,
            &selected_template,
            &security,
            save_credentials_flag,
            &tui,
        )? {
            println!();
            println!("{}", format!("✓ Credenciales guardadas en: {}", saved_path.display()).green());
            println!("{}", "  (Permisos: solo lectura para el dueño)".dimmed());
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

fn get_security_defaults(template: &str) -> SecurityConfig {
    if template.starts_with("android/") {
        SecurityConfig {
            mode: "root".to_string(),
            remote_user: "root".to_string(),
            container_user: None,
            remote_password: password::generate_12(),
            container_password: password::generate_12(),
            sudo_mode: "none".to_string(),
            network_mode: "bridge".to_string(),
            network_name: None,
        }
    } else if template == "nodejs" || template == "android/react-native" {
        SecurityConfig {
            mode: "developer".to_string(),
            remote_user: "node".to_string(),
            container_user: Some("node".to_string()),
            remote_password: password::generate_12(),
            container_password: password::generate_12(),
            sudo_mode: "nopasswd".to_string(),
            network_mode: "bridge".to_string(),
            network_name: None,
        }
    } else {
        SecurityConfig {
            mode: "developer".to_string(),
            remote_user: "developer".to_string(),
            container_user: Some("developer".to_string()),
            remote_password: password::generate_12(),
            container_password: password::generate_12(),
            sudo_mode: "nopasswd".to_string(),
            network_mode: "bridge".to_string(),
            network_name: None,
        }
    }
}

fn build_security_config(
    template: &str,
    mode_flag: Option<&str>,
    user_flag: Option<&str>,
    remote_pass_flag: Option<&str>,
    container_pass_flag: Option<&str>,
    sudo_flag: Option<&str>,
    network_mode_flag: Option<&str>,
    tui: &Tui,
) -> Result<SecurityConfig> {
    let defaults = get_security_defaults(template);

    // Network mode (used in both interactive and non-interactive)
    let (network_mode, network_name) = if let Some(nm) = network_mode_flag {
        (nm.to_lowercase(), None)
    } else {
        prompt_network_mode(&defaults.network_mode, template, tui)?
    };

    // If all flags are provided, use them directly (non-interactive mode)
    if let Some(mode) = mode_flag {
        let mode = mode.to_lowercase();
        let remote_user = user_flag.map(|s| s.to_string()).unwrap_or_else(|| {
            if mode == "root" {
                "root".to_string()
            } else {
                defaults.remote_user.clone()
            }
        });
        let container_user = if mode == "root" {
            None
        } else {
            Some(remote_user.clone())
        };
        let remote_password = remote_pass_flag.map(|s| s.to_string()).unwrap_or_else(|| password::generate_12());
        let container_password = container_pass_flag.map(|s| s.to_string()).unwrap_or_else(|| password::generate_12());
        let sudo_mode = sudo_flag.map(|s| s.to_lowercase()).unwrap_or_else(|| {
            match mode.as_str() {
                "developer" => "nopasswd".to_string(),
                "secure" => "none".to_string(),
                "root" => "none".to_string(),
                _ => defaults.sudo_mode.clone(),
            }
        });

        return Ok(SecurityConfig {
            mode,
            remote_user,
            container_user,
            remote_password,
            container_password,
            sudo_mode,
            network_mode,
            network_name,
        });
    }

    // Interactive mode
    let mode = prompt_security_mode(&defaults.mode, template, tui)?;

    if mode == "root" {
        let container_password = if let Some(pass) = container_pass_flag {
            pass.to_string()
        } else {
            tui.draw_frame("Contraseña de root", Some(template))?;
            if let Some(ctx) = crate::utils::tui::get_step_context("Contraseña de root") {
                tui.print_context(&ctx)?;
            }
            prompt_password("root del contenedor")?
        };

        return Ok(SecurityConfig {
            mode,
            remote_user: "root".to_string(),
            container_user: None,
            remote_password: container_password.clone(),
            container_password,
            sudo_mode: "none".to_string(),
            network_mode,
            network_name,
        });
    }

    let remote_user = if let Some(user) = user_flag {
        user.to_string()
    } else {
        prompt_remote_user(&defaults.remote_user, template, tui)?
    };

    let (remote_password, container_password) = if remote_pass_flag.is_some() && container_pass_flag.is_some() {
        (remote_pass_flag.unwrap().to_string(), container_pass_flag.unwrap().to_string())
    } else {
        prompt_passwords(&remote_user, template, tui)?
    };

    let sudo_mode = if let Some(sudo) = sudo_flag {
        sudo.to_lowercase()
    } else {
        let default_sudo = match mode.as_str() {
            "developer" => "nopasswd",
            "secure" => "none",
            _ => &defaults.sudo_mode,
        };
        prompt_sudo_mode(default_sudo, template, tui)?
    };

    let container_user = Some(remote_user.clone());

    Ok(SecurityConfig {
        mode,
        remote_user,
        container_user,
        remote_password,
        container_password,
        sudo_mode,
        network_mode,
        network_name,
    })
}

fn prompt_security_mode(default: &str, template: &str, tui: &Tui) -> Result<String> {
    let options = vec![
        "Modo Desarrollador (recomendado) — usuario con sudo sin contraseña",
        "Modo Seguro — usuario sin sudo",
        "Modo Root — todo como root",
        "Personalizado — configurar manualmente",
    ];

    let default_idx = match default {
        "developer" => 0,
        "secure" => 1,
        "root" => 2,
        _ => 0,
    };

    loop {
        tui.draw_frame("Configuración de seguridad", Some(template))?;
        println!("{}", "Configuración de Seguridad del Contenedor".bold());
        println!("{}", "===========================================".bold());
        println!();

        let mut items = options.clone();
        items.push("❓  Ayuda");

        let selection = Select::new()
            .with_prompt("Elige el perfil de seguridad")
            .items(&items)
            .default(default_idx)
            .interact()
            .context("Failed to display security mode selection")?;

        if selection == options.len() {
            tui.show_help_box("Configuración de seguridad")?;
            continue;
        }

        let mode = match selection {
            0 => "developer",
            1 => "secure",
            2 => "root",
            3 => "custom",
            _ => "developer",
        };

        println!("{}", format!("  ✓ Modo: {}", mode).green());
        return Ok(mode.to_string());
    }
}

fn prompt_remote_user(default: &str, template: &str, tui: &Tui) -> Result<String> {
    loop {
        tui.draw_frame("Usuario de desarrollo", Some(template))?;
        if let Some(ctx) = crate::utils::tui::get_step_context("Usuario de desarrollo") {
            tui.print_context(&ctx)?;
        }
        println!("  {} {}", "Tip:".italic().dimmed(), "Escribí ?help y presioná Enter para ver ayuda detallada".italic().dimmed());

        let input: String = Input::new()
            .with_prompt("Nombre de usuario de desarrollo")
            .default(default.to_string())
            .interact_text()
            .context("Failed to read remote user name")?;

        if input.trim() == "?help" {
            tui.show_help_box("Usuario de desarrollo")?;
            continue;
        }

        println!("{}", format!("  ✓ Usuario: {}", input).green());
        return Ok(input);
    }
}

fn prompt_passwords(remote_user: &str, template: &str, tui: &Tui) -> Result<(String, String)> {
    tui.draw_frame("Contraseñas del contenedor", Some(template))?;
    if let Some(ctx) = crate::utils::tui::get_step_context("Contraseñas del contenedor") {
        tui.print_context(&ctx)?;
    }

    let use_custom = Confirm::new()
        .with_prompt("¿Configurar contraseñas personalizadas?")
        .default(false)
        .interact()
        .context("Failed to display password confirmation")?;

    if use_custom {
        tui.draw_frame("Contraseña de root", Some(template))?;
        if let Some(ctx) = crate::utils::tui::get_step_context("Contraseña de root") {
            tui.print_context(&ctx)?;
        }
        let container_password = prompt_password("root del contenedor")?;
        let step = format!("Contraseña de {}", remote_user);
        tui.draw_frame(&step, Some(template))?;
        if let Some(ctx) = crate::utils::tui::get_step_context("Contraseña de root") {
            tui.print_context(&ctx)?;
        }
        let remote_password = prompt_password(remote_user)?;
        Ok((remote_password, container_password))
    } else {
        let remote_password = password::generate_12();
        let container_password = password::generate_12();
        println!("{}", "  ✓ Contraseñas auto-generadas".green());
        Ok((remote_password, container_password))
    }
}

fn prompt_password(for_user: &str) -> Result<String> {
    loop {
        let pass = match Password::new()
            .with_prompt(format!("Contraseña para {} (Enter para auto-generar)", for_user))
            .allow_empty_password(true)
            .interact()
        {
            Ok(p) => p,
            Err(_) => anyhow::bail!("Prompt cancelado por el usuario"),
        };

        if pass.is_empty() {
            let generated = password::generate_12();
            println!("{}", format!("  ✓ Contraseña auto-generada para {}: {}", for_user, generated.cyan()).green());
            return Ok(generated);
        }

        if pass.len() < 6 {
            println!("{}", "  ✗ La contraseña debe tener al menos 6 caracteres. Intentá de nuevo.".red());
            continue;
        }

        let confirm = match Password::new()
            .with_prompt(format!("Confirmar contraseña para {}", for_user))
            .allow_empty_password(true)
            .interact()
        {
            Ok(p) => p,
            Err(_) => anyhow::bail!("Prompt cancelado por el usuario"),
        };

        if pass != confirm {
            println!("{}", "  ✗ Las contraseñas no coinciden. Intentá de nuevo.".red());
            continue;
        }

        return Ok(pass);
    }
}

fn prompt_sudo_mode(default: &str, template: &str, tui: &Tui) -> Result<String> {
    let options = vec![
        "sudo sin contraseña (NOPASSWD:ALL)",
        "sudo con contraseña",
        "sin sudo",
    ];

    let default_idx = match default {
        "nopasswd" => 0,
        "password" => 1,
        "none" => 2,
        _ => 0,
    };

    loop {
        tui.draw_frame("Privilegios sudo", Some(template))?;

        let mut items = options.clone();
        items.push("❓  Ayuda");

        let selection = Select::new()
            .with_prompt("Modo sudo para el usuario de desarrollo")
            .items(&items)
            .default(default_idx)
            .interact()
            .context("Failed to display sudo mode selection")?;

        if selection == options.len() {
            tui.show_help_box("Privilegios sudo")?;
            continue;
        }

        let mode = match selection {
            0 => "nopasswd",
            1 => "password",
            2 => "none",
            _ => "nopasswd",
        };

        println!("{}", format!("  ✓ Sudo: {}", mode).green());
        return Ok(mode.to_string());
    }
}

fn prompt_network_mode(default: &str, template: &str, tui: &Tui) -> Result<(String, Option<String>)> {
    let options = vec![
        "Bridge (recomendado) — aislamiento de red con port mapping",
        "Host — máximo rendimiento, sin aislamiento (solo Linux)",
        "None — contenedor sin acceso a red",
    ];

    let default_idx = match default {
        "bridge" => 0,
        "host" => 1,
        "none" => 2,
        _ => 0,
    };

    let mode = loop {
        tui.draw_frame("Configuración de red", Some(template))?;
        println!("{}", "Modo de red del contenedor".bold());
        println!("{}", "==========================".bold());
        println!();

        let mut items = options.clone();
        items.push("❓  Ayuda");

        let selection = Select::new()
            .with_prompt("Elige el modo de red")
            .items(&items)
            .default(default_idx)
            .interact()
            .context("Failed to display network mode selection")?;

        if selection == options.len() {
            tui.show_help_box("Configuración de red")?;
            continue;
        }

        let mode = match selection {
            0 => "bridge",
            1 => "host",
            2 => "none",
            _ => "bridge",
        };

        println!("{}", format!("  ✓ Red: {}", mode).green());
        break mode;
    };

    // If bridge, ask about external shared network
    let network_name = if mode == "bridge" {
        tui.draw_frame("Red compartida", Some(template))?;
        let use_shared = Confirm::new()
            .with_prompt("¿Querés unir este devcontainer a una red externa compartida? (útil para conectar con servicios en otros docker-compose)")
            .default(false)
            .interact()
            .context("Failed to display shared network confirmation")?;

        if use_shared {
            let input: String = Input::new()
                .with_prompt("Nombre de la red compartida")
                .default("shared_net".to_string())
                .interact_text()
                .context("Failed to read shared network name")?;
            Some(input.trim().to_string())
        } else {
            None
        }
    } else {
        None
    };

    Ok((mode.to_string(), network_name))
}

fn maybe_save_credentials(
    project_name: &str,
    template: &str,
    security: &SecurityConfig,
    save_credentials_flag: Option<&str>,
    tui: &Tui,
) -> Result<Option<std::path::PathBuf>> {
    let should_save = if let Some(flag) = save_credentials_flag {
        // Non-empty flag means yes (value is the path or "default")
        !flag.is_empty()
    } else {
        // Interactive prompt
        tui.draw_frame("Guardar credenciales", Some(template))?;
        if let Some(ctx) = crate::utils::tui::get_step_context("Guardar credenciales") {
            tui.print_context(&ctx)?;
        }
        Confirm::new()
            .with_prompt("¿Guardar las credenciales en un archivo?")
            .default(false)
            .interact()
            .context("Failed to display save credentials confirmation")?
    };

    if !should_save {
        return Ok(None);
    }

    let path = if let Some(flag) = save_credentials_flag {
        if flag == "default" {
            credentials::default_credentials_path(project_name)
        } else {
            std::path::PathBuf::from(flag)
        }
    } else {
        tui.draw_frame("Ruta del archivo de credenciales", Some(template))?;
        let default = credentials::default_credentials_path(project_name);
        let input: String = Input::new()
            .with_prompt("Ruta del archivo de credenciales")
            .default(default.to_string_lossy().to_string())
            .interact_text()
            .context("Failed to read credentials file path")?;
        std::path::PathBuf::from(input.trim())
    };

    credentials::save_credentials(&path, project_name, template, security)
        .context("Failed to save credentials file")?;

    Ok(Some(path))
}

fn select_template_interactive(cache: &CacheManager, tui: &Tui) -> Result<String> {
    let templates = cache.get_available_templates();

    if templates.is_empty() {
        anyhow::bail!("No templates found. Run 'devc update' to download templates.");
    }

    loop {
        let mut items: Vec<String> = templates.clone();
        items.push("❓  Ayuda — ¿Qué es un template?".to_string());

        println!("{}", "Select a template:".bold());

        let selection = Select::new()
            .items(&items)
            .default(0)
            .interact()
            .context("Failed to display template selection")?;

        if selection == templates.len() {
            tui.show_help_box("Selección de template")?;
            tui.draw_frame("Selección de template", None)?;
            continue;
        }

        let selected = &templates[selection];
        println!("{}", format!("  ✓ {}", selected).green());
        return Ok(selected.to_string());
    }
}

fn prompt_project_name(tui: &Tui) -> Result<String> {
    if let Some(ctx) = crate::utils::tui::get_step_context("Nombre del proyecto") {
        tui.print_context(&ctx)?;
    }

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

fn prompt_target_directory(tui: &Tui) -> Result<std::path::PathBuf> {
    if let Some(ctx) = crate::utils::tui::get_step_context("Ubicación del devcontainer") {
        tui.print_context(&ctx)?;
    }

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

fn prompt_git_name(tui: &Tui) -> String {
    if let Some(ctx) = crate::utils::tui::get_step_context("Configuración de Git") {
        let _ = tui.print_context(&ctx);
    }
    let input: String = Input::new()
        .with_prompt("Git User Name")
        .default("user".to_string())
        .interact_text()
        .unwrap_or_else(|_| "user".to_string());

    input
}

fn prompt_git_email(tui: &Tui) -> String {
    if let Some(ctx) = crate::utils::tui::get_step_context("Configuración de Git") {
        let _ = tui.print_context(&ctx);
    }
    let input: String = Input::new()
        .with_prompt("Git User Email")
        .default("user@example.com".to_string())
        .interact_text()
        .unwrap_or_else(|_| "user@example.com".to_string());

    input
}

fn prompt_custom_versions(
    dockerfile_path: &std::path::Path,
    template: &str,
    tui: &Tui,
) -> Result<Option<Vec<(String, String)>>> {
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

    tui.draw_frame("Personalizar versiones", Some(template))?;
    if let Some(ctx) = crate::utils::tui::get_step_context("Personalizar versiones") {
        tui.print_context(&ctx)?;
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

        let final_value = if name == "NODE_MAJOR_VERSION" {
            if let Some(options) = get_version_options(&name) {
                let mut defaults = vec![false; options.len()];
                if let Some(idx) = options.iter().position(|o| o == &suggested_default) {
                    defaults[idx] = true;
                }

                let step = format!("Versión de {}", name);
                tui.draw_frame(&step, Some(template))?;
                let picked = MultiSelect::new()
                    .with_prompt(format!("{} (space to select, enter to confirm)", name))
                    .items(&options)
                    .defaults(&defaults)
                    .interact()
                    .context(format!("Failed to select versions for {}", name))?;

                if picked.is_empty() {
                    suggested_default
                } else {
                    picked.iter().map(|&i: &usize| options[i].clone()).collect::<Vec<String>>().join(",")
                }
            } else {
                suggested_default
            }
        } else if let Some(options) = get_version_options(&name) {
            let default_idx = options
                .iter()
                .position(|o| o == &suggested_default)
                .unwrap_or_else(|| {
                    options.iter()
                        .position(|o| o == &default_value)
                        .unwrap_or(0)
                });

            let step = format!("Versión de {}", name);
            tui.draw_frame(&step, Some(template))?;
            let selection = Select::new()
                .with_prompt(format!("{}", name))
                .items(&options)
                .default(default_idx)
                .interact()
                .context(format!("Failed to select version for {}", name))?;

            options[selection].clone()
        } else {
            let step = format!("Versión de {}", name);
            tui.draw_frame(&step, Some(template))?;
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
        "JAVA_VERSION" => Some(vec![
            "17".to_string(),
            "21".to_string(),
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

fn apply_custom_versions_to_config_files(
    target_dir: &std::path::Path,
    args: &[(String, String)],
) -> Result<()> {
    let devcontainer_json_path = target_dir.join(".devcontainer").join("devcontainer.json");
    if !devcontainer_json_path.exists() {
        return Ok(());
    }

    let mut content = std::fs::read_to_string(&devcontainer_json_path)?;
    let mut modified = false;

    for (name, value) in args {
        if name == "JAVA_VERSION" {
            content = content.replace("java-17-openjdk-amd64", &format!("java-{}-openjdk-amd64", value));
            content = content.replace("JavaSE-17", &format!("JavaSE-{}", value));
            modified = true;
        }
    }

    if modified {
        std::fs::write(&devcontainer_json_path, content)
            .context("Failed to write customized devcontainer.json")?;
    }

    Ok(())
}

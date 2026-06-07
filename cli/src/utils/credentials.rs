use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::utils::SecurityConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialsFile {
    pub project: String,
    pub template: String,
    pub mode: String,
    pub remote_user: String,
    pub remote_password: String,
    pub container_password: String,
}

pub fn save_credentials(
    path: &Path,
    project_name: &str,
    template: &str,
    security: &SecurityConfig,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create credentials directory: {}", parent.display()))?;
    }

    let creds = CredentialsFile {
        project: project_name.to_string(),
        template: template.to_string(),
        mode: security.mode.clone(),
        remote_user: security.remote_user.clone(),
        remote_password: security.remote_password.clone(),
        container_password: security.container_password.clone(),
    };

    let content = serde_json::to_string_pretty(&creds)
        .context("Failed to serialize credentials")?;

    fs::write(path, content)
        .context(format!("Failed to write credentials file: {}", path.display()))?;

    // Set restrictive permissions on Unix: owner read/write only (600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions)
            .context("Failed to set credentials file permissions")?;
    }

    Ok(())
}

pub fn default_credentials_path(project_name: &str) -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".devc")
        .join("credentials")
        .join(format!("{}.json", project_name.replace(|c: char| !c.is_alphanumeric(), "_")))
}

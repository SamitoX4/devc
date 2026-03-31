use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct ConfigMerger;

impl ConfigMerger {
    pub fn merge_template(
        project_dir: &Path,
        project_name: &str,
        git_name: Option<&str>,
        git_email: Option<&str>,
    ) -> Result<()> {
        let devcontainer_dir = project_dir.join(".devcontainer");

        if !devcontainer_dir.exists() {
            anyhow::bail!("Invalid template: .devcontainer folder not found");
        }

        Self::update_devcontainer_json(
            &devcontainer_dir,
            project_name,
            git_name,
            git_email,
        )?;

        Ok(())
    }

    fn update_devcontainer_json(
        devcontainer_dir: &Path,
        project_name: &str,
        git_name: Option<&str>,
        git_email: Option<&str>,
    ) -> Result<()> {
        let json_path = devcontainer_dir.join("devcontainer.json");
        
        if !json_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&json_path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse devcontainer.json")?;

        if let Some(obj) = json.as_object_mut() {
            if let Some(name) = obj.get_mut("name") {
                *name = serde_json::Value::String(format!("{} DevContainer", project_name));
            }

            if let Some(container_env) = obj.get_mut("containerEnv") {
                if let Some(env) = container_env.as_object_mut() {
                    if let (Some(name), Some(email)) = (git_name, git_email) {
                        env.insert(
                            "GIT_USER_NAME".to_string(),
                            serde_json::Value::String(name.to_string()),
                        );
                        env.insert(
                            "GIT_USER_EMAIL".to_string(),
                            serde_json::Value::String(email.to_string()),
                        );
                    }
                }
            } else if git_name.is_some() && git_email.is_some() {
                let mut env = serde_json::Map::new();
                env.insert("GIT_USER_NAME".to_string(), serde_json::Value::String(git_name.unwrap().to_string()));
                env.insert("GIT_USER_EMAIL".to_string(), serde_json::Value::String(git_email.unwrap().to_string()));
                obj.insert("containerEnv".to_string(), serde_json::Value::Object(env));
            }
        }

        let output = serde_json::to_string_pretty(&json)?;
        fs::write(&json_path, output)?;

        Ok(())
    }

    pub fn update_docker_compose(devcontainer_dir: &Path, project_name: &str) -> Result<()> {
        let compose_path = devcontainer_dir.join("docker-compose.yml");
        
        if !compose_path.exists() {
            return Ok(());
        }

        let mut content = fs::read_to_string(&compose_path)?;

        content = content.replace(
            "testing_devcontainer",
            &project_name.replace("-", "_").to_lowercase(),
        );

        fs::write(&compose_path, content)?;
        Ok(())
    }
}

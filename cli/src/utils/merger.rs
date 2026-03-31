use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::utils::field_order::FieldOrder;

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

        Self::apply_modifications(&mut json, project_name, git_name, git_email)?;
        
        let output = Self::serialize_ordered(&json)?;
        fs::write(&json_path, output)?;

        Ok(())
    }

    fn apply_modifications(
        json: &mut serde_json::Value,
        project_name: &str,
        git_name: Option<&str>,
        git_email: Option<&str>,
    ) -> Result<()> {
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

        Ok(())
    }

    fn serialize_ordered(value: &serde_json::Value) -> Result<String> {
        let order = FieldOrder::get_order();
        let mut output = String::new();
        
        if let Some(obj) = value.as_object() {
            output.push_str("{\n");
            
            let mut key_list: Vec<&String> = Vec::new();
            
            for field in order {
                if let Some(key) = obj.keys().find(|k| k.as_str() == *field) {
                    key_list.push(key);
                }
            }
            
            for key in obj.keys() {
                if !order.contains(&key.as_str()) {
                    key_list.push(key);
                }
            }
            
            for (i, key) in key_list.iter().enumerate() {
                if i > 0 {
                    output.push_str(",\n");
                }
                output.push_str(&format!("  \"{key}\": "));
                Self::write_value(&mut output, obj.get(*key).unwrap(), 1)?;
            }
            
            output.push_str("\n}\n");
        } else {
            Self::write_value(&mut output, value, 0)?;
            output.push('\n');
        }
        
        Ok(output)
    }

    fn write_value(output: &mut String, value: &serde_json::Value, indent: usize) -> Result<()> {
        let spaces = "  ".repeat(indent);
        
        match value {
            serde_json::Value::Object(map) => {
                if map.is_empty() {
                    output.push_str("{}\n");
                } else {
                    output.push_str("{\n");
                    let entries: Vec<_> = map.iter().collect();
                    for (i, (key, val)) in entries.iter().enumerate() {
                        if i > 0 {
                            output.push_str(",\n");
                        }
                        output.push_str(&format!("{spaces}  \"{key}\": "));
                        Self::write_value(output, val, indent + 1)?;
                    }
                    output.push_str(&format!("\n{spaces}}}"));
                }
            }
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    output.push_str("[]\n");
                } else {
                    output.push_str("[\n");
                    for (i, item) in arr.iter().enumerate() {
                        if i > 0 {
                            output.push_str(",\n");
                        }
                        output.push_str(&format!("{spaces}  "));
                        Self::write_value(output, item, indent + 1)?;
                    }
                    output.push_str(&format!("\n{spaces}]"));
                }
            }
            serde_json::Value::String(s) => {
                output.push('"');
                output.push_str(&Self::escape_string(s));
                output.push_str("\"");
            }
            serde_json::Value::Number(n) => {
                output.push_str(&n.to_string());
            }
            serde_json::Value::Bool(b) => {
                output.push_str(if *b { "true" } else { "false" });
            }
            serde_json::Value::Null => {
                output.push_str("null");
            }
        }
        Ok(())
    }

    fn escape_string(s: &str) -> String {
        let mut result = String::new();
        for ch in s.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                c if c.is_control() => {
                    result.push('\\');
                    result.push('u');
                    result.push('{');
                    result.push_str(&format!("{:04x}", c as u32));
                    result.push('}');
                }
                c => result.push(c),
            }
        }
        result
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

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
        let json: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse devcontainer.json")?;

        let modified = Self::apply_modifications(json, project_name, git_name, git_email)?;
        let ordered = Self::reorder_fields(modified);
        let output = Self::serialize_ordered(&ordered)?;
        fs::write(&json_path, output)?;

        Ok(())
    }

    fn apply_modifications(
        mut json: serde_json::Value,
        project_name: &str,
        git_name: Option<&str>,
        git_email: Option<&str>,
    ) -> Result<serde_json::Value> {
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

        Ok(json)
    }

    fn reorder_fields(json: serde_json::Value) -> serde_json::Value {
        let order = FieldOrder::get_order();
        let mut ordered_map = serde_json::Map::new();

        for field in order {
            if let Some(obj) = json.as_object() {
                if let Some(value) = obj.get(field) {
                    ordered_map.insert(field.to_string(), value.clone());
                }
            }
        }

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                if !ordered_map.contains_key(key) {
                    ordered_map.insert(key.clone(), value.clone());
                }
            }
        }

        serde_json::Value::Object(ordered_map)
    }

    fn serialize_ordered(value: &serde_json::Value) -> Result<String> {
        let mut output = String::new();
        Self::write_json(&mut output, value, 0)?;
        Ok(output)
    }

    fn write_json(output: &mut String, value: &serde_json::Value, indent: usize) -> Result<()> {
        let spaces = "  ".repeat(indent);
        
        match value {
            serde_json::Value::Object(map) => {
                if map.is_empty() {
                    output.push_str("{}\n");
                } else {
                    output.push_str("{\n");
                    let entries: Vec<_> = map.iter().collect();
                    for (i, (key, val)) in entries.iter().enumerate() {
                        output.push_str(&format!("{spaces}  \"{key}\": "));
                        Self::write_json_value(output, val, indent + 1)?;
                        if i < entries.len() - 1 {
                            output.push(',');
                        }
                        output.push('\n');
                    }
                    output.push_str(&format!("{spaces}}}"));
                    if indent == 0 {
                        output.push('\n');
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                if arr.is_empty() {
                    output.push_str("[]\n");
                } else {
                    output.push_str("[\n");
                    for (i, item) in arr.iter().enumerate() {
                        output.push_str(&format!("{spaces}  "));
                        Self::write_json_value(output, item, indent + 1)?;
                        if i < arr.len() - 1 {
                            output.push(',');
                        }
                        output.push('\n');
                    }
                    output.push_str(&format!("{spaces}]"));
                    if indent == 0 {
                        output.push('\n');
                    }
                }
            }
            _ => Self::write_json_value(output, value, indent)?,
        }
        Ok(())
    }

    fn write_json_value(output: &mut String, value: &serde_json::Value, _indent: usize) -> Result<()> {
        match value {
            serde_json::Value::String(s) => {
                output.push('"');
                output.push_str(&Self::escape_string(s));
                output.push('"');
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
            serde_json::Value::Array(arr) => {
                output.push_str("[\n");
                for (i, item) in arr.iter().enumerate() {
                    output.push_str(&format!("{}", "  ".repeat(_indent + 1)));
                    Self::write_json_value(output, item, _indent + 1)?;
                    if i < arr.len() - 1 {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&format!("{}]", "  ".repeat(_indent)));
            }
            serde_json::Value::Object(map) => {
                output.push_str("{\n");
                let entries: Vec<_> = map.iter().collect();
                for (i, (key, val)) in entries.iter().enumerate() {
                    output.push_str(&format!("{0}", "  ".repeat(_indent + 1)));
                    output.push('"');
                    output.push_str(&Self::escape_string(key));
                    output.push_str("\": ");
                    Self::write_json_value(output, val, _indent + 1)?;
                    if i < entries.len() - 1 {
                        output.push(',');
                    }
                    output.push('\n');
                }
                output.push_str(&format!("{}}}", "  ".repeat(_indent)));
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

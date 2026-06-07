pub mod cache;
pub mod copier;
pub mod credentials;
pub mod fetcher;
pub mod field_order;
pub mod merger;
pub mod password;

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub mode: String,           // developer | secure | root | custom
    pub remote_user: String,
    pub container_user: Option<String>,
    pub remote_password: String,
    pub container_password: String,
    pub sudo_mode: String,      // nopasswd | password | none
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            mode: "developer".to_string(),
            remote_user: "developer".to_string(),
            container_user: Some("developer".to_string()),
            remote_password: password::generate_12(),
            container_password: password::generate_12(),
            sudo_mode: "nopasswd".to_string(),
        }
    }
}

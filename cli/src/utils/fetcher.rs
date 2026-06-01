use anyhow::{Context, Result};
use std::path::Path;
use zip::ZipArchive;
use std::io::Cursor;

const REPO_URL: &str = "https://github.com/SamitoX4/devcontainers";

pub struct TemplateFetcher {
    client: reqwest::Client,
}

impl TemplateFetcher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn get_latest_version(&self, verbose: bool) -> Result<String> {
        let url = format!("{}/releases/latest", REPO_URL);
        
        if verbose {
            println!("Checking latest release...");
        }

        let response = self.client.get(&url).send().await?;
        
        if let Some(tag) = response.url().path_segments().and_then(|s| s.last()) {
            let version = tag.trim_start_matches('v').to_string();
            return Ok(version);
        }

        let response_text = response.text().await?;
        
        if let Some(pos) = response_text.find("release/tag/") {
            let slice = &response_text[pos + 12..];
            if let Some(end) = slice.find('"') {
                let tag = &slice[..end];
                let version = tag.trim_start_matches('v').to_string();
                return Ok(version);
            }
        }

        Ok("0.0.0".to_string())
    }

    pub async fn download_templates(&self, target_dir: &Path, verbose: bool) -> Result<()> {
        let archive_url = format!(
            "https://github.com/SamitoX4/devcontainers/archive/refs/heads/master.zip"
        );

        if verbose {
            println!("Downloading from: {}", archive_url);
        }

        let response = self.client
            .get(&archive_url)
            .send()
            .await
            .context("Failed to download archive")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download: HTTP {}", response.status());
        }

        let bytes = response.bytes().await?;

        if verbose {
            println!("Extracting archive...");
        }

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let path_str = path.to_string_lossy();
                    
                    if !path_str.starts_with("devcontainers-master/templates/") {
                        continue;
                    }

                    let relative = path_str.replace("devcontainers-master/templates/", "");
                    if relative.is_empty() || relative == file.name() {
                        continue;
                    }

                    target_dir.join(relative)
                }
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}

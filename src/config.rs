use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlogConfig {
    pub site_title: String,
    pub site_description: String,
}

impl Default for BlogConfig {
    fn default() -> Self {
        BlogConfig {
            site_title: "Your Blog".to_string(),
            site_description: "Your blog description".to_string(),
        }
    }
}

impl BlogConfig {
    pub fn from_file<P: AsRef<Path> + std::fmt::Debug + Clone>(
        path: P,
    ) -> Result<Self, std::io::Error> {
        if path.as_ref().exists() {
            let content = std::fs::read_to_string(path.clone())?;
            let config = serde_yaml::from_str(&content).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Config file {} could not be parsed: {e}",
                        path.as_ref().display()
                    ),
                )
            })?;
            dbg!(&config);
            Ok(config)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Config file {} not found", path.as_ref().display()),
            ))
        }
    }

    pub fn from_file_or_default<P: AsRef<Path> + std::fmt::Debug + Clone>(path: P) -> Self {
        dbg!(&path);
        match Self::from_file(path) {
            Ok(config) => config,
            Err(e) => {
                println!("Warning: Could not read config file: {e}. Using default values. To solve create a file named 'blog_config.yaml' in the root directory of your content directory with required fields.");
                Self::default()
            }
        }
    }
}

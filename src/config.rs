use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub public: Option<String>,
    pub clean_urls: Option<CleanUrls>,
    pub trailing_slash: Option<bool>,
    pub rewrites: Option<Vec<Rewrite>>,
    pub redirects: Option<Vec<Redirect>>,
    pub headers: Option<Vec<HeaderRule>>,
    pub directory_listing: Option<DirectoryListing>,
    pub symlinks: Option<bool>,
    pub etag: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum CleanUrls {
    Boolean(bool),
    Globs(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rewrite {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Redirect {
    pub source: String,
    pub destination: String,
    #[serde(rename = "type")]
    pub redirect_type: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeaderRule {
    pub source: String,
    pub headers: Vec<Header>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum DirectoryListing {
    Boolean(bool),
    Globs(Vec<String>),
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("json");

        let config = match extension {
            "json" => serde_json::from_str(&content)?,
            "yaml" | "yml" => serde_yaml::from_str(&content)?,
            "toml" => toml::from_str(&content)?,
            _ => serde_json::from_str(&content)?, // Default to JSON
        };

        Ok(config)
    }

    pub fn find_and_load(custom_path: Option<PathBuf>) -> Result<Self> {
        if let Some(path) = custom_path {
            if path.exists() {
                return Self::load(&path);
            }
        }

        let defaults = ["serve.json", "serve.yaml", "serve.yml", "serve.toml"];
        for file in defaults {
            let path = Path::new(file);
            if path.exists() {
                return Self::load(path);
            }
        }

        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_json_parsing() {
        let json = r#"{
            "public": "dist",
            "cleanUrls": true,
            "rewrites": [{"source": "/a", "destination": "/b"}]
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.public, Some("dist".to_string()));
        assert!(matches!(config.clean_urls, Some(CleanUrls::Boolean(true))));
        assert_eq!(config.rewrites.unwrap()[0].source, "/a");
    }

    #[test]
    fn test_config_yaml_parsing() {
        let yaml = "public: static\ncleanUrls: false";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.public, Some("static".to_string()));
        assert!(matches!(config.clean_urls, Some(CleanUrls::Boolean(false))));
    }

    #[test]
    fn test_config_toml_parsing() {
        let toml = "public = 'web'\ncleanUrls = true";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.public, Some("web".to_string()));
        assert!(matches!(config.clean_urls, Some(CleanUrls::Boolean(true))));
    }
}

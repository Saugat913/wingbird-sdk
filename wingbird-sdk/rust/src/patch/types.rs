use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatchMetadata {
    pub id: String,
    pub patch_number: u32,
    pub libapp_hash: String,
    pub patch_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchConfig {
    pub patch_number: u32,
    pub patch_hash: String,
    pub libapp_hash: String,
    pub release_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibCache {
    pub libapp_hash: String,
    pub release_version: String,
    pub patch_number: u32,
}

#[derive(Debug)]
pub struct LibVersion {
    pub release_version: String,
    pub patch_number: u32,
}

impl LibVersion {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let data = content.split(":").collect::<Vec<&str>>();
        let release_version = data[0].to_string();
        let patch_number = data[1].parse::<u32>()?;
        Ok(LibVersion {
            release_version,
            patch_number,
        })
    }
    
    pub fn to_file(&self, path: &str) -> anyhow::Result<()> {
        let content = format!("{}:{}", self.release_version, self.patch_number);
        std::fs::write(path, content)?;
        Ok(())
    }
}

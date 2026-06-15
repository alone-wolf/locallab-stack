use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::manifest::AppManifest;

#[derive(Clone, Debug)]
pub struct WorkspaceLayout {
    root_dir: PathBuf,
}

impl WorkspaceLayout {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn root_manifest_path(&self) -> PathBuf {
        self.root_dir.join("llstk.yml")
    }

    pub fn root_compose_path(&self) -> PathBuf {
        self.root_dir.join("docker-compose.yml")
    }

    pub fn readme_path(&self) -> PathBuf {
        self.root_dir.join("README.md")
    }

    pub fn gateway_dir(&self) -> PathBuf {
        self.root_dir.join("gateway")
    }

    pub fn gateway_caddyfile_path(&self) -> PathBuf {
        self.gateway_dir().join("Caddyfile")
    }

    pub fn gateway_data_dir(&self) -> PathBuf {
        self.gateway_dir().join("data")
    }

    pub fn gateway_config_dir(&self) -> PathBuf {
        self.gateway_dir().join("config")
    }

    pub fn certs_dir(&self) -> PathBuf {
        self.root_dir.join("certs")
    }

    pub fn certs_ca_dir(&self) -> PathBuf {
        self.certs_dir().join("ca")
    }

    pub fn certs_issued_dir(&self) -> PathBuf {
        self.certs_dir().join("issued")
    }

    pub fn cert_path(&self) -> PathBuf {
        self.certs_issued_dir().join("locallab.pem")
    }

    pub fn key_path(&self) -> PathBuf {
        self.certs_issued_dir().join("locallab-key.pem")
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.root_dir.join("templates")
    }

    pub fn app_dir(&self, name: &str) -> PathBuf {
        self.root_dir.join(format!("app.{name}"))
    }

    pub fn app_manifest_path(&self, name: &str) -> PathBuf {
        self.app_dir(name).join("llstk.yml")
    }

    pub fn app_compose_path(&self, name: &str) -> PathBuf {
        self.app_dir(name).join("docker-compose.yml")
    }

    pub fn discover_app_manifests(&self) -> Result<Vec<(String, PathBuf)>> {
        let mut apps = Vec::new();
        if !self.root_dir.exists() {
            return Ok(apps);
        }
        for entry in fs::read_dir(&self.root_dir)
            .with_context(|| format!("failed to read {}", self.root_dir.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let Some(app_name) = file_name.strip_prefix("app.") else {
                continue;
            };
            let manifest = entry.path().join("llstk.yml");
            if manifest.exists() {
                apps.push((app_name.to_string(), manifest));
            }
        }
        apps.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(apps)
    }

    pub fn read_apps(&self) -> Result<Vec<AppManifest>> {
        self.discover_app_manifests()?
            .into_iter()
            .map(|(_, path)| AppManifest::read_from(&path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_paths() {
        let layout = WorkspaceLayout::new(PathBuf::from(".custom"));
        assert_eq!(
            layout.root_manifest_path(),
            PathBuf::from(".custom/llstk.yml")
        );
        assert_eq!(layout.app_dir("gitea"), PathBuf::from(".custom/app.gitea"));
    }
}

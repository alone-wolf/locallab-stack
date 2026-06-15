use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RootManifest {
    pub version: u32,
    pub name: String,
    pub root: String,
    pub network: NetworkConfig,
    pub gateway: GatewayConfig,
    pub cert: CertConfig,
    pub hosts: HostsConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NetworkConfig {
    pub global: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GatewayConfig {
    pub provider: String,
    pub container: String,
    pub http_port: u16,
    pub https_port: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CertConfig {
    pub provider: String,
    pub domains: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HostsConfig {
    pub enabled: bool,
    pub ip: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppManifest {
    pub version: u32,
    pub name: String,
    pub domain: String,
    pub upstreams: BTreeMap<String, Upstream>,
    #[serde(default)]
    pub ports: BTreeMap<String, PortMapping>,
    #[serde(default)]
    pub services: BTreeMap<String, Service>,
    #[serde(default)]
    pub data: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Upstream {
    pub container: String,
    pub port: u16,
    pub public: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PortMapping {
    pub host: u16,
    pub container: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Service {
    pub public: bool,
    pub networks: Vec<ServiceNetwork>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceNetwork {
    Global,
    Private,
}

impl RootManifest {
    pub fn default_named(name: impl Into<String>) -> Self {
        Self {
            version: 1,
            name: name.into(),
            root: "./.locallab".to_string(),
            network: NetworkConfig {
                global: "locallabstack-global".to_string(),
            },
            gateway: GatewayConfig {
                provider: "caddy".to_string(),
                container: "locallabstack-gateway".to_string(),
                http_port: 80,
                https_port: 443,
            },
            cert: CertConfig {
                provider: "mkcert".to_string(),
                domains: vec!["locallab".to_string(), "*.locallab".to_string()],
            },
            hosts: HostsConfig {
                enabled: true,
                ip: "127.0.0.1".to_string(),
            },
        }
    }

    pub fn read_from(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read root manifest {}", path.display()))?;
        let manifest: Self = serde_yaml::from_str(&text)
            .with_context(|| format!("failed to parse root manifest {}", path.display()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn write_to(&self, path: &Path) -> Result<()> {
        let text = serde_yaml::to_string(self)?;
        fs::write(path, text)
            .with_context(|| format!("failed to write root manifest {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("unsupported manifest version {}", self.version);
        }
        if self.name.trim().is_empty() {
            bail!("stack name cannot be empty");
        }
        if self.network.global.trim().is_empty() {
            bail!("network.global cannot be empty");
        }
        if self.gateway.provider != "caddy" {
            bail!("unsupported gateway provider {}", self.gateway.provider);
        }
        if self.cert.provider != "mkcert" {
            bail!("unsupported cert provider {}", self.cert.provider);
        }
        if self.hosts.ip.trim().is_empty() {
            bail!("hosts.ip cannot be empty");
        }
        Ok(())
    }
}

impl AppManifest {
    pub fn default_for(name: &str, domain: Option<String>) -> Result<Self> {
        validate_app_name(name)?;
        let domain = domain.unwrap_or_else(|| format!("{name}.locallab"));
        let mut upstreams = BTreeMap::new();
        upstreams.insert(
            "web".to_string(),
            Upstream {
                container: name.to_string(),
                port: 80,
                public: true,
            },
        );
        let mut services = BTreeMap::new();
        services.insert(
            name.to_string(),
            Service {
                public: true,
                networks: vec![ServiceNetwork::Global],
            },
        );
        let manifest = Self {
            version: 1,
            name: name.to_string(),
            domain,
            upstreams,
            ports: BTreeMap::new(),
            services,
            data: vec!["./data".to_string()],
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn read_from(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read app manifest {}", path.display()))?;
        let manifest: Self = serde_yaml::from_str(&text)
            .with_context(|| format!("failed to parse app manifest {}", path.display()))?;
        manifest
            .validate()
            .with_context(|| format!("invalid app manifest {}", path.display()))?;
        Ok(manifest)
    }

    pub fn write_to(&self, path: &Path) -> Result<()> {
        let text = serde_yaml::to_string(self)?;
        fs::write(path, text)
            .with_context(|| format!("failed to write app manifest {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("unsupported manifest version {}", self.version);
        }
        validate_app_name(&self.name)?;
        validate_domain(&self.domain)?;
        for (name, upstream) in &self.upstreams {
            if upstream.container.trim().is_empty() {
                bail!("upstream {name} container cannot be empty");
            }
            if upstream.public {
                let Some(service) = self.services.get(&upstream.container) else {
                    bail!(
                        "public upstream {name} references missing service {}",
                        upstream.container
                    );
                };
                if !service.networks.contains(&ServiceNetwork::Global) {
                    bail!(
                        "public upstream {name} service {} must join global network",
                        upstream.container
                    );
                }
            }
        }
        Ok(())
    }

    pub fn public_upstreams(&self) -> Vec<(&String, &Upstream)> {
        self.upstreams
            .iter()
            .filter(|(_, upstream)| upstream.public)
            .collect()
    }
}

pub fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("app name cannot be empty");
    }
    if name.starts_with('-') || name.ends_with('-') {
        bail!("app name must not start or end with '-'");
    }
    let mut previous_dash = false;
    for ch in name.chars() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !valid {
            bail!(
                "invalid app name {name}; use lowercase letters, digits, and single '-' separators"
            );
        }
        if ch == '-' && previous_dash {
            bail!("app name must not contain repeated '-'");
        }
        previous_dash = ch == '-';
    }
    Ok(())
}

pub fn validate_domain(domain: &str) -> Result<()> {
    if domain.trim().is_empty() {
        bail!("domain cannot be empty");
    }
    if domain.contains(' ') || domain.contains('/') {
        bail!("domain must not contain spaces or slashes");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_manifest_round_trips() {
        let manifest = RootManifest::default_named("default");
        let text = serde_yaml::to_string(&manifest).unwrap();
        let parsed: RootManifest = serde_yaml::from_str(&text).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn app_manifest_round_trips() {
        let manifest = AppManifest::default_for("demo", None).unwrap();
        let text = serde_yaml::to_string(&manifest).unwrap();
        let parsed: AppManifest = serde_yaml::from_str(&text).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn validates_app_names() {
        assert!(validate_app_name("gitea").is_ok());
        assert!(validate_app_name("api-demo1").is_ok());
        assert!(validate_app_name("api_demo").is_err());
        assert!(validate_app_name("-api").is_err());
        assert!(validate_app_name("api--demo").is_err());
    }

    #[test]
    fn public_upstream_requires_global_network() {
        let mut manifest = AppManifest::default_for("demo", None).unwrap();
        manifest.services.get_mut("demo").unwrap().networks.clear();
        assert!(manifest.validate().is_err());
    }
}
